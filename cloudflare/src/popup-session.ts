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
    console.log('[DO] WebSocket upgrade requested');

    const upgradeHeader = request.headers.get('Upgrade');
    if (upgradeHeader !== 'websocket') {
      console.error('[DO] Invalid upgrade header:', upgradeHeader);
      return new Response('Expected WebSocket', { status: 426 });
    }

    const pair = new WebSocketPair();
    const [client, server] = Object.values(pair);

    console.log('[DO] Accepting WebSocket connection');
    // Accept WebSocket with hibernation support
    this.ctx.acceptWebSocket(server);

    console.log('[DO] Initializing session metadata');
    // Initialize session metadata
    this.sessions.set(server, {});
    console.log('[DO] Total connected clients:', this.sessions.size);

    return new Response(null, {
      status: 101,
      webSocket: client,
    });
  }

  private async handleShowPopup(request: Request): Promise<Response> {
    console.log('[DO] handleShowPopup called');

    try {
      const body = await request.json() as { definition: PopupDefinition; timeout_ms: number };
      console.log('[DO] Request body parsed - title:', body.definition?.title || '(no title)');
      console.log('[DO] Timeout:', body.timeout_ms, 'ms');

      // Validate required fields
      if (!body.definition) {
        console.error('[DO] Missing required field: definition');
        return new Response(
          JSON.stringify({ status: 'error', message: 'Missing required field: definition' }),
          { status: 400, headers: { 'Content-Type': 'application/json' } }
        );
      }

      // Check if any clients are connected
      console.log('[DO] Connected clients:', this.sessions.size);
      if (this.sessions.size === 0) {
        console.error('[DO] No clients connected');
        return new Response(
          JSON.stringify({ status: 'error', message: 'No clients connected' }),
          { status: 503, headers: { 'Content-Type': 'application/json' } }
        );
      }

      // Generate unique popup ID
      const popupId = uuidv4();
      console.log('[DO] Generated popup ID:', popupId);

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

      console.log('[DO] Broadcasting show_popup to', this.sessions.size, 'client(s)');
      this.broadcast(message);
      console.log('[DO] Broadcast complete');

      // Wait for result
      console.log('[DO] Waiting for result...');
      const result = await resultPromise;
      console.log('[DO] Result received:', JSON.stringify(result));

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
    console.log('[DO] webSocketMessage received');

    try {
      const data = typeof message === 'string' ? message : new TextDecoder().decode(message);
      console.log('[DO] Raw message:', data);

      const msg = JSON.parse(data) as ClientMessage;
      console.log('[DO] Parsed message type:', msg.type);

      switch (msg.type) {
        case 'ready':
          console.log('[DO] READY message - device:', msg.device_name || '(unnamed)');
          this.handleReady(ws, msg.device_name);
          break;

        case 'result':
          console.log('[DO] RESULT message - popup:', msg.id);
          console.log('[DO] Result data:', JSON.stringify(msg.result));
          this.handleResult(msg.id, msg.result);
          break;

        case 'pong':
          console.log('[DO] PONG message received');
          // Heartbeat response, no action needed
          break;

        default:
          console.warn('[DO] Unknown message type:', msg);
      }
    } catch (error) {
      console.error('[DO] Error processing WebSocket message:', error);
    }
  }

  async webSocketClose(ws: WebSocket, code: number, reason: string, wasClean: boolean) {
    // Remove from sessions
    this.sessions.delete(ws);
    ws.close(code, 'Client disconnected');
  }

  private handleReady(ws: WebSocket, deviceName?: string) {
    console.log('[DO] handleReady called - device:', deviceName || '(unnamed)');

    // Update session metadata
    const metadata: SessionMetadata = { deviceName };
    this.sessions.set(ws, metadata);
    console.log('[DO] Session metadata updated');

    // Persist metadata for hibernation
    ws.serializeAttachment(metadata);
    console.log('[DO] Metadata persisted for hibernation');
    console.log('[DO] Total sessions:', this.sessions.size);
  }

  private handleResult(popupId: string, result: PopupResult) {
    console.log('[DO] handleResult called for popup:', popupId);
    console.log('[DO] Result:', JSON.stringify(result));

    const pending = this.pendingPopups.get(popupId);
    if (!pending) {
      console.warn('[DO] Received result for unknown popup ID:', popupId);
      console.warn('[DO] Pending popups:', Array.from(this.pendingPopups.keys()));
      return;
    }

    console.log('[DO] Clearing timeout for popup:', popupId);
    // Clear timeout
    clearTimeout(pending.timeoutId);

    console.log('[DO] Resolving promise for popup:', popupId);
    // Resolve the promise
    pending.resolve(result);

    console.log('[DO] Cleaning up popup:', popupId);
    // Clean up
    this.pendingPopups.delete(popupId);

    console.log('[DO] Broadcasting close_popup');
    // Broadcast close_popup to all clients
    this.broadcastClosePopup(popupId);
  }

  private broadcast(message: ServerMessage) {
    const data = JSON.stringify(message);
    console.log('[DO] Broadcasting message type:', message.type);
    console.log('[DO] Message payload:', data);
    console.log('[DO] Number of clients:', this.sessions.size);

    let successCount = 0;
    let errorCount = 0;

    for (const ws of this.sessions.keys()) {
      try {
        ws.send(data);
        successCount++;
      } catch (error) {
        errorCount++;
        console.error('[DO] Error broadcasting to client:', error);
      }
    }

    console.log('[DO] Broadcast complete - success:', successCount, 'errors:', errorCount);
  }

  private broadcastClosePopup(popupId: string) {
    const message: ServerMessage = {
      type: 'close_popup',
      id: popupId
    };
    this.broadcast(message);
  }
}
