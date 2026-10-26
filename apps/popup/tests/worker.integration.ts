import { afterAll, beforeAll, describe, expect, it } from "vitest";
import {
  Client,
  StreamableHTTPClientTransport,
} from "@modelcontextprotocol/client";
import { popupInputSchema, popupDescription } from "../src/authoring";
const endpoint = process.env.POPUP_MCP_URL ?? "http://127.0.0.1:8787/mcp";
const client = new Client({ name: "popup-worker-smoke", version: "1" });
beforeAll(async () => {
  await client.connect(new StreamableHTTPClientTransport(new URL(endpoint)));
});
afterAll(async () => {
  await client.close();
});
describe("real Worker MCP HTTP", () => {
  it("publishes the exact schema, prompt, and UI metadata", async () => {
    const { tools } = await client.listTools();
    expect(tools).toHaveLength(1);
    expect(tools[0].name).toBe("popup");
    expect(tools[0].inputSchema).toEqual(popupInputSchema);
    expect(tools[0].description).toBe(popupDescription);
    expect(tools[0]._meta?.ui).toMatchObject({
      resourceUri: "ui://popup/form.html",
    });
  });
  it("presents a form immediately and returns self-contained app HTML", async () => {
    const response = await client.callTool({
      name: "popup",
      arguments: popupInputSchema.examples[0],
    });
    expect(response.isError).not.toBe(true);
    expect(response.structuredContent).toMatchObject({
      status: "awaiting_input",
      formId: expect.any(String),
      definition: popupInputSchema.examples[0],
    });
    const { contents } = await client.readResource({
      uri: "ui://popup/form.html",
    });
    expect(contents[0].mimeType).toBe("text/html;profile=mcp-app");
    expect("text" in contents[0] && contents[0].text).toContain(
      "<!doctype html>",
    );
    expect("text" in contents[0] && contents[0].text).not.toMatch(
      /<script[^>]+src=/,
    );
  });
  it("returns actionable authoring errors without opening a form", async () => {
    const response = await client.callTool({
      name: "popup",
      arguments: {
        title: "Bad branch",
        elements: [{ select: "Choice", options: ["Alpha", "Beta"], Alhpa: [] }],
      },
    });
    expect(response.isError).toBe(true);
    expect(JSON.stringify(response.content)).toContain("Alhpa");
    expect(response.structuredContent ?? {}).not.toHaveProperty("formId");
  });
  it("supports public CORS and rejects oversized protocol bodies", async () => {
    const preflight = await fetch(endpoint, {
      method: "OPTIONS",
      headers: { Origin: "https://host.example" },
    });
    expect(preflight.status).toBe(204);
    expect(preflight.headers.get("Access-Control-Allow-Origin")).toBe("*");
    const huge = await fetch(endpoint, {
      method: "POST",
      body: "x".repeat(2 * 1024 * 1024 + 1),
    });
    expect(huge.status).toBe(413);
  });
});
