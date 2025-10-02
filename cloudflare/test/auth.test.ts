import { describe, it, expect } from 'vitest';
import {
  createAuthenticatedRequest,
  createUnauthenticatedRequest,
  createWebSocketRequest,
  testWorkerFetch,
} from './helpers';

describe('Authentication', () => {
  describe('/connect endpoint', () => {
    it('accepts valid Bearer token for WebSocket upgrade', async () => {
      const request = createWebSocketRequest('http://localhost/connect', 'test-secret-token');
      const response = await testWorkerFetch(request);

      expect(response.status).toBe(101); // WebSocket upgrade

      // Clean up WebSocket connection
      if (response.webSocket) {
        response.webSocket.accept();
        response.webSocket.close(1000, 'Test complete');
        await new Promise((resolve) => setTimeout(resolve, 50));
      }
    });

    it('rejects WebSocket upgrade without Authorization header', async () => {
      const request = new Request('http://localhost/connect', {
        headers: {
          Upgrade: 'websocket',
        },
      });
      const response = await testWorkerFetch(request);

      expect(response.status).toBe(401);
      expect(await response.text()).toContain('Unauthorized');
    });

    it('rejects WebSocket upgrade with invalid token', async () => {
      const request = createWebSocketRequest('http://localhost/connect', 'wrong-token');
      const response = await testWorkerFetch(request);

      expect(response.status).toBe(401);
      expect(await response.text()).toContain('Invalid token');
    });

    it('rejects malformed Authorization header', async () => {
      const request = new Request('http://localhost/connect', {
        headers: {
          Authorization: 'InvalidFormat token123',
          Upgrade: 'websocket',
        },
      });
      const response = await testWorkerFetch(request);

      expect(response.status).toBe(401);
      expect(await response.text()).toContain('Unauthorized');
    });
  });

  describe('/show-popup endpoint', () => {
    it('accepts valid Bearer token for POST requests', async () => {
      const request = createAuthenticatedRequest(
        'http://localhost/show-popup',
        'POST',
        'test-secret-token',
        {
          definition: {
            title: 'Test',
            elements: [{ type: 'text', content: 'Hello' }],
          },
          timeout_ms: 5000,
        }
      );

      const response = await testWorkerFetch(request);

      // Should NOT be 401 - actual response depends on DO state
      expect(response.status).not.toBe(401);
    });

    it('rejects POST without Authorization header', async () => {
      const request = createUnauthenticatedRequest(
        'http://localhost/show-popup',
        'POST',
        {
          definition: {
            title: 'Test',
            elements: [{ type: 'text', content: 'Hello' }],
          },
          timeout_ms: 5000,
        }
      );

      const response = await testWorkerFetch(request);

      expect(response.status).toBe(401);
      expect(await response.text()).toContain('Unauthorized');
    });

    it('rejects POST with invalid token', async () => {
      const request = createAuthenticatedRequest(
        'http://localhost/show-popup',
        'POST',
        'wrong-token',
        {
          definition: {
            title: 'Test',
            elements: [{ type: 'text', content: 'Hello' }],
          },
          timeout_ms: 5000,
        }
      );

      const response = await testWorkerFetch(request);

      expect(response.status).toBe(401);
      expect(await response.text()).toContain('Invalid token');
    });
  });

  describe('/sse endpoint (MCP)', () => {
    it('does NOT require authentication', async () => {
      const request = createUnauthenticatedRequest('http://localhost/sse');
      const response = await testWorkerFetch(request);

      // Should NOT return 401 for MCP SSE endpoint
      expect(response.status).not.toBe(401);
    });

    it('works with authentication header present', async () => {
      const request = createAuthenticatedRequest('http://localhost/sse');
      const response = await testWorkerFetch(request);

      // Should still work even with auth header
      expect(response.status).not.toBe(401);
    });
  });

  describe('Unknown routes', () => {
    it('returns 404 for unknown routes with valid auth', async () => {
      const request = createAuthenticatedRequest('http://localhost/unknown');
      const response = await testWorkerFetch(request);

      expect(response.status).toBe(404);
    });

    it('returns 401 for unknown routes without auth', async () => {
      const request = createUnauthenticatedRequest('http://localhost/unknown');
      const response = await testWorkerFetch(request);

      // Auth check happens before route check
      expect(response.status).toBe(401);
    });
  });
});
