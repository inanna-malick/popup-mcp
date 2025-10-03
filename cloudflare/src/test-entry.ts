// Test-specific entry point that avoids MCP server import (which has broken Node.js deps)
import { PopupSession } from './popup-session';

export { PopupSession };

// Real Worker fetch logic minus MCP endpoint
export default {
  async fetch(request: Request, env: Env, ctx: ExecutionContext): Promise<Response> {
    const url = new URL(request.url);

    // MCP SSE endpoint would go here but package is broken in Miniflare
    // @cloudflare/mcp-server-cloudflare imports node:child_process which doesn't exist in Workers
    if (url.pathname.startsWith('/sse')) {
      return new Response('MCP endpoint not testable in Miniflare (broken package dependency)', {
        status: 501
      });
    }

    // Route to Durable Object
    // Use a fixed ID for single instance (all clients connect to same DO)
    const id = env.POPUP_SESSION.idFromName('global');
    const stub = env.POPUP_SESSION.get(id);

    // Forward request to Durable Object
    if (url.pathname === '/connect' || url.pathname === '/show-popup') {
      return stub.fetch(request);
    }

    return new Response('Not found', { status: 404 });
  },
};
