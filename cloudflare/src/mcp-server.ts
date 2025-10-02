import { McpServer } from '@cloudflare/mcp-server-cloudflare';
import { z } from 'zod';
import type { PopupDefinition } from './protocol';

// Zod schema for popup element types
const ElementSchema: z.ZodType<any> = z.lazy(() =>
  z.union([
    z.object({ type: z.literal('text'), content: z.string() }),
    z.object({
      type: z.literal('slider'),
      label: z.string(),
      min: z.number(),
      max: z.number(),
      default: z.number().optional(),
    }),
    z.object({
      type: z.literal('checkbox'),
      label: z.string(),
      default: z.boolean().optional(),
    }),
    z.object({
      type: z.literal('textbox'),
      label: z.string(),
      placeholder: z.string().optional(),
      rows: z.number().optional(),
    }),
    z.object({
      type: z.literal('multiselect'),
      label: z.string(),
      options: z.array(z.string()),
    }),
    z.object({
      type: z.literal('choice'),
      label: z.string(),
      options: z.array(z.string()),
      default: z.number().optional(),
    }),
    z.object({
      type: z.literal('group'),
      label: z.string(),
      elements: z.array(ElementSchema),
    }),
    z.object({
      type: z.literal('conditional'),
      condition: z.union([
        z.string(),
        z.object({ field: z.string(), value: z.string() }),
        z.object({ field: z.string(), count: z.string() }),
      ]),
      elements: z.array(ElementSchema),
    }),
  ])
);

const PopupDefinitionSchema = z.object({
  title: z.string().optional(),
  elements: z.array(ElementSchema),
});

const RemotePopupInputSchema = z.object({
  definition: PopupDefinitionSchema,
  timeout_ms: z.number().optional().default(30000),
});

export class PopupMcpServer {
  private server: McpServer;

  constructor() {
    this.server = new McpServer({
      name: 'Remote Popup Server',
      version: '1.0.0',
    });

    this.init();
  }

  private init() {
    // Define the remote_popup tool
    this.server.tool(
      'remote_popup',
      RemotePopupInputSchema,
      async (input, extra) => {
        const { definition, timeout_ms } = input;
        const env = extra.env as Env;

        try {
          // Forward to Durable Object
          const id = env.POPUP_SESSION.idFromName('global');
          const stub = env.POPUP_SESSION.get(id);

          // Create request to DO's show-popup endpoint
          const doRequest = new Request('http://internal/show-popup', {
            method: 'POST',
            headers: {
              'Content-Type': 'application/json',
            },
            body: JSON.stringify({ definition, timeout_ms }),
          });

          // Call DO and wait for result
          const response = await stub.fetch(doRequest);
          const result = await response.json();

          if (!response.ok) {
            return {
              content: [
                {
                  type: 'text',
                  text: `Error: ${JSON.stringify(result)}`,
                },
              ],
              isError: true,
            };
          }

          // Return successful result
          return {
            content: [
              {
                type: 'text',
                text: JSON.stringify(result, null, 2),
              },
            ],
          };
        } catch (error) {
          return {
            content: [
              {
                type: 'text',
                text: `Error creating popup: ${error}`,
              },
            ],
            isError: true,
          };
        }
      }
    );

    // Add tool description
    this.server.setToolDescription(
      'remote_popup',
      'Create a native GUI popup on connected client devices. Supports text, sliders, checkboxes, text inputs, multi-select, and more. First connected client to respond wins. Returns user interaction result.'
    );
  }

  // Serve SSE endpoint
  serveSSE(basePath: string) {
    return this.server.serveSSE(basePath);
  }
}

// Create singleton instance
export const PopupMCP = new PopupMcpServer();
