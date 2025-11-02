# Popup Relay - Cloudflare Workers + Durable Objects

Distributed relay infrastructure for remote popup invocation. Enables AI assistants and applications to trigger native GUI popups on remote machines via WebSocket connections.

## Overview

The Cloudflare Workers deployment provides:

1. **Remote MCP Server** - MCP endpoints with GitHub OAuth or Bearer token auth
2. **WebSocket Relay** - Durable Object-based persistent connection management
3. **Simple HTTP API** - Direct POST endpoint for non-MCP integrations
4. **Client Daemon** - Rust-based WebSocket client (`popup-client`)

## Architecture

```
AI Assistant/App → Cloudflare Worker (MCP) → PopupSession Durable Object
    ↓
WebSocket broadcast → popup-client daemon(s) → popup-gui subprocess
    ↓
User interaction → JSON result → client → DO → Worker → Response
```

### Components

- **Worker** (`src/index.ts`) - Entry point, auth, routing
- **PopupSession DO** (`src/popup-session.ts`) - Hibernatable WebSocket server
- **MCP Agents** (`src/mcp-server.ts`) - OAuth and header-based MCP implementations
- **Protocol** (`src/protocol.ts`) - TypeScript types matching Rust definitions

## Quick Start

### 1. Deploy Worker

```bash
cd cloudflare
npm install

# Configure secrets
npx wrangler secret put AUTH_TOKEN
npx wrangler secret put GITHUB_CLIENT_ID
npx wrangler secret put GITHUB_CLIENT_SECRET
npx wrangler secret put COOKIE_ENCRYPTION_KEY

# Deploy
npx wrangler deploy
```

### 2. Install and Configure Client Daemon

```bash
# Install popup-client
cargo install --path ../crates/popup-client

# Configure (~/.config/popup-client/config.toml)
server_url = "wss://your-worker.workers.dev/connect"
device_name = "laptop"

# Start daemon
popup-client
```

### 3. Configure Claude Desktop (Optional)

For GitHub OAuth-based MCP access:

```json
{
  "mcpServers": {
    "popup-remote": {
      "url": "https://your-worker.workers.dev/mcp",
      "transport": {"type": "streamableHttp"}
    }
  }
}
```

Or for header-based auth (Letta, etc.):

```json
{
  "mcpServers": {
    "popup-remote": {
      "url": "https://your-worker.workers.dev/header_auth",
      "transport": {"type": "streamableHttp"},
      "headers": {
        "Authorization": "Bearer YOUR_AUTH_TOKEN"
      }
    }
  }
}
```

## API Endpoints

### MCP Endpoints

#### GitHub OAuth Flow: `POST /mcp`

MCP Streamable HTTP endpoint with GitHub OAuth authentication.

**Authentication:** GitHub OAuth (redirects to consent flow on first use)

**Tool:** `remote_popup` with 30-second default timeout

**Configuration:**
```json
{
  "mcpServers": {
    "popup": {
      "url": "https://your-worker.workers.dev/mcp",
      "transport": {"type": "streamableHttp"}
    }
  }
}
```

#### Header-Based Auth: `POST /header_auth`

MCP endpoint using Bearer token authentication.

**Authentication:** `Authorization: Bearer <token>` header

**Tool:** `remote_popup` with 5-minute default timeout

**Use Case:** Server-to-server integrations (Letta agents, custom MCP clients)

**Tool Schema:**
```json
{
  "definition": {
    "title": "Popup Title",
    "elements": [
      {"text": "Message", "id": "message"},
      {"slider": "Volume", "id": "volume", "min": 0, "max": 100}
    ]
  },
  "timeout_ms": 300000
}
```

### WebSocket Endpoint: `GET /connect`

Establishes long-lived WebSocket connection for popup-client daemons.

**Authentication:** None (trust-on-first-use)

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

### Direct HTTP API: `POST /popup`

Direct HTTP endpoint bypassing MCP protocol.

**Authentication:** `Authorization: Bearer <token>` header

**Request:**
```json
{
  "definition": {
    "title": "Confirm Action",
    "elements": [
      {"text": "Delete all files?", "id": "confirm_msg"}
    ]
  },
  "timeout_ms": 60000
}
```

**Response (Success):**
```json
{
  "status": "completed",
  "button": "submit",
  "field_values": {...}
}
```

**Response (Timeout):**
```json
{
  "status": "timeout",
  "message": "No response received within 60000ms"
}
```

