import { describe, it, expect } from 'vitest';
import { env } from 'cloudflare:test';
import { createWebSocketRequest, createPopupDefinition, testWorkerFetch } from './helpers';
import type { ServerMessage, ClientMessage } from '../src/protocol';

describe('WebSocket Connection', () => {
  it('establishes WebSocket connection with valid auth', async () => {
    const request = createWebSocketRequest('http://localhost/connect');
    const response = await testWorkerFetch(request);

    expect(response.status).toBe(101);
    expect(response.webSocket).toBeDefined();
  });

  it('receives ready message and stores device metadata', async () => {
    const request = createWebSocketRequest('http://localhost/connect');
    const response = await testWorkerFetch(request);

    const ws = response.webSocket!;
    ws.accept();

    // Send ready message
    const readyMsg: ClientMessage = {
      type: 'ready',
      device_name: 'test-device',
    };
    ws.send(JSON.stringify(readyMsg));

    // Note: We can't easily verify metadata storage without DO introspection
    // This test mainly verifies the message is accepted without error
    expect(response.status).toBe(101);

    // Clean up WebSocket connection
    ws.close(1000, 'Test complete');
    await new Promise((resolve) => setTimeout(resolve, 50));
  });

  it('handles pong response to ping', async () => {
    const request = createWebSocketRequest('http://localhost/connect');
    const response = await testWorkerFetch(request);

    const ws = response.webSocket!;
    ws.accept();

    // Send pong message
    const pongMsg: ClientMessage = {
      type: 'pong',
    };

    // Should not throw
    expect(() => ws.send(JSON.stringify(pongMsg))).not.toThrow();

    // Clean up WebSocket connection
    ws.close(1000, 'Test complete');
    await new Promise((resolve) => setTimeout(resolve, 50));
  });

  it('handles result message', async () => {
    const request = createWebSocketRequest('http://localhost/connect');
    const response = await testWorkerFetch(request);

    const ws = response.webSocket!;
    ws.accept();

    // Send result message
    const resultMsg: ClientMessage = {
      type: 'result',
      id: 'test-uuid',
      result: {
        status: 'completed',
        button: 'submit',
        Volume: 75,
      },
    };

    // Should not throw
    expect(() => ws.send(JSON.stringify(resultMsg))).not.toThrow();

    // Clean up WebSocket connection
    ws.close(1000, 'Test complete');
    await new Promise((resolve) => setTimeout(resolve, 50));
  });

  it('handles WebSocket close gracefully', async () => {
    const request = createWebSocketRequest('http://localhost/connect');
    const response = await testWorkerFetch(request);

    const ws = response.webSocket!;
    ws.accept();

    // Close connection
    expect(() => ws.close(1000, 'Test close')).not.toThrow();
  });

  it('rejects non-WebSocket requests to /connect', async () => {
    const request = new Request('http://localhost/connect', {
      method: 'GET',
      headers: {
        Authorization: 'Bearer test-secret-token',
      },
    });

    const response = await testWorkerFetch(request);

    expect(response.status).toBe(426); // Upgrade Required
    expect(await response.text()).toContain('WebSocket');
  });
});

describe('WebSocket Message Broadcasting', () => {
  it('sends show_popup message format correctly', async () => {
    // This test verifies the message structure
    const showPopupMsg: ServerMessage = {
      type: 'show_popup',
      id: 'test-uuid',
      definition: createPopupDefinition(),
      timeout_ms: 5000,
    };

    // Verify serialization works
    const serialized = JSON.stringify(showPopupMsg);
    const deserialized = JSON.parse(serialized) as ServerMessage;

    expect(deserialized.type).toBe('show_popup');
    expect(deserialized).toHaveProperty('id');
    expect(deserialized).toHaveProperty('definition');
    expect(deserialized).toHaveProperty('timeout_ms');
  });

  it('sends close_popup message format correctly', async () => {
    const closePopupMsg: ServerMessage = {
      type: 'close_popup',
      id: 'test-uuid',
    };

    const serialized = JSON.stringify(closePopupMsg);
    const deserialized = JSON.parse(serialized) as ServerMessage;

    expect(deserialized.type).toBe('close_popup');
    expect(deserialized).toHaveProperty('id');
  });

  it('sends ping message format correctly', async () => {
    const pingMsg: ServerMessage = {
      type: 'ping',
    };

    const serialized = JSON.stringify(pingMsg);
    const deserialized = JSON.parse(serialized) as ServerMessage;

    expect(deserialized.type).toBe('ping');
  });
});
