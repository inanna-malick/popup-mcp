// Environment bindings for Cloudflare Worker

interface Env {
  // Durable Object binding
  POPUP_SESSION: DurableObjectNamespace;

  // Secret (set via: wrangler secret put POPUP_AUTH_SECRET)
  POPUP_AUTH_SECRET: string;
}
