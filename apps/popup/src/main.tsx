import { createSignal, Show } from "solid-js";
import { render } from "solid-js/web";
import {
  App,
  applyDocumentTheme,
  applyHostStyleVariables,
} from "@modelcontextprotocol/ext-apps";
import { PopupForm } from "./ui/PopupForm";
import { createDelivery } from "./bridge/delivery";
import { compileDefinition, summaryText, type PopupResult } from "./core";

const app = new App(
  { name: "Popup", version: "0.1.0" },
  {},
  { autoResize: true },
);
const [pending, setPending] = createSignal("Connecting to your conversation…");
const [form, setForm] = createSignal<{
  formId: string;
  definition: unknown;
  delivery: ReturnType<typeof createDelivery>;
  drafts: ReturnType<typeof draftPublisher>;
}>();
let storage: Storage | undefined;
try {
  storage = window.localStorage;
} catch {
  /* Sandboxed storage may be unavailable. */
}
/**
 * Keeps the model's view of an unsubmitted form current, so an answer given in chat instead (or
 * after the frame reloads) can account for what the user had already picked. Drafts stop for good
 * once the user submits or cancels, so they can never overwrite the real answer.
 */
function draftPublisher(formId: string, title: string | undefined) {
  let timer: ReturnType<typeof setTimeout> | undefined;
  let finished = false;
  return {
    publish(draft: PopupResult) {
      if (finished || !app.getHostCapabilities()?.updateModelContext?.text)
        return;
      clearTimeout(timer);
      timer = setTimeout(() => {
        if (finished) return;
        const text = summaryText(draft, { title, submissionId: "draft" })
          .replace(/^Popup response/, "Popup draft, not submitted")
          .replace(/\n\(response draft\)$/, "");
        void app
          .updateModelContext({
            content: [{ type: "text", text }],
            structuredContent: { type: "popup_draft", formId, result: draft },
          })
          .catch(() => {});
      }, 500);
    },
    finish() {
      finished = true;
      clearTimeout(timer);
    },
  };
}
function formTitle(definition: unknown): string | undefined {
  const title = (definition as { title?: unknown } | null)?.title;
  return typeof title === "string" ? title : undefined;
}
/**
 * A host that bounds the frame's height gets a form that scrolls inside that bound with its
 * actions pinned, instead of growing past the host's view and hiding Submit.
 */
function bound(context: ReturnType<typeof app.getHostContext>) {
  const dims = context?.containerDimensions as
    { height?: number; maxHeight?: number } | undefined;
  const limit = dims?.height ?? dims?.maxHeight;
  const root = document.documentElement;
  root.classList.toggle("bounded", !!limit);
  if (limit) root.style.setProperty("--popup-max-height", `${limit}px`);
  else root.style.removeProperty("--popup-max-height");
}
function theme(context: ReturnType<typeof app.getHostContext>) {
  bound(context);
  if (context?.theme) applyDocumentTheme(context.theme);
  if (context?.styles?.variables)
    applyHostStyleVariables(context.styles.variables);
}
app.onhostcontextchanged = (context) => theme(context);
app.ontoolinputpartial = () => {
  if (!form()) setPending("Preparing your interaction…");
};
app.ontoolinput = () => {
  if (!form()) setPending("Preparing your interaction…");
};
app.ontoolcancelled = () => {
  setForm(undefined);
  setPending("This form was cancelled.");
};
app.ontoolresult = (result) => {
  if (form()) return; // Replayed results must never reset an in-progress answer.
  if (result.isError) {
    setPending(
      "The form could not be opened. The agent can correct it and try again.",
    );
    return;
  }
  const raw = result.structuredContent;
  const data =
    raw && typeof raw === "object"
      ? (raw as Record<string, unknown>)
      : undefined;
  if (
    !data ||
    typeof data.formId !== "string" ||
    data.status !== "awaiting_input"
  ) {
    setPending("The host returned an unsupported form response.");
    return;
  }
  try {
    compileDefinition(data.definition);
    const delivery = createDelivery(
      data.formId,
      {
        async sendMessage(message) {
          if (!app.getHostCapabilities()?.message?.text)
            throw new Error(
              "This host cannot receive form answers directly. Copy your response and paste it into the conversation.",
            );
          try {
            return await app.sendMessage(message, { timeout: 15000 });
          } catch {
            throw new Error(
              "Delivery could not be confirmed. Retry sends the same response identifier, or copy your response into chat.",
            );
          }
        },
        ...(app.getHostCapabilities()?.updateModelContext?.text
          ? {
              updateModelContext: (params) =>
                app.updateModelContext(params, { timeout: 15000 }),
            }
          : {}),
      },
      storage,
      { title: formTitle(data.definition) },
    );
    setForm({
      formId: data.formId,
      definition: data.definition,
      delivery,
      drafts: draftPublisher(data.formId, formTitle(data.definition)),
    });
  } catch (error) {
    setPending(error instanceof Error ? error.message : "Invalid form");
  }
};
const dispose = render(
  () => (
    <Show
      when={form()}
      fallback={
        <main class="popup">
          <p role="status">{pending()}</p>
        </main>
      }
    >
      {(current) => (
        <PopupForm
          definition={current().definition}
          initiallyCompleted={current().delivery.completed}
          onSubmit={(result) => {
            current().drafts.finish();
            return current().delivery.send(result);
          }}
          onCancel={() => {
            current().drafts.finish();
            return current().delivery.send({ status: "cancelled" });
          }}
          onDraft={(draft) => current().drafts.publish(draft)}
          onOpenLink={async (url) => {
            const result = await app.openLink({ url });
            if (result.isError)
              throw new Error("The host could not open this link.");
          }}
        />
      )}
    </Show>
  ),
  document.getElementById("root")!,
);
window.addEventListener(
  "pagehide",
  () => {
    dispose();
    void app.close();
  },
  { once: true },
);
if (window.parent === window)
  setPending(
    "Connect an MCP Apps client to this Worker’s /mcp endpoint to open a form.",
  );
else
  void app
    .connect()
    .then(() => theme(app.getHostContext()))
    .catch(() =>
      setPending(
        "Could not connect to the host. Please reopen this form from your conversation.",
      ),
    );
