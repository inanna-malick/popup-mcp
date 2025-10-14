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

For local development, create `.dev.vars`:

```bash
# .dev.vars
AUTH_TOKEN=your-secret-token-here
GITHUB_CLIENT_ID=your-github-oauth-client-id
GITHUB_CLIENT_SECRET=your-github-oauth-client-secret
COOKIE_ENCRYPTION_KEY=your-32-byte-hex-key
```

For production:

```bash
wrangler secret put AUTH_TOKEN
wrangler secret put GITHUB_CLIENT_ID
wrangler secret put GITHUB_CLIENT_SECRET
wrangler secret put COOKIE_ENCRYPTION_KEY
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

**Authentication:** None required (clients use trust-on-first-use model)

**Headers:**
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

### Create Popup: `POST /popup`

Direct HTTP endpoint for creating popup requests. Blocks until first connected client responds or timeout.

**Authentication:** Required - `Authorization: Bearer <token>` header matching `AUTH_TOKEN` environment variable

**Headers:**
- `Authorization: Bearer <token>`
- `Content-Type: application/json`

**Request Body:**
```json
{
  "definition": {
    "title": "Popup Title",
    "elements": [
      {"type": "text", "content": "Message"},
      {"type": "slider", "label": "Volume", "min": 0, "max": 100}
    ]
  },
  "timeout_ms": 300000
}
```

**Response (Success):**
```json
{
  "status": "completed",
  "button": "submit",
  "Volume": "75/100"
}
```

**Response (No Clients):**
```json
{
  "status": "error",
  "message": "No clients connected"
}
```

**Response (Timeout):**
```json
{
  "status": "timeout",
  "message": "No response received within 300000ms"
}
```

**Python Example:**
```python
import os
import requests

def show_popup(definition: dict, timeout_ms: int = 300000) -> dict:
    auth_token = os.getenv('POPUP_AUTH_TOKEN')
    if not auth_token:
        return {"status": "error", "message": "POPUP_AUTH_TOKEN not set"}

    response = requests.post(
        "https://your-worker.workers.dev/popup",
        json={"definition": definition, "timeout_ms": timeout_ms},
        headers={"Authorization": f"Bearer {auth_token}"},
        timeout=(timeout_ms / 1000) + 5
    )
    return response.json()
```

### MCP with Header Auth: `POST /header_auth`

Alternative MCP endpoint using bearer token authentication instead of GitHub OAuth.

**Authentication:** Required - `Authorization: Bearer <token>` header matching `AUTH_TOKEN` environment variable

**Use Case:** Server-to-server integrations (e.g., Letta agents)

**Available Tool:**
- `remote_popup`: Same schema as SSE endpoint

## Authentication Summary

| Endpoint | Authentication | Use Case |
|----------|----------------|----------|
| `/sse`, `/mcp` | GitHub OAuth | Claude Desktop (browser-based) |
| `/header_auth` | Bearer token | Letta / MCP server-to-server |
| `/popup` | Bearer token | Direct HTTP API (Python, curl, etc.) |
| `/connect` | None | WebSocket clients (popup-client daemon) |

## Durable Object Behavior

- Single DO instance manages all WebSocket connections
- WebSocket hibernation reduces memory usage and duration charges
- Concurrent popup requests supported (tracked by UUID)
- "First response wins" - first client to respond resolves the HTTP request
- All clients receive `close_popup` message when popup completes

## Required Secrets

Set these via `wrangler secret put`:

- `AUTH_TOKEN` - Bearer token for `/popup` and `/header_auth` endpoints
- `GITHUB_CLIENT_ID` - GitHub OAuth client ID (for `/mcp` OAuth flow)
- `GITHUB_CLIENT_SECRET` - GitHub OAuth client secret (for `/mcp` OAuth flow)
- `COOKIE_ENCRYPTION_KEY` - 32-byte hex key for OAuth cookies (generate with `openssl rand -hex 32`)

## Development Notes

- TypeScript types in `src/protocol.ts` must stay synchronized with Rust types in `crates/popup-common/src/protocol.rs`
- Uses WebSocket hibernation API for efficient resource usage
- Single Durable Object instance (ID: "global") handles all traffic