**Python Example:**
```python
import os
import requests

def show_popup(definition: dict, timeout_ms: int = 300000) -> dict:
    response = requests.post(
        "https://your-worker.workers.dev/popup",
        json={"definition": definition, "timeout_ms": timeout_ms},
        headers={"Authorization": f"Bearer {os.getenv('POPUP_AUTH_TOKEN')}"},
        timeout=(timeout_ms / 1000) + 5
    )
    return response.json()

# Usage
result = show_popup({
    "title": "Approve Action",
    "elements": [
        {"text": "Proceed with deployment?", "id": "approve_msg"}
    ]
})

if result.get("button") == "submit":
    print("User approved!")
```

See `letta_tool.py` for a complete Letta integration example.

## Configuration

### Required Secrets

Set via `wrangler secret put <NAME>`:

| Secret | Purpose | How to Generate |
|--------|---------|-----------------|
| `AUTH_TOKEN` | Bearer token for `/popup` and `/header_auth` | `openssl rand -base64 32` |
| `GITHUB_CLIENT_ID` | GitHub OAuth client ID | Create OAuth app at github.com/settings/developers |
| `GITHUB_CLIENT_SECRET` | GitHub OAuth client secret | From GitHub OAuth app settings |
| `COOKIE_ENCRYPTION_KEY` | OAuth cookie encryption | `openssl rand -hex 32` |

### GitHub OAuth App Setup

1. Go to https://github.com/settings/developers
2. Create new OAuth App
3. Set **Authorization callback URL**: `https://your-worker.workers.dev/callback`
4. Copy Client ID and Client Secret
5. Set as Wrangler secrets

### Client Configuration

Create `~/.config/popup-client/config.toml`:

```toml
server_url = "wss://your-worker.workers.dev/connect"
device_name = "my-laptop"  # Optional, defaults to hostname
gui_binary = "popup"       # Optional, defaults to "popup-gui"
```

## Durable Objects

### PopupSession

- Single instance handles all WebSocket connections
- Uses hibernatable WebSocket API for efficient resource usage
- Manages concurrent popup requests (tracked by UUID)
- First-response-wins pattern - first client response completes the request
- All connected clients receive `close_popup` after completion

### MCP Agents

- `PopupMcpAgent` - GitHub OAuth-based agent for browser MCP clients
- `HeaderAuthMcpAgent` - Token-based agent for server-to-server MCP

## Development

### Local Testing

```bash
# Start local dev server
npm run dev

# Run tests
npm test

# Run specific test
npm test -- durable-object.test.ts

# Type check
npx tsc --noEmit
```

### Testing with Real Client

```bash
# Terminal 1: Start local worker
npm run dev

# Terminal 2: Start client daemon (pointing to local)
popup-client --server-url ws://localhost:8787/connect

# Terminal 3: Test API
curl -X POST http://localhost:8787/popup \
  -H "Authorization: Bearer test-token" \
  -H "Content-Type: application/json" \
  -d '{
    "definition": {
      "title": "Test",
      "elements": [{"text": "Hello", "id": "test_msg"}]
    },
    "timeout_ms": 30000
  }'
```

### Protocol Synchronization

TypeScript types in `src/protocol.ts` must match Rust types in `../crates/popup-common/src/protocol.rs`.

When updating protocol:
1. Update Rust types first
2. Update TypeScript types to match
3. Run both test suites: `cargo test` and `npm test`

## Authentication Summary

| Endpoint | Authentication | Use Case |
|----------|----------------|----------|
| `/mcp` | GitHub OAuth | Claude Desktop (browser-based MCP clients) |
| `/header_auth` | Bearer token | Letta agents, server-to-server MCP |
| `/popup` | Bearer token | Direct HTTP API (Python, curl) |
| `/connect` | None | WebSocket clients (popup-client daemon) |

## Key Features

- **Hibernatable WebSockets** - Durable Objects persist connections across Worker restarts
- **First-response-wins** - Multiple clients can connect; first result completes the request
- **Concurrent requests** - Multiple popup requests tracked independently by UUID
- **Timeout handling** - Graceful timeout responses if no client responds
- **Multiple auth methods** - OAuth for browsers, Bearer tokens for server-to-server

## Troubleshooting

**No clients connected error:**
- Ensure popup-client daemon is running
- Check WebSocket connection: `popup-client --server-url wss://your-worker.workers.dev/connect`
- Verify client logs for connection errors

**Timeout errors:**
- User took too long to respond
- Client daemon crashed/restarted during popup display
- Network issues between client and Worker

**Authentication errors:**
- Verify secrets are set correctly: `wrangler secret list`
- For OAuth: Check GitHub OAuth app callback URL matches Worker URL
- For Bearer token: Ensure `AUTH_TOKEN` secret matches client configuration

## Examples

See example files:
- `nested_example.json` - Complex nested conditionals
- `test_definition.json` - Simple test popup
- `letta_tool.py` - Letta agent integration

## License

MIT
