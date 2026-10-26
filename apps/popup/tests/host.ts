import {
  AppBridge,
  McpUiHostStylesSchema,
  PostMessageTransport,
} from "@modelcontextprotocol/ext-apps/app-bridge";
import type { Envelope } from "../src/bridge/delivery";
import {
  Client,
  StreamableHTTPClientTransport,
} from "@modelcontextprotocol/client";
interface Options {
  rejectMessages?: number;
  messaging?: boolean;
  formId?: string;
  theme?: "light" | "dark";
  endpoint?: string;
  styleVariables?: Record<string, string>;
}
const harness = window as unknown as {
  showForm: (definition: unknown, options?: Options) => Promise<void>;
  messages: Envelope[];
  attempts: Envelope[];
  bridge?: AppBridge;
  sizes: { width?: number; height?: number }[];
};
harness.messages = [];
harness.attempts = [];
harness.sizes = [];
harness.showForm = async (definition, options = {}) => {
  await harness.bridge?.close();
  document.querySelector("iframe")?.remove();
  let formId = options.formId ?? crypto.randomUUID();
  let toolResult: Parameters<AppBridge["sendToolResult"]>[0] = {
    content: [],
    structuredContent: { status: "awaiting_input", formId, definition },
  };
  let html: string | undefined;
  let client: Client | null = null;
  if (options.endpoint) {
    client = new Client({ name: "popup-reference-host", version: "1" });
    await client.connect(
      new StreamableHTTPClientTransport(new URL(options.endpoint)),
    );
    toolResult = await client.callTool({
      name: "popup",
      arguments: definition as Record<string, unknown>,
    });
    const resource = await client.readResource({ uri: "ui://popup/form.html" });
    const item = resource.contents[0];
    if (!("text" in item)) throw new Error("Expected HTML resource");
    html = item.text;
  }
  const iframe = document.createElement("iframe");
  iframe.dataset.testid = "app";
  iframe.title = "Popup form";
  iframe.setAttribute("sandbox", "allow-scripts allow-same-origin");
  document.body.appendChild(iframe);
  let rejected = options.rejectMessages ?? 0;
  const bridge = new AppBridge(
    client,
    { name: "Popup reference bridge harness", version: "1" },
    {
      ...(options.messaging === false ? {} : { message: { text: {} } }),
      openLinks: {},
    },
    {
      hostContext: {
        theme: options.theme ?? "light",
        displayMode: "inline",
        styles: McpUiHostStylesSchema.parse({
          variables: options.styleVariables ?? {},
        }),
      },
    },
  );
  harness.bridge = bridge;
  bridge.onsizechange = (size) => {
    harness.sizes.push(size);
  };
  bridge.onmessage = async (params) => {
    const text = params.content.find((item) => item.type === "text");
    if (!text || text.type !== "text")
      throw new Error("Expected text response");
    const envelope = JSON.parse(text.text) as Envelope;
    harness.attempts.push(envelope);
    if (rejected-- > 0) return { isError: true };
    harness.messages.push(envelope);
    return {};
  };
  bridge.onopenlink = async () => ({});
  const ready = new Promise<void>((resolve, reject) => {
    bridge.oninitialized = () => {
      void (async () => {
        await bridge.sendToolInput({
          arguments: definition as Record<string, unknown>,
        });
        await bridge.sendToolResult(toolResult);
        resolve();
      })().catch(reject);
    };
  });
  await bridge.connect(
    new PostMessageTransport(iframe.contentWindow!, iframe.contentWindow!),
  );
  if (html) iframe.srcdoc = html;
  else iframe.src = "/dist/ui/index.html";
  await ready;
};
