import { describe, it, expect } from 'vitest';
import { env } from 'cloudflare:test';
import {
  createAuthenticatedRequest,
  createWebSocketRequest,
  createPopupDefinition,
  createResultMessage,
  createCancelledResultMessage,
  testWorkerFetch,
} from './helpers';
import type { ServerMessage, PopupResult } from '../src/protocol';

describe('End-to-End Integration', () => {
  describe('Full Popup Flow', () => {
    it('completes full flow: HTTP request → WebSocket → Result', async () => {
      const id = env.POPUP_SESSION.idFromName('global');
      const stub = env.POPUP_SESSION.get(id);

      // Step 1: Establish WebSocket connection
      const wsRequest = createWebSocketRequest('http://localhost/connect');
      const wsResponse = await stub.fetch(wsRequest);
      expect(wsResponse.status).toBe(101);

      const ws = wsResponse.webSocket!;
      ws.accept();

      // Send ready message
      ws.send(JSON.stringify({ type: 'ready', device_name: 'test-client' }));

      // Step 2: Create popup via HTTP
      const popupDefinition = createPopupDefinition({
        title: 'Integration Test',
        includeSlider: true,
      });

      const httpRequest = new Request('http://internal/show-popup', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          definition: popupDefinition,
          timeout_ms: 5000,
        }),
      });

      const responsePromise = stub.fetch(httpRequest);

      // Step 3: Receive show_popup message
      const messagePromise = new Promise<ServerMessage>((resolve) => {
        ws.addEventListener('message', (event) => {
          const msg = JSON.parse(event.data as string) as ServerMessage;
          if (msg.type === 'show_popup') {
            resolve(msg);
          }
        });
      });

      const showPopupMsg = await messagePromise;
      expect(showPopupMsg.type).toBe('show_popup');
      expect(showPopupMsg).toHaveProperty('id');
      expect(showPopupMsg).toHaveProperty('definition');
      expect(showPopupMsg).toHaveProperty('timeout_ms');

      // Step 4: Send result
      const resultMsg = createResultMessage(showPopupMsg.id, {
        Volume: '50/100',
      });
      ws.send(JSON.stringify(resultMsg));

      // Step 5: Verify HTTP response
      const response = await responsePromise;
      expect(response.ok).toBe(true);

      const result = (await response.json()) as PopupResult;
      expect(result.status).toBe('completed');
      expect(result).toHaveProperty('button', 'submit');

      // Clean up WebSocket connection
      ws.close(1000, 'Test complete');
      await new Promise((resolve) => setTimeout(resolve, 50));
    });

    it('handles cancelled popup correctly', async () => {
      const id = env.POPUP_SESSION.idFromName('global');
      const stub = env.POPUP_SESSION.get(id);

      // Establish connection
      const wsRequest = createWebSocketRequest('http://localhost/connect');
      const wsResponse = await stub.fetch(wsRequest);
      const ws = wsResponse.webSocket!;
      ws.accept();

      ws.send(JSON.stringify({ type: 'ready' }));

      // Create popup
      const httpRequest = new Request('http://internal/show-popup', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          definition: createPopupDefinition(),
          timeout_ms: 5000,
        }),
      });

      const responsePromise = stub.fetch(httpRequest);

      // Wait for show_popup
      const msg = await new Promise<ServerMessage>((resolve) => {
        ws.addEventListener(
          'message',
          (event) => {
            resolve(JSON.parse(event.data as string) as ServerMessage);
          },
          { once: true }
        );
      });

      // Send cancelled result
      if (msg.type === 'show_popup') {
        const cancelMsg = createCancelledResultMessage(msg.id);
        ws.send(JSON.stringify(cancelMsg));
      }

      const response = await responsePromise;
      const result = (await response.json()) as PopupResult;

      expect(result.status).toBe('cancelled');

      // Clean up WebSocket connection
      ws.close(1000, 'Test complete');
      await new Promise((resolve) => setTimeout(resolve, 50));
    });
  });

  describe('Timeout Scenarios', () => {
    it('returns timeout result when no response received', async () => {
      const id = env.POPUP_SESSION.idFromName('global');
      const stub = env.POPUP_SESSION.get(id);

      // Establish connection but don't send result
      const wsRequest = createWebSocketRequest('http://localhost/connect');
      const wsResponse = await stub.fetch(wsRequest);
      const ws = wsResponse.webSocket!;
      ws.accept();

      const httpRequest = new Request('http://internal/show-popup', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          definition: createPopupDefinition(),
          timeout_ms: 100, // Very short timeout
        }),
      });

      const response = await stub.fetch(httpRequest);
      const result = (await response.json()) as PopupResult;

      expect(result.status).toBe('timeout');
      expect(result).toHaveProperty('message');

      // Clean up WebSocket connection
      ws.close(1000, 'Test complete');
      await new Promise((resolve) => setTimeout(resolve, 50));
    });

    it('broadcasts close_popup on timeout', async () => {
      const id = env.POPUP_SESSION.idFromName('global');
      const stub = env.POPUP_SESSION.get(id);

      const wsRequest = createWebSocketRequest('http://localhost/connect');
      const wsResponse = await stub.fetch(wsRequest);
      const ws = wsResponse.webSocket!;
      ws.accept();

      const messages: ServerMessage[] = [];
      ws.addEventListener('message', (event) => {
        messages.push(JSON.parse(event.data as string) as ServerMessage);
      });

      const httpRequest = new Request('http://internal/show-popup', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          definition: createPopupDefinition(),
          timeout_ms: 100,
        }),
      });

      await stub.fetch(httpRequest);

      // Wait a bit for close_popup message
      await new Promise((resolve) => setTimeout(resolve, 150));

      // Should have received both show_popup and close_popup
      expect(messages.length).toBeGreaterThanOrEqual(1);
      const hasClosePopup = messages.some((m) => m.type === 'close_popup');
      expect(hasClosePopup).toBe(true);

      // Clean up WebSocket connection
      ws.close(1000, 'Test complete');
      await new Promise((resolve) => setTimeout(resolve, 50));
    });
  });

  describe('Multiple Clients', () => {
    it('broadcasts show_popup to all connected clients', async () => {
      const id = env.POPUP_SESSION.idFromName('global');
      const stub = env.POPUP_SESSION.get(id);

      // Connect two clients
      const ws1Response = await stub.fetch(
        createWebSocketRequest('http://localhost/connect')
      );
      const ws1 = ws1Response.webSocket!;
      ws1.accept();

      const ws2Response = await stub.fetch(
        createWebSocketRequest('http://localhost/connect')
      );
      const ws2 = ws2Response.webSocket!;
      ws2.accept();

      const client1Messages: ServerMessage[] = [];
      const client2Messages: ServerMessage[] = [];

      ws1.addEventListener('message', (event) => {
        client1Messages.push(JSON.parse(event.data as string) as ServerMessage);
      });

      ws2.addEventListener('message', (event) => {
        client2Messages.push(JSON.parse(event.data as string) as ServerMessage);
      });

      // Create popup
      const httpRequest = new Request('http://internal/show-popup', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          definition: createPopupDefinition(),
          timeout_ms: 200,
        }),
      });

      await stub.fetch(httpRequest);

      // Wait for messages
      await new Promise((resolve) => setTimeout(resolve, 100));

      // Both clients should have received show_popup
      expect(client1Messages.some((m) => m.type === 'show_popup')).toBe(true);
      expect(client2Messages.some((m) => m.type === 'show_popup')).toBe(true);

      // Clean up WebSocket connections
      ws1.close(1000, 'Test complete');
      ws2.close(1000, 'Test complete');
      await new Promise((resolve) => setTimeout(resolve, 50));
    });

    it('first response wins, others receive close_popup', async () => {
      const id = env.POPUP_SESSION.idFromName('global');
      const stub = env.POPUP_SESSION.get(id);

      // Connect client
      const wsResponse = await stub.fetch(
        createWebSocketRequest('http://localhost/connect')
      );
      const ws = wsResponse.webSocket!;
      ws.accept();

      ws.send(JSON.stringify({ type: 'ready' }));

      const messages: ServerMessage[] = [];
      ws.addEventListener('message', (event) => {
        messages.push(JSON.parse(event.data as string) as ServerMessage);
      });

      // Create popup
      const httpRequest = new Request('http://internal/show-popup', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          definition: createPopupDefinition(),
          timeout_ms: 5000,
        }),
      });

      const responsePromise = stub.fetch(httpRequest);

      // Wait for show_popup
      await new Promise((resolve) => setTimeout(resolve, 50));

      const showPopup = messages.find((m) => m.type === 'show_popup');
      expect(showPopup).toBeDefined();

      if (showPopup && showPopup.type === 'show_popup') {
        // Send result
        ws.send(
          JSON.stringify({
            type: 'result',
            id: showPopup.id,
            result: { status: 'completed', button: 'submit' },
          })
        );
      }

      await responsePromise;

      // Wait for close_popup
      await new Promise((resolve) => setTimeout(resolve, 50));

      // Should receive close_popup after result
      const closePopup = messages.find((m) => m.type === 'close_popup');
      expect(closePopup).toBeDefined();

      // Clean up WebSocket connection
      ws.close(1000, 'Test complete');
      await new Promise((resolve) => setTimeout(resolve, 50));
    });
  });

  describe('Error Handling', () => {
    it('handles invalid JSON in popup request', async () => {
      const id = env.POPUP_SESSION.idFromName('global');
      const stub = env.POPUP_SESSION.get(id);

      const httpRequest = new Request('http://internal/show-popup', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: 'invalid json',
      });

      const response = await stub.fetch(httpRequest);
      expect(response.status).toBe(400);
    });

    it('handles missing definition in request', async () => {
      const id = env.POPUP_SESSION.idFromName('global');
      const stub = env.POPUP_SESSION.get(id);

      const httpRequest = new Request('http://internal/show-popup', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          timeout_ms: 5000,
          // definition missing
        }),
      });

      const response = await stub.fetch(httpRequest);
      expect(response.status).toBe(400);
    });
  });

  describe('Worker Routing', () => {
    it('routes authenticated /show-popup through Worker', async () => {
      const request = createAuthenticatedRequest(
        'http://localhost/show-popup',
        'POST',
        'test-secret-token',
        {
          definition: createPopupDefinition(),
          timeout_ms: 100,
        }
      );

      const response = await testWorkerFetch(request);

      // Should be routed to DO (will timeout due to no clients, but that's ok)
      expect(response.status).not.toBe(401);
      expect(response.status).not.toBe(404);
    });

    it('routes authenticated /connect through Worker', async () => {
      const request = createWebSocketRequest(
        'http://localhost/connect',
        'test-secret-token'
      );

      const response = await testWorkerFetch(request);

      expect(response.status).toBe(101);
    });
  });
});
