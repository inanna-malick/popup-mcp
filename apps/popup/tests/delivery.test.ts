import { describe, expect, it, vi } from "vitest";
import { createDelivery } from "../src/bridge/delivery";
describe("terminal response delivery", () => {
  it("retries the same frozen envelope and does not resend after acknowledgment", async () => {
    const sender = {
      sendMessage: vi
        .fn()
        .mockResolvedValueOnce({ isError: true })
        .mockResolvedValue({}),
    };
    const delivery = createDelivery("form-1", sender);
    const result = {
      status: "completed" as const,
      button: "submit",
      _labels: {},
      answer: "original",
    };
    await expect(delivery.send(result)).rejects.toThrow("did not accept");
    result.answer = "changed";
    await delivery.send(result);
    await delivery.send(result);
    expect(sender.sendMessage).toHaveBeenCalledTimes(2);
    expect(sender.sendMessage.mock.calls[0]).toEqual(
      sender.sendMessage.mock.calls[1],
    );
    expect(delivery.completed).toBe(true);
  });
  it("coalesces concurrent submissions", async () => {
    let finish!: (result: {}) => void;
    const sender = {
      sendMessage: vi.fn(
        () =>
          new Promise<{}>((resolve) => {
            finish = resolve;
          }),
      ),
    };
    const delivery = createDelivery("form-2", sender);
    const a = delivery.send({ status: "cancelled" }),
      b = delivery.send({ status: "cancelled" });
    expect(a).toBe(b);
    finish({});
    await a;
    expect(sender.sendMessage).toHaveBeenCalledTimes(1);
  });
  it("stores only terminal metadata; storage failure does not undo host acceptance", async () => {
    const data = new Map<string, string>();
    const storage = {
      getItem: (key: string) => data.get(key) ?? null,
      setItem: (key: string, value: string) => {
        data.set(key, value);
      },
    };
    const sender = { sendMessage: vi.fn().mockResolvedValue({}) };
    await createDelivery("form-3", sender, storage).send({
      status: "completed",
      _labels: {},
      secret: "not-in-receipt",
    });
    expect([...data.values()].join("")).not.toContain("not-in-receipt");
    const replay = createDelivery("form-3", sender, storage);
    expect(replay.completed).toBe(true);
    await replay.send({ status: "cancelled" });
    expect(sender.sendMessage).toHaveBeenCalledTimes(1);
    const broken = {
      getItem: () => {
        throw new Error("blocked");
      },
      setItem: () => {
        throw new Error("blocked");
      },
    };
    const delivery = createDelivery("form-4", sender, broken);
    await delivery.send({ status: "cancelled" });
    expect(delivery.completed).toBe(true);
  });
  it("puts the envelope in model context and sends a readable turn when the host allows", async () => {
    const sender = {
      sendMessage: vi.fn().mockResolvedValue({}),
      updateModelContext: vi.fn().mockResolvedValue({}),
    };
    await createDelivery("form-5", sender, undefined, {
      title: "Pick one",
    }).send({
      status: "completed",
      button: "submit",
      choice: "a",
      flag: true,
      _labels: { choice: "Choice", flag: "Flag" },
      _option_labels: { choice: { a: "Option A" } },
      _defaulted: ["flag"],
    });
    const context = sender.updateModelContext.mock.calls[0][0];
    expect(context.structuredContent).toMatchObject({
      type: "popup_result",
      formId: "form-5",
    });
    const text: string = sender.sendMessage.mock.calls[0][0].content[0].text;
    expect(text).toBe(
      `Popup response · Pick one\n- Choice: Option A\n- Flag: Yes (default)\n(response ${context.structuredContent.submissionId})`,
    );
    expect(sender.updateModelContext.mock.invocationCallOrder[0]).toBeLessThan(
      sender.sendMessage.mock.invocationCallOrder[0],
    );
  });
  it("falls back to the JSON envelope when model context fails", async () => {
    const sender = {
      sendMessage: vi.fn().mockResolvedValue({}),
      updateModelContext: vi.fn().mockRejectedValue(new Error("no")),
    };
    await createDelivery("form-6", sender).send({ status: "cancelled" });
    expect(
      JSON.parse(sender.sendMessage.mock.calls[0][0].content[0].text),
    ).toMatchObject({
      type: "popup_result",
      result: { status: "cancelled" },
    });
  });
});
