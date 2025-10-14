import OAuthProvider from '@cloudflare/workers-oauth-provider';
import { Hono } from 'hono';
import { PopupSession } from './popup-session';
import { PopupMcpAgent } from './mcp-server';
import { HeaderAuthMcpAgent } from './mcp-server-header-auth';
import { GitHubHandler } from './auth';
import { validateBearerTokenRaw } from './auth-header';

export { PopupSession, PopupMcpAgent, HeaderAuthMcpAgent };

// Extend GitHubHandler with /connect and /show-popup routes
const app = new Hono();
app.route('/', GitHubHandler);

// Add PopupSession Durable Object routes
app.all('/connect', async (c) => {
  const env = c.env as Env;
  const id = env.POPUP_SESSION.idFromName('global');
  const stub = env.POPUP_SESSION.get(id);
  return stub.fetch(c.req.raw);
});

app.all('/show-popup', async (c) => {
  const env = c.env as Env;
  const id = env.POPUP_SESSION.idFromName('global');
  const stub = env.POPUP_SESSION.get(id);
  return stub.fetch(c.req.raw);
});

// Create OAuthProvider instance (not default export)
const oauthProvider = new OAuthProvider({
  apiHandlers: {
    '/sse': PopupMcpAgent.serveSSE('/sse'), // Deprecated SSE protocol
    '/mcp': PopupMcpAgent.serve('/mcp'), // Streamable HTTP protocol
  },
  authorizeEndpoint: '/authorize',
  clientRegistrationEndpoint: '/register',
  defaultHandler: app as any,
  tokenEndpoint: '/token',
});

// Top-level fetch handler - intercepts /connect and /header_auth before OAuthProvider
export default {
  async fetch(request: Request, env: Env, ctx: ExecutionContext): Promise<Response> {
    const url = new URL(request.url);

    // Handle /connect WebSocket upgrade (bypass OAuthProvider - not WebSocket-aware)
    if (url.pathname === '/connect') {
      const id = env.POPUP_SESSION.idFromName('global');
      const stub = env.POPUP_SESSION.get(id);
      return stub.fetch(request);
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

    // Handle /header_auth with bearer token auth (bypass OAuthProvider)
    if (url.pathname === '/header_auth') {
      // Validate bearer token
      const authError = validateBearerTokenRaw(request, env);
      if (authError) {
        return authError;
      }

      // Auth valid - call HeaderAuthMcpAgent with custom binding
      return HeaderAuthMcpAgent.serve('/header_auth', { binding: 'HEADER_AUTH_MCP' }).fetch(request, env, ctx);
    }

    // Everything else goes to OAuthProvider
    return oauthProvider.fetch(request, env, ctx);
  },
};
