import { env, SELF } from 'cloudflare:test';
import type { PopupDefinition, ClientMessage, ServerMessage } from '../src/protocol';

/**
 * Invoke the real Worker's fetch handler
 */
export async function testWorkerFetch(request: Request): Promise<Response> {
  return SELF.fetch(request);
}

/**
 * Create a WebSocket upgrade request
 */
export function createWebSocketRequest(url: string): Request {
  return new Request(url, {
    headers: {
      Upgrade: 'websocket',
      Connection: 'Upgrade',
      'Sec-WebSocket-Version': '13',
      'Sec-WebSocket-Key': 'test-key',
    },
  });
}

/**
 * Factory for creating test popup definitions (V2 format)
 */
export function createPopupDefinition(options?: {
  title?: string;
  includeSlider?: boolean;
  includeCheckbox?: boolean;
  includeTextbox?: boolean;
}): PopupDefinition {
  const elements: PopupDefinition['elements'] = [
    { text: 'Test popup content', id: 'test_text' },
  ];

  if (options?.includeSlider) {
    elements.push({
      slider: 'Volume',
      id: 'volume',
      min: 0,
      max: 100,
      default: 50,
    });
  }

  if (options?.includeCheckbox) {
    elements.push({
      checkbox: 'Enable notifications',
      id: 'enable_notifications',
      default: true,
    });
  }

  if (options?.includeTextbox) {
    elements.push({
      textbox: 'Name',
      id: 'name',
      placeholder: 'Enter your name',
    });
  }

  return {
    title: options?.title || 'Test Popup',
    elements,
  };
}

/**
 * Send a ClientMessage over WebSocket
 */
export async function sendClientMessage(
  ws: WebSocket,
  message: ClientMessage
): Promise<void> {
  ws.send(JSON.stringify(message));
}

/**
 * Wait for and parse a ServerMessage from WebSocket
 */
export async function waitForServerMessage(
  ws: WebSocket,
  timeout: number = 5000
): Promise<ServerMessage> {
  return new Promise((resolve, reject) => {
    const timeoutId = setTimeout(() => {
      reject(new Error(`Timeout waiting for server message after ${timeout}ms`));
    }, timeout);

    ws.addEventListener('message', (event) => {
      clearTimeout(timeoutId);
      try {
        const message = JSON.parse(event.data as string) as ServerMessage;
        resolve(message);
      } catch (e) {
        reject(new Error(`Failed to parse server message: ${e}`));
      }
    }, { once: true });
  });
}

/**
 * Create a ready message
 */
export function createReadyMessage(deviceName?: string): ClientMessage {
  return {
    type: 'ready',
    ...(deviceName && { device_name: deviceName }),
  };
}

/**
 * Create a result message
 */
export function createResultMessage(
  popupId: string,
  values: Record<string, unknown> = {},
  button: string = 'submit'
): ClientMessage {
  return {
    type: 'result',
    id: popupId,
    result: {
      status: 'completed',
      button,
      ...values,
    },
  };
}

/**
 * Create a cancelled result message
 */
export function createCancelledResultMessage(popupId: string): ClientMessage {
  return {
    type: 'result',
    id: popupId,
    result: {
      status: 'cancelled',
    },
  };
}

/**
 * Wait for response with timeout
 */
export async function waitForResponse<T>(
  promise: Promise<T>,
  timeout: number = 5000
): Promise<T> {
  return Promise.race([
    promise,
    new Promise<T>((_, reject) =>
      setTimeout(() => reject(new Error(`Timeout after ${timeout}ms`)), timeout)
    ),
  ]);
}
