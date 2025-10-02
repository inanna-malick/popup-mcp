import OAuthProvider from '@cloudflare/workers-oauth-provider';
import { Hono } from 'hono';
import { PopupSession } from './popup-session';
import { PopupMcpAgent } from './mcp-server';
import { GitHubHandler } from './auth';

export { PopupSession, PopupMcpAgent };

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
