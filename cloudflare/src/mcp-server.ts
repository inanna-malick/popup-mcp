import { McpAgent } from 'agents/mcp';
import { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import { z } from 'zod';
import type { PopupDefinition } from './protocol';
import type { Props } from './utils';

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

// Define our MCP agent with remote_popup tool
export class PopupMcpAgent extends McpAgent<Env, Record<string, never>, Props> {
  server = new McpServer({
    name: 'Remote Popup Server (GitHub OAuth)',
    version: '1.0.0',
  });

  async init() {
    // Define the remote_popup tool
    this.server.tool(
      'remote_popup',
      {
        definition: PopupDefinitionSchema,
        timeout_ms: z.number().optional().default(300000),
      },
      async ({ definition, timeout_ms }) => {
        const env = this.env as Env;

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
  }
}
