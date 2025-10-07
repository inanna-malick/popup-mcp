import type { Context, Next } from 'hono';

/**
 * Middleware to validate Bearer token authentication
 *
 * Validates that:
 * 1. AUTH_TOKEN secret is configured
 * 2. Authorization header is present
 * 3. Token matches configured secret
 */
export async function validateBearerToken(c: Context<{ Bindings: Env }>, next: Next) {
  const env = c.env as Env;

  // Check if AUTH_TOKEN is configured
  if (!env.AUTH_TOKEN) {
    return c.text('AUTH_TOKEN not configured', 500);
  }

  // Extract Authorization header
  const authHeader = c.req.header('Authorization');

  if (!authHeader) {
    return c.text('Missing Authorization header', 401);
  }

  // Parse Bearer token
  const match = authHeader.match(/^Bearer\s+(.+)$/i);

  if (!match) {
    return c.text('Invalid bearer token', 401);
  }

  const token = match[1];

  // Validate token
  if (token !== env.AUTH_TOKEN) {
    return c.text('Invalid bearer token', 401);
  }

  // Token is valid, continue
  await next();
}
