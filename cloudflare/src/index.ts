import OAuthProvider from '@cloudflare/workers-oauth-provider';
import { Hono } from 'hono';
import { PopupSession } from './popup-session';
import { PopupMcpAgent } from './mcp-server';
import { HeaderAuthMcpAgent } from './mcp-server-header-auth';
import { GitHubHandler } from './auth';

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

// Top-level fetch handler - intercepts /header_auth before OAuthProvider
export default {
  async fetch(request: Request, env: Env, ctx: ExecutionContext): Promise<Response> {
    const url = new URL(request.url);

    // Handle /header_auth with bearer token auth (bypass OAuthProvider)
    if (url.pathname === '/header_auth') {
      // Inline bearer token validation
      if (!env.AUTH_TOKEN) {
        return new Response('AUTH_TOKEN not configured', { status: 500 });
      }

      const authHeader = request.headers.get('Authorization');
      if (!authHeader) {
        return new Response('Missing Authorization header', { status: 401 });
      }

      const match = authHeader.match(/^Bearer\s+(.+)$/i);
      if (!match) {
        return new Response('Invalid bearer token', { status: 401 });
      }

      const token = match[1];
      if (token !== env.AUTH_TOKEN) {
        return new Response('Invalid bearer token', { status: 401 });
      }

      // Auth valid - call HeaderAuthMcpAgent with custom binding
      return HeaderAuthMcpAgent.serve('/header_auth', { binding: 'HEADER_AUTH_MCP' }).fetch(request, env, ctx);
    }

    // Everything else goes to OAuthProvider
    return oauthProvider.fetch(request, env, ctx);
  },
};
