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

    // Header auth endpoint - test the middleware logic
    if (url.pathname === '/mcp/header_auth') {
      // Check if AUTH_TOKEN is configured
      if (!env.AUTH_TOKEN) {
        return new Response('AUTH_TOKEN not configured', { status: 500 });
      }

      // Extract Authorization header
      const authHeader = request.headers.get('Authorization');

      if (!authHeader) {
        return new Response('Missing Authorization header', { status: 401 });
      }

      // Parse Bearer token
      const match = authHeader.match(/^Bearer\s+(.+)$/i);

      if (!match) {
        return new Response('Invalid bearer token', { status: 401 });
      }

      const token = match[1];

      // Validate token
      if (token !== env.AUTH_TOKEN) {
        return new Response('Invalid bearer token', { status: 401 });
      }

      // Token is valid - MCP agent would handle here, but it's broken in tests
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
