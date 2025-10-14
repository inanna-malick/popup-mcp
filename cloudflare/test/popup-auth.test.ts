import { describe, it, expect } from 'vitest';
import { testWorkerFetch } from './helpers';

describe('/popup endpoint authentication', () => {
  describe('Bearer Token Validation', () => {
    it('returns 401 when Authorization header is missing', async () => {
      const request = new Request('http://localhost/popup', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          definition: { title: 'Test', elements: [] },
          timeout_ms: 5000,
        }),
      });

      const response = await testWorkerFetch(request);
      const text = await response.text();

      expect(response.status).toBe(401);
      expect(text).toBe('Missing Authorization header');
    });

    it('returns 401 when bearer token is invalid', async () => {
      const request = new Request('http://localhost/popup', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': 'Bearer wrong-token',
        },
        body: JSON.stringify({
          definition: { title: 'Test', elements: [] },
          timeout_ms: 5000,
        }),
      });

      const response = await testWorkerFetch(request);
      const text = await response.text();

      expect(response.status).toBe(401);
      expect(text).toBe('Invalid bearer token');
    });

    it('returns 401 when Authorization header format is invalid', async () => {
      const request = new Request('http://localhost/popup', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': 'InvalidFormat',
        },
        body: JSON.stringify({
          definition: { title: 'Test', elements: [] },
          timeout_ms: 5000,
        }),
      });

      const response = await testWorkerFetch(request);
      const text = await response.text();

      expect(response.status).toBe(401);
      expect(text).toBe('Invalid bearer token');
    });

    it('returns 503 with valid token when no clients connected', async () => {
      const request = new Request('http://localhost/popup', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': 'Bearer test-token-12345',
        },
        body: JSON.stringify({
          definition: { title: 'Test', elements: [] },
          timeout_ms: 5000,
        }),
      });

      const response = await testWorkerFetch(request);
      const result = await response.json();

      expect(response.status).toBe(503);
      expect(result).toHaveProperty('status', 'error');
      expect(result).toHaveProperty('message', 'No clients connected');
    });
  });

  describe('Request Processing', () => {
    it('accepts valid bearer token and processes request', async () => {
      const request = new Request('http://localhost/popup', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': 'Bearer test-token-12345',
        },
        body: JSON.stringify({
          definition: {
            title: 'Test Popup',
            elements: [
              { type: 'text', content: 'Hello World' },
            ],
          },
          timeout_ms: 5000,
        }),
      });

      const response = await testWorkerFetch(request);

      // Should not reject auth (503 = no clients, not 401)
      expect(response.status).not.toBe(401);
      expect(response.headers.get('Content-Type')).toBe('application/json');
    });
  });
});
