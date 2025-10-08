// Environment bindings for Cloudflare Worker

interface Env {
  // Durable Object bindings
  POPUP_SESSION: DurableObjectNamespace;
  MCP_OBJECT: DurableObjectNamespace;
  HEADER_AUTH_MCP: DurableObjectNamespace;

  // KV binding (for OAuth state)
  OAUTH_KV: KVNamespace;

  // Secrets (set via: wrangler secret put <NAME>)
  POPUP_AUTH_SECRET: string;
  AUTH_TOKEN: string; // Bearer token for header-based auth
  GITHUB_CLIENT_ID: string; // GitHub OAuth client ID
  GITHUB_CLIENT_SECRET: string; // GitHub OAuth client secret
  COOKIE_ENCRYPTION_KEY: string; // Cookie encryption key for OAuth
}
