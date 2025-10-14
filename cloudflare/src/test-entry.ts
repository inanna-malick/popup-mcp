// Test-specific entry point that avoids MCP server import (which has broken Node.js deps)
import { PopupSession } from './popup-session';
import { validateBearerTokenRaw } from './auth-header';

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

    // Handle /popup - POST endpoint with bearer token auth
    if (url.pathname === '/popup' && request.method === 'POST') {
      // Validate bearer token
      const authError = validateBearerTokenRaw(request, env);
      if (authError) {
        return authError;
      }

      try {
        const bodyText = await request.text();
        const body = JSON.parse(bodyText);

        const id = env.POPUP_SESSION.idFromName('global');
        const stub = env.POPUP_SESSION.get(id);

        const response = await stub.fetch(new Request('http://internal/show-popup', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: bodyText,
        }));

        const resultText = await response.text();

        return new Response(resultText, {
          status: response.status,
          headers: { 'Content-Type': 'application/json' },
        });
      } catch (error) {
        console.error('[/popup] Error processing request:', error);
        return new Response(
          JSON.stringify({ status: 'error', message: String(error) }),
          { status: 500, headers: { 'Content-Type': 'application/json' } }
        );
      }
    }

    // Header auth endpoint - test the middleware logic
    if (url.pathname === '/header_auth') {
      // Validate bearer token
      const authError = validateBearerTokenRaw(request, env);
      if (authError) {
        return authError;
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
