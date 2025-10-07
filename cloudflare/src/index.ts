import OAuthProvider from '@cloudflare/workers-oauth-provider';
import { Hono } from 'hono';
import { PopupSession } from './popup-session';
import { PopupMcpAgent } from './mcp-server';
import { HeaderAuthMcpAgent } from './mcp-server-header-auth';
import { GitHubHandler } from './auth';
import { validateBearerToken } from './auth-header';

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

// Add header auth MCP endpoint
app.all('/mcp/header_auth', validateBearerToken, async (c) => {
  return HeaderAuthMcpAgent.serve('/mcp/header_auth')(c.req.raw, c.env);
});

// Export OAuthProvider with multiple MCP endpoints
export default new OAuthProvider({
  apiHandlers: {
    '/sse': PopupMcpAgent.serveSSE('/sse'), // Deprecated SSE protocol
    '/mcp': PopupMcpAgent.serve('/mcp'), // Streamable HTTP protocol
  },
  authorizeEndpoint: '/authorize',
  clientRegistrationEndpoint: '/register',
  defaultHandler: app as any,
  tokenEndpoint: '/token',
});
