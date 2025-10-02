import { PopupSession } from './popup-session';
import { PopupMCP } from './mcp-server';

export { PopupSession };

export default {
  async fetch(request: Request, env: Env, ctx: ExecutionContext): Promise<Response> {
    const url = new URL(request.url);

    // MCP SSE endpoint (no auth required - used by MCP clients like Claude Desktop)
    if (url.pathname.startsWith('/sse')) {
      return PopupMCP.serveSSE('/sse').fetch(request, env, ctx);
    }

    // Auth required for WebSocket and direct HTTP endpoints
    const authHeader = request.headers.get('Authorization');
    if (!authHeader || !authHeader.startsWith('Bearer ')) {
      return new Response('Unauthorized: Missing or invalid Authorization header', { status: 401 });
    }

    const token = authHeader.substring(7); // Remove 'Bearer ' prefix
    if (token !== env.POPUP_AUTH_SECRET) {
      return new Response('Unauthorized: Invalid token', { status: 401 });
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
