import { McpServer, fromJsonSchema } from "@modelcontextprotocol/server";
import {
  registerAppResource,
  registerAppTool,
  RESOURCE_MIME_TYPE,
} from "@modelcontextprotocol/ext-apps/server";
import { popupDescription, popupInputSchema } from "../authoring";
import { compileDefinition, computeVisibility, DefinitionError } from "../core";

export const RESOURCE_URI = "ui://popup/form.html";
export interface Env {
  ASSETS: Fetcher;
}
export function createServer(env: Env) {
  const server = new McpServer({ name: "popup", version: "0.1.0" });
  registerAppTool(
    server,
    "popup",
    {
      title: "Have a structured conversation",
      description: popupDescription,
      inputSchema: fromJsonSchema(
        popupInputSchema as Parameters<typeof fromJsonSchema>[0],
        {
          getValidator<T>() {
            return (input: unknown) => {
              try {
                compileDefinition(input);
                return {
                  valid: true as const,
                  data: input as T,
                  errorMessage: undefined,
                };
              } catch (error) {
                return {
                  valid: false as const,
                  data: undefined,
                  errorMessage:
                    error instanceof Error ? error.message : "Invalid form",
                };
              }
            };
          },
        },
      ),
      annotations: {
        readOnlyHint: true,
        destructiveHint: false,
        openWorldHint: false,
      },
      _meta: { ui: { resourceUri: RESOURCE_URI } },
    },
    async (definition) => {
      try {
        const form = compileDefinition(definition);
        const formId = crypto.randomUUID();
        const visible = computeVisibility(form, form.defaults);
        const outline = form.all
          .filter((node) => visible.has(node.key))
          .map((node) =>
            node.options.length
              ? `${node.label}: ${node.options.map((option) => `${option.label ?? option.value}${option.description ? ` (${option.description})` : ""}${option.recommendation ? ` [Recommended: ${option.recommendation}]` : ""}`).join("; ")}`
              : node.label,
          )
          .join("\n")
          .slice(0, 8000);
        return {
          content: [
            {
              type: "text" as const,
              text: [
                form.title.trim(),
                outline,
                "Awaiting your response. Use the interactive response, or answer in chat if your client does not display it. No answer has been submitted yet.",
              ]
                .filter(Boolean)
                .join("\n"),
            },
          ],
          structuredContent: { status: "awaiting_input", formId, definition },
        };
      } catch (error) {
        return {
          isError: true,
          content: [
            {
              type: "text" as const,
              text: error instanceof Error ? error.message : "Invalid form",
            },
          ],
          structuredContent: {
            issues:
              error instanceof DefinitionError
                ? error.issues
                : [{ path: "$", message: "Invalid form" }],
          },
        };
      }
    },
  );
  registerAppResource(
    server,
    "Popup form",
    RESOURCE_URI,
    {
      mimeType: RESOURCE_MIME_TYPE,
      _meta: {
        ui: {
          csp: { connectDomains: [], resourceDomains: [] },
          prefersBorder: false,
        },
      },
    },
    async () => {
      const asset = await env.ASSETS.fetch(
        new Request("https://assets.local/index.html"),
      );
      if (!asset.ok) throw new Error("Form UI asset is unavailable");
      return {
        contents: [
          {
            uri: RESOURCE_URI,
            mimeType: RESOURCE_MIME_TYPE,
            text: await asset.text(),
            _meta: {
              ui: {
                csp: { connectDomains: [], resourceDomains: [] },
                prefersBorder: false,
              },
            },
          },
        ],
      };
    },
  );
  return server;
}
