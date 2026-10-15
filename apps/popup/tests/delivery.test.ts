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
});
