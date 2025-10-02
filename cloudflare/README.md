# Popup Relay - Cloudflare Worker + Durable Object

WebSocket relay server for remote popup GUI functionality using Cloudflare Workers and Durable Objects.

## Architecture

- **Worker** (`src/index.ts`): Entry point, auth validation, routing to Durable Object
- **Durable Object** (`src/popup-session.ts`): WebSocket server with hibernation, manages client sessions and popup requests
- **Protocol** (`src/protocol.ts`): TypeScript types matching Rust definitions

## Setup

### 1. Install Dependencies

```bash
cd cloudflare
npm install
```

### 2. Configure Secrets

For local development:

```bash
cp .dev.vars.example .dev.vars
# Edit .dev.vars and set POPUP_AUTH_SECRET
```

For production:

```bash
wrangler secret put POPUP_AUTH_SECRET
# Enter your secret when prompted
```

### 3. Local Development

```bash
npm run dev
```

This starts the Worker at `http://localhost:8787`.

### 4. Deploy to Production

```bash
npm run deploy
```

## API Endpoints

### MCP SSE Endpoint: `GET /sse`

MCP (Model Context Protocol) server for AI assistants to create popups via natural language.

**No authentication required** - designed for use with MCP clients like Claude Desktop.

**Available Tool:**
- `remote_popup`: Create a native GUI popup on connected client devices

**Tool Input Schema:**
```json
{
  "definition": {
    "title": "Popup Title",
    "elements": [
      {"type": "text", "content": "Hello world"},
      {"type": "slider", "label": "Volume", "min": 0, "max": 100},
      {"type": "checkbox", "label": "Enable feature"},
      ...
    ]
  },
  "timeout_ms": 30000  // optional, defaults to 30000
}
```

**Tool Output:**
Returns the popup result as JSON (first connected client to respond wins).

#### Claude Desktop Configuration

Add to your Claude Desktop config (`~/Library/Application Support/Claude/claude_desktop_config.json` on macOS):

```json
{
  "mcpServers": {
    "popup-relay": {
      "url": "https://your-worker.workers.dev/sse"
    }
  }
}
```

Then restart Claude Desktop. You'll see a 🔨 icon with the `remote_popup` tool.

### WebSocket Connection: `GET /connect`

Establish long-lived WebSocket connection for receiving popup requests.

**Headers:**
- `Authorization: Bearer <token>`
- `Upgrade: websocket`

**Client → Server Messages:**
```json
{"type": "ready", "device_name": "laptop-1"}
{"type": "result", "id": "popup-uuid", "result": {...}}
{"type": "pong"}
```

**Server → Client Messages:**
```json
{"type": "show_popup", "id": "uuid", "definition": {...}, "timeout_ms": 30000}
{"type": "close_popup", "id": "uuid"}
{"type": "ping"}
```

### Create Popup: `POST /show-popup`

Create a new popup request. Blocks until first connected client responds or timeout.

**Headers:**
- `Authorization: Bearer <token>`
- `Content-Type: application/json`

**Request Body:**
```json
{
  "definition": {
    "title": "Popup Title",
    "elements": [...]
  },
  "timeout_ms": 30000
}
```

**Response:**
```json
{
  "status": "completed",
  "button": "submit",
  ...
}
```

## Authentication

Both endpoints require `Authorization: Bearer <token>` header. The token must match the `POPUP_AUTH_SECRET` environment variable.

## Durable Object Behavior

- Single DO instance manages all WebSocket connections
- WebSocket hibernation reduces memory usage and duration charges
- Concurrent popup requests supported (tracked by UUID)
- "First response wins" - first client to respond resolves the HTTP request
- All clients receive `close_popup` message when popup completes

## Required Secrets

Set these via `wrangler secret put`:

- `POPUP_AUTH_SECRET` - Bearer token for authentication

## Development Notes

- TypeScript types in `src/protocol.ts` must stay synchronized with Rust types in `crates/popup-common/src/protocol.rs`
- Uses WebSocket hibernation API for efficient resource usage
- Single Durable Object instance (ID: "global") handles all traffic
