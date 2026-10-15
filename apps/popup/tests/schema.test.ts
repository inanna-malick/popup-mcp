import { readFileSync, readdirSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { Client } from "@modelcontextprotocol/client";
import {
  InMemoryTransport,
  fromJsonSchema,
} from "@modelcontextprotocol/server";
import { RESOURCE_MIME_TYPE } from "@modelcontextprotocol/ext-apps/server";
import { createServer, RESOURCE_URI, type Env } from "../src/server/mcp";
import { compileDefinition } from "../src/core";
import { popupDescription, popupInputSchema } from "../src/authoring";
import { scenarios, repairCases } from "../evals/scenarios";
import { selfSample } from "../evals/self-sample";

const schema = fromJsonSchema(
  popupInputSchema as Parameters<typeof fromJsonSchema>[0],
);
const examplesDirectory = new URL("../../../examples/", import.meta.url);
const files = readdirSync(examplesDirectory).filter((f) => f.endsWith(".json"));

describe("published schema and compatibility", () => {
  it("accepts an unedited fresh-agent DSL smoke sample", async () => {
    const input = JSON.parse(
      readFileSync(new URL("../evals/dsl-smoke.json", import.meta.url), "utf8"),
    );
    expect(await schema["~standard"].validate(input)).not.toHaveProperty(
      "issues",
    );
    expect(() => compileDefinition(input)).not.toThrow();
  });
  it("publishes stable choices, recommendations and visual-only disclosure consistently", async () => {
    const valid = [
      {
        elements: [
          {
            multi: "Directions",
            id: "directions",
            options: [
              {
                id: "inspect",
                label: "Inspect behavior",
                recommendation: "Addresses the observed issue",
                children: [{ check: "Check initial expansion", id: "initial" }],
              },
            ],
          },
        ],
      },
      {
        elements: [
          {
            disclosure: "Inspect a proposal",
            open: false,
            elements: [
              {
                input: "Working wording",
                id: "wording",
                default: "Keep this draft",
              },
            ],
          },
        ],
      },
      {
        elements: [
          {
            select: "Mode",
            id: "mode",
            options: [
              {
                id: "default",
                label: "Use defaults",
                children: [{ input: "Detail" }],
              },
            ],
            default: "default",
          },
        ],
      },
      {
        elements: [
          {
            rank: "Priorities",
            id: "priorities",
            options: [
              {
                id: "speed",
                label: "Latency",
                recommendation: "Largest observed cost",
              },
              { id: "cost", label: "Runner cost" },
            ],
          },
        ],
      },
      {
        elements: [
          {
            multi: "Mixed choices",
            options: ["Classic", { id: "stable", label: "Stable choice" }],
          },
        ],
      },
    ];
    for (const definition of valid) {
      expect(() => compileDefinition(definition)).not.toThrow();
      expect(await schema["~standard"].validate(definition)).not.toHaveProperty(
        "issues",
      );
    }
    const invalid = [
      {
        elements: [
          { select: "Choice", options: [{ id: "blank", label: " " }] },
        ],
      },
      {
        elements: [
          {
            select: "Choice",
            options: [{ id: "a", label: "A", recommendation: " " }],
          },
        ],
      },
      { elements: [{ select: "Choice", options: [{ id: "choice" }] }] },
      {
        elements: [
          {
            select: "Choice",
            options: [{ id: "choice", label: "Choice", value: "Legacy" }],
          },
        ],
      },
      {
        elements: [
          {
            select: "Choice",
            options: [{ id: "choice", label: "Choice", recommendation: true }],
          },
        ],
      },
      {
        elements: [
          {
            rank: "Rank",
            options: [
              { id: "a", label: "A", children: [] },
              { id: "b", label: "B" },
            ],
          },
        ],
      },
      { elements: [{ disclosure: "Inspect", open: "yes", elements: [] }] },
    ];
    for (const definition of invalid) {
      expect(() => compileDefinition(definition)).toThrow();
      expect(await schema["~standard"].validate(definition)).toHaveProperty(
        "issues",
      );
    }
  });
  it("accepts omitted and empty titles while rejecting non-string titles", async () => {
    for (const input of [
      { elements: [{ input: "Your correction" }] },
      { title: "", elements: [] },
    ]) {
      expect(compileDefinition(input).title).toBe("");
      expect(await schema["~standard"].validate(input)).not.toHaveProperty(
        "issues",
      );
    }
    for (const title of [null, 12, false]) {
      const input = { title, elements: [] };
      expect(() => compileDefinition(input)).toThrow();
      expect(await schema["~standard"].validate(input)).toHaveProperty(
        "issues",
      );
    }
  });
  it("audits every repository JSON example", async () => {
    for (const file of files) {
      const input = JSON.parse(
        readFileSync(new URL(file, examplesDirectory), "utf8"),
      );
      if (file === "incident_triage_tree.json") {
        // Rust semantic-routing example intentionally outside the linear classic scope.
        expect(() => compileDefinition(input)).toThrow(
          /next.*linear navigation/,
        );
        expect(await schema["~standard"].validate(input)).toHaveProperty(
          "issues",
        );
      } else {
        expect(() => compileDefinition(input), file).not.toThrow();
        expect(
          await schema["~standard"].validate(input),
          file,
        ).not.toHaveProperty("issues");
      }
    }
    expect(files).toHaveLength(23);
  });
  it("validates executable reference examples with the SDK default runtime validator", async () => {
    for (const input of [
      ...popupInputSchema.examples,
      ...scenarios.map((s) => s.definition),
      selfSample.definition,
    ]) {
      expect(await schema["~standard"].validate(input)).not.toHaveProperty(
        "issues",
      );
      expect(() => compileDefinition(input)).not.toThrow();
    }
    for (const fixture of repairCases)
      expect(() => compileDefinition(fixture.definition)).toThrow();
    for (const input of [
      { title: "Bad rows", elements: [{ input: "Text", rows: 101 }] },
      {
        title: "Aliases",
        elements: [
          {
            select: "Choice",
            options: [{ value: "A", description: "X", because: "Y" }],
          },
        ],
      },
      {
        title: "Routing",
        elements: [{ page: "Step", elements: [], next: [] }],
      },
    ]) {
      expect(await schema["~standard"].validate(input)).toHaveProperty(
        "issues",
      );
      expect(() => compileDefinition(input)).toThrow();
    }
  });
  it("publishes and executes the actual MCP tool and its UI resource", async () => {
    const html = "<!doctype html><html><body>Popup test asset</body></html>";
    const assetRequests: string[] = [];
    const server = createServer({
      ASSETS: {
        fetch: async (request: Request) => {
          assetRequests.push(request.url);
          return new Response(html, {
            headers: { "content-type": "text/html" },
          });
        },
      } as unknown as Env["ASSETS"],
    });
    const client = new Client({ name: "schema-test", version: "1" });
    const [clientTransport, serverTransport] =
      InMemoryTransport.createLinkedPair();
    try {
      await Promise.all([
        server.connect(serverTransport),
        client.connect(clientTransport),
      ]);
      const { tools } = await client.listTools();
      expect(tools[0].inputSchema).toEqual(popupInputSchema);
      expect(tools[0].description).toBe(popupDescription);
      expect(tools[0]._meta).toMatchObject({
        ui: { resourceUri: RESOURCE_URI },
      });
      const definition = popupInputSchema.examples[0];
      const opened = await client.callTool({
        name: "popup",
        arguments: definition,
      });
      expect(opened.isError).not.toBe(true);
      expect(opened.structuredContent).toMatchObject({
        status: "awaiting_input",
        definition,
      });
      expect(
        (opened.structuredContent as Record<string, unknown>).formId,
      ).toMatch(/^[0-9a-f-]{36}$/);
      const untitled = await client.callTool({
        name: "popup",
        arguments: { elements: [{ input: "What would you change?" }] },
      });
      expect(untitled.isError).not.toBe(true);
      expect(untitled.content).toContainEqual({
        type: "text",
        text: expect.stringMatching(/^What would you change\?/),
      });
      const suggested = await client.callTool({
        name: "popup",
        arguments: {
          elements: [
            {
              multi: "Next steps",
              id: "steps",
              options: [
                {
                  id: "investigate",
                  label: "Inspect behavior",
                  recommendation: "This tests the reported issue",
                },
              ],
            },
          ],
        },
      });
      expect(suggested.isError).not.toBe(true);
      expect(suggested.content).toContainEqual({
        type: "text",
        text: expect.stringContaining(
          "Inspect behavior [Recommended: This tests the reported issue]",
        ),
      });
      const invalid = await client.callTool({
        name: "popup",
        arguments: repairCases[0].definition,
      });
      expect(invalid.isError).toBe(true);
      expect(JSON.stringify(invalid)).toMatch(/Postgress/);
      expect(JSON.stringify(invalid)).toMatch(/Postgres/);
      const resources = await client.listResources();
      expect(resources.resources).toContainEqual(
        expect.objectContaining({
          uri: RESOURCE_URI,
          mimeType: RESOURCE_MIME_TYPE,
        }),
      );
      const asset = await client.readResource({ uri: RESOURCE_URI });
      expect(asset.contents).toEqual([
        {
          uri: RESOURCE_URI,
          mimeType: RESOURCE_MIME_TYPE,
          text: html,
          _meta: {
            ui: {
              csp: { connectDomains: [], resourceDomains: [] },
              prefersBorder: false,
            },
          },
        },
      ]);
      expect(assetRequests).toEqual(["https://assets.local/index.html"]);
    } finally {
      await Promise.all([client.close(), server.close()]);
    }
  });
});
