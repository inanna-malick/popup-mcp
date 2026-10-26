import type { PopupResult } from "../core";
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
}
export function createDelivery(
  formId: string,
  sender: MessageSender,
  storage?: Pick<Storage, "getItem" | "setItem">,
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
      const text = JSON.stringify(envelope);
      inflight = (async () => {
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
