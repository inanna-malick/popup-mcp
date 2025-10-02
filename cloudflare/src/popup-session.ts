import { DurableObject } from 'cloudflare:workers';
import { v4 as uuidv4 } from 'uuid';
import type { ClientMessage, ServerMessage, PopupDefinition, PopupResult } from './protocol';

interface SessionMetadata {
  deviceName?: string;
}

interface PendingPopup {
  resolve: (result: PopupResult) => void;
  reject: (error: Error) => void;
  timeoutId: number;
}

export class PopupSession extends DurableObject {
  private sessions: Map<WebSocket, SessionMetadata> = new Map();
  private pendingPopups: Map<string, PendingPopup> = new Map();

  constructor(ctx: DurableObjectState, env: Env) {
    super(ctx, env);

    // Recover hibernating WebSocket connections
    this.ctx.getWebSockets().forEach((ws) => {
      const metadata = ws.deserializeAttachment() as SessionMetadata | null;
      this.sessions.set(ws, metadata || {});
    });
  }

  async fetch(request: Request): Promise<Response> {
    const url = new URL(request.url);

    // WebSocket connection endpoint
    if (url.pathname === '/connect') {
      return this.handleWebSocketUpgrade(request);
    }

    // HTTP endpoint for creating popup requests
    if (url.pathname === '/show-popup' && request.method === 'POST') {
      return this.handleShowPopup(request);
    }

    return new Response('Not found', { status: 404 });
  }

  private async handleWebSocketUpgrade(request: Request): Promise<Response> {
    const upgradeHeader = request.headers.get('Upgrade');
    if (upgradeHeader !== 'websocket') {
      return new Response('Expected WebSocket', { status: 426 });
    }

    const pair = new WebSocketPair();
    const [client, server] = Object.values(pair);

    // Accept WebSocket with hibernation support
    this.ctx.acceptWebSocket(server);

    // Initialize session metadata
    this.sessions.set(server, {});

    return new Response(null, {
      status: 101,
      webSocket: client,
    });
  }

  private async handleShowPopup(request: Request): Promise<Response> {
    try {
      const body = await request.json() as { definition: PopupDefinition; timeout_ms: number };

      // Check if any clients are connected
      if (this.sessions.size === 0) {
        return new Response(
          JSON.stringify({ status: 'error', message: 'No clients connected' }),
          { status: 503, headers: { 'Content-Type': 'application/json' } }
        );
      }

      // Generate unique popup ID
      const popupId = uuidv4();

      // Create promise that resolves when first client responds
      const resultPromise = new Promise<PopupResult>((resolve, reject) => {
        // Set up timeout
        const timeoutId = setTimeout(() => {
          this.pendingPopups.delete(popupId);
          // Broadcast close_popup to all clients
          this.broadcastClosePopup(popupId);
          resolve({
            status: 'timeout',
            message: `No response received within ${body.timeout_ms}ms`
          });
        }, body.timeout_ms) as unknown as number;

        this.pendingPopups.set(popupId, { resolve, reject, timeoutId });
      });

      // Broadcast show_popup to all connected clients
      const message: ServerMessage = {
        type: 'show_popup',
        id: popupId,
        definition: body.definition,
        timeout_ms: body.timeout_ms
      };

      this.broadcast(message);

      // Wait for result
      const result = await resultPromise;

      return new Response(JSON.stringify(result), {
        headers: { 'Content-Type': 'application/json' }
      });

    } catch (error) {
      return new Response(
        JSON.stringify({ status: 'error', message: String(error) }),
        { status: 400, headers: { 'Content-Type': 'application/json' } }
      );
    }
  }

  async webSocketMessage(ws: WebSocket, message: ArrayBuffer | string) {
    try {
      const data = typeof message === 'string' ? message : new TextDecoder().decode(message);
      const msg = JSON.parse(data) as ClientMessage;

      switch (msg.type) {
        case 'ready':
          this.handleReady(ws, msg.device_name);
          break;

        case 'result':
          this.handleResult(msg.id, msg.result);
          break;

        case 'pong':
          // Heartbeat response, no action needed
          break;

        default:
          console.warn('Unknown message type:', msg);
      }
    } catch (error) {
      console.error('Error processing WebSocket message:', error);
    }
  }

  async webSocketClose(ws: WebSocket, code: number, reason: string, wasClean: boolean) {
    // Remove from sessions
    this.sessions.delete(ws);
    ws.close(code, 'Client disconnected');
  }

  private handleReady(ws: WebSocket, deviceName?: string) {
    // Update session metadata
    const metadata: SessionMetadata = { deviceName };
    this.sessions.set(ws, metadata);

    // Persist metadata for hibernation
    ws.serializeAttachment(metadata);
  }

  private handleResult(popupId: string, result: PopupResult) {
    const pending = this.pendingPopups.get(popupId);
    if (!pending) {
      console.warn('Received result for unknown popup ID:', popupId);
      return;
    }

    // Clear timeout
    clearTimeout(pending.timeoutId);

    // Resolve the promise
    pending.resolve(result);

    // Clean up
    this.pendingPopups.delete(popupId);

    // Broadcast close_popup to all clients
    this.broadcastClosePopup(popupId);
  }

  private broadcast(message: ServerMessage) {
    const data = JSON.stringify(message);
    for (const ws of this.sessions.keys()) {
      try {
        ws.send(data);
      } catch (error) {
        console.error('Error broadcasting to client:', error);
      }
    }
  }

  private broadcastClosePopup(popupId: string) {
    const message: ServerMessage = {
      type: 'close_popup',
      id: popupId
    };
    this.broadcast(message);
  }
}
