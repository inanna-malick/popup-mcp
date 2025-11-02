import { McpAgent } from 'agents/mcp';
import { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import { z } from 'zod';
import type { PopupDefinition } from './protocol';

// Zod schema for V2 popup element types (element-as-key format)
const ElementSchema: z.ZodType<any> = z.lazy(() =>
  z.union([
    // Text element (id optional)
    z.object({
      text: z.string(),
      id: z.string().optional(),
      when: z.string().optional(),
    }),
    // Slider element (id required)
    z.object({
      slider: z.string(),
      id: z.string(),
      min: z.number(),
      max: z.number(),
      default: z.number().optional(),
      when: z.string().optional(),
      reveals: z.array(ElementSchema).optional(),
    }),
    // Checkbox element (id required)
    z.object({
      checkbox: z.string(),
      id: z.string(),
      default: z.boolean().optional(),
      when: z.string().optional(),
      reveals: z.array(ElementSchema).optional(),
    }),
    // Textbox element (id required)
    z.object({
      textbox: z.string(),
      id: z.string(),
      placeholder: z.string().optional(),
      rows: z.number().optional(),
      when: z.string().optional(),
    }),
    // Multiselect element (id required, option-as-key nesting via passthrough)
    z.object({
      multiselect: z.string(),
      id: z.string(),
      options: z.array(z.string()),
      when: z.string().optional(),
      reveals: z.array(ElementSchema).optional(),
    }).passthrough(),
    // Choice element (id required, option-as-key nesting via passthrough)
    z.object({
      choice: z.string(),
      id: z.string(),
      options: z.array(z.string()),
      default: z.number().optional(),
      when: z.string().optional(),
      reveals: z.array(ElementSchema).optional(),
    }).passthrough(),
    // Group element
    z.object({
      group: z.string(),
      elements: z.array(ElementSchema),
      when: z.string().optional(),
    }),
  ])
);

const PopupDefinitionSchema = z.object({
  title: z.string().optional(),
  elements: z.array(ElementSchema),
});

// Define our MCP agent with remote_popup tool (header auth version)
export class HeaderAuthMcpAgent extends McpAgent<Env, Record<string, never>, Record<string, never>> {
  server = new McpServer({
    name: 'Remote Popup Server',
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
