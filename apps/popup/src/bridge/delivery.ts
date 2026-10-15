import { summaryText, type PopupResult } from "../core";
export interface Envelope {
  type: "popup_result";
  formId: string;
  submissionId: string;
  result: PopupResult;
}
export interface MessageSender {
  sendMessage(params: {
    role: "user";
    content: { type: "text"; text: string }[];
  }): Promise<{ isError?: boolean }>;
  /**
   * Present when the host accepts model context. The envelope then travels there, and the user's
   * turn carries a readable summary instead of raw JSON.
   */
  updateModelContext?(params: {
    content: { type: "text"; text: string }[];
    structuredContent: Record<string, unknown>;
  }): Promise<unknown>;
}
export function createDelivery(
  formId: string,
  sender: MessageSender,
  storage?: Pick<Storage, "getItem" | "setItem">,
  options: { title?: string } = {},
) {
  const key = `popup:receipt:${formId}`;
  let acknowledged = false;
  try {
    acknowledged =
      storage?.getItem(key) !== null && storage?.getItem(key) !== undefined;
  } catch {
    /* Storage is optional in sandboxed hosts. */
  }
  let envelope: Envelope | undefined;
  let inflight: Promise<void> | undefined;
  /** The message text: a summary when the envelope reached model context, otherwise the envelope itself. */
  async function messageText(envelope: Envelope): Promise<string> {
    const json = JSON.stringify(envelope);
    try {
      // Failure is reported by rejection; fall back to carrying the envelope in the message.
      await sender.updateModelContext!({
        content: [{ type: "text", text: json }],
        structuredContent: envelope as unknown as Record<string, unknown>,
      });
    } catch {
      return json;
    }
    return summaryText(envelope.result, {
      title: options.title,
      submissionId: envelope.submissionId,
    });
  }
  return {
    get completed() {
      return acknowledged;
    },
    send(result: PopupResult): Promise<void> {
      if (acknowledged) return Promise.resolve();
      if (inflight) return inflight;
      if (!envelope)
        envelope = {
          type: "popup_result",
          formId,
          submissionId: crypto.randomUUID(),
          result: JSON.parse(JSON.stringify(result)) as PopupResult,
        };
      inflight = (async () => {
        // Without model context the message goes out synchronously, as it always has.
        const text = sender.updateModelContext
          ? await messageText(envelope!)
          : JSON.stringify(envelope);
        const reply = await sender.sendMessage({
          role: "user",
          content: [{ type: "text", text }],
        });
        if (reply.isError)
          throw new Error(
            "The host did not accept your response. Retry delivery or copy it into chat.",
          );
        acknowledged = true;
        try {
          storage?.setItem(
            key,
            JSON.stringify({
              submissionId: envelope!.submissionId,
              status: result.status,
            }),
          );
        } catch {
          /* Never undo acknowledged delivery. */
        }
      })().finally(() => {
        inflight = undefined;
      });
      return inflight;
    },
  };
}
