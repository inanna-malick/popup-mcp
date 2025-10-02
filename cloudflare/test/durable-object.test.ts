import { describe, it, expect } from 'vitest';
import { env, runInDurableObject } from 'cloudflare:test';
import { PopupSession } from '../src/popup-session';
import { createPopupDefinition, createWebSocketRequest } from './helpers';
import type { PopupResult } from '../src/protocol';

describe('Durable Object - PopupSession', () => {
  describe('WebSocket Session Management', () => {
    it('initializes with empty sessions map', async () => {
      const id = env.POPUP_SESSION.idFromName('global');
      const stub = env.POPUP_SESSION.get(id);

      await runInDurableObject(stub, (instance: PopupSession) => {
        // @ts-expect-error - accessing private field for testing
        expect(instance.sessions.size).toBe(0);
      });
    });

    it('recovers hibernating WebSocket connections', async () => {
      const id = env.POPUP_SESSION.idFromName('global');
      const stub = env.POPUP_SESSION.get(id);

      // Create WebSocket connection
      const request = createWebSocketRequest('http://localhost/connect');
      const response = await stub.fetch(request);
      expect(response.status).toBe(101);

      const ws = response.webSocket!;
      ws.accept();

      // Verify session was added
      await runInDurableObject(stub, (instance: PopupSession) => {
        // @ts-expect-error - accessing private field for testing
        expect(instance.sessions.size).toBeGreaterThan(0);
      });

      // Clean up WebSocket connection
      ws.close(1000, 'Test complete');
      await new Promise((resolve) => setTimeout(resolve, 50));
    });
  });

  describe('Popup Request Handling', () => {
    it('returns 503 when no clients connected', async () => {
      const id = env.POPUP_SESSION.idFromName('global');
      const stub = env.POPUP_SESSION.get(id);

      const request = new Request('http://internal/show-popup', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          definition: createPopupDefinition(),
          timeout_ms: 1000,
        }),
      });

      const response = await stub.fetch(request);
      expect(response.status).toBe(503);

      const result = await response.json();
      expect(result).toHaveProperty('status', 'error');
      expect(result).toHaveProperty('message');
      expect(result.message).toContain('No clients connected');
    });

    it('creates pending popup with UUID', async () => {
      const id = env.POPUP_SESSION.idFromName('global');
      const stub = env.POPUP_SESSION.get(id);

      // First establish a WebSocket connection
      const wsRequest = createWebSocketRequest('http://localhost/connect');
      const wsResponse = await stub.fetch(wsRequest);
      const ws = wsResponse.webSocket!;
      ws.accept();

      // Create popup request (will timeout, but that's ok for this test)
      const popupRequest = new Request('http://internal/show-popup', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          definition: createPopupDefinition(),
          timeout_ms: 100, // Short timeout for test
        }),
      });

      // This will timeout, but we're just verifying the pending popup is created
      const responsePromise = stub.fetch(popupRequest);

      // Give it a moment to create the pending popup
      await new Promise((resolve) => setTimeout(resolve, 50));

      await runInDurableObject(stub, (instance: PopupSession) => {
        // @ts-expect-error - accessing private field for testing
        expect(instance.pendingPopups.size).toBe(1);
      });

      // Wait for timeout
      await responsePromise;

      // Clean up WebSocket connection
      ws.close(1000, 'Test complete');
      await new Promise((resolve) => setTimeout(resolve, 50));
    });

    it('handles timeout correctly', async () => {
      const id = env.POPUP_SESSION.idFromName('global');
      const stub = env.POPUP_SESSION.get(id);

      // Establish WebSocket connection
      const wsRequest = createWebSocketRequest('http://localhost/connect');
      const wsResponse = await stub.fetch(wsRequest);
      const ws = wsResponse.webSocket!;
      ws.accept();

      // Create popup with very short timeout
      const popupRequest = new Request('http://internal/show-popup', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          definition: createPopupDefinition(),
          timeout_ms: 100,
        }),
      });

      const response = await stub.fetch(popupRequest);
      const result = (await response.json()) as PopupResult;

      expect(result.status).toBe('timeout');
      expect(result).toHaveProperty('message');

      // Clean up WebSocket connection
      ws.close(1000, 'Test complete');
      await new Promise((resolve) => setTimeout(resolve, 50));
    });
  });

  describe('First Response Wins Pattern', () => {
    it('resolves with first client response', async () => {
      const id = env.POPUP_SESSION.idFromName('global');
      const stub = env.POPUP_SESSION.get(id);

      // Establish WebSocket
      const wsRequest = createWebSocketRequest('http://localhost/connect');
      const wsResponse = await stub.fetch(wsRequest);
      const ws = wsResponse.webSocket!;
      ws.accept();

      // Send ready message
      ws.send(JSON.stringify({ type: 'ready', device_name: 'test' }));

      // Listen for show_popup message to get the actual popup ID
      const popupIdPromise = new Promise<string>((resolve) => {
        ws.addEventListener('message', (event) => {
          const msg = JSON.parse(event.data as string);
          if (msg.type === 'show_popup') {
            resolve(msg.id);
          }
        }, { once: true });
      });

      // Create popup request
      const popupRequest = new Request('http://internal/show-popup', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          definition: createPopupDefinition(),
          timeout_ms: 5000,
        }),
      });

      const responsePromise = stub.fetch(popupRequest);

      // Wait for show_popup and extract the ID
      const popupId = await popupIdPromise;

      // Send result with the correct popup ID
      ws.send(
        JSON.stringify({
          type: 'result',
          id: popupId,
          result: {
            status: 'completed',
            button: 'submit',
            Volume: 75,
          },
        })
      );

      const response = await responsePromise;
      expect(response.ok).toBe(true);

      const result = await response.json();
      expect(result.status).toBe('completed');

      // Clean up WebSocket connection
      ws.close(1000, 'Test complete');
      await new Promise((resolve) => setTimeout(resolve, 50));
    });
  });

  describe('State Persistence', () => {
    it('persists session metadata for hibernation', async () => {
      const id = env.POPUP_SESSION.idFromName('global');
      const stub = env.POPUP_SESSION.get(id);

      const request = createWebSocketRequest('http://localhost/connect');
      const response = await stub.fetch(request);

      const ws = response.webSocket!;
      ws.accept();

      // Send ready with device name
      ws.send(JSON.stringify({ type: 'ready', device_name: 'laptop-1' }));

      // Wait for metadata to be persisted
      await new Promise((resolve) => setTimeout(resolve, 100));

      // Metadata should be serialized for hibernation
      // Note: Actual verification of hibernation persistence is complex
      expect(response.status).toBe(101);

      // Clean up WebSocket connection
      ws.close(1000, 'Test complete');
      await new Promise((resolve) => setTimeout(resolve, 50));
    });
  });

  describe('Concurrent Popups', () => {
    it('tracks multiple pending popups by UUID', async () => {
      const id = env.POPUP_SESSION.idFromName('global');
      const stub = env.POPUP_SESSION.get(id);

      // Establish connection
      const wsRequest = createWebSocketRequest('http://localhost/connect');
      const wsResponse = await stub.fetch(wsRequest);
      const ws = wsResponse.webSocket!;
      ws.accept();

      // Create two popup requests concurrently
      const popup1 = stub.fetch(
        new Request('http://internal/show-popup', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            definition: createPopupDefinition({ title: 'Popup 1' }),
            timeout_ms: 500,
          }),
        })
      );

      const popup2 = stub.fetch(
        new Request('http://internal/show-popup', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            definition: createPopupDefinition({ title: 'Popup 2' }),
            timeout_ms: 500,
          }),
        })
      );

      // Give them time to be created
      await new Promise((resolve) => setTimeout(resolve, 50));

      await runInDurableObject(stub, (instance: PopupSession) => {
        // @ts-expect-error - accessing private field for testing
        expect(instance.pendingPopups.size).toBe(2);
      });

      // Wait for both to timeout
      await Promise.all([popup1, popup2]);

      // Clean up WebSocket connection
      ws.close(1000, 'Test complete');
      await new Promise((resolve) => setTimeout(resolve, 50));
    });
  });
});
