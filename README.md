# popup-mcp

**Native GUI popups via MCP** - Display interactive popup windows from AI assistants through the Model Context Protocol.

## Overview

popup-mcp provides three ways to create native GUI popups with form elements:

1. **Local MCP Server** - Direct stdio integration for Claude Desktop
2. **Remote MCP Server** - Cloudflare Workers relay with WebSocket client daemon (OAuth or header auth)
3. **Simple HTTP API** - Direct POST endpoint, no auth, no MCP overhead

Built with Rust (egui GUI) and TypeScript (Cloudflare Workers), supporting text, sliders, checkboxes, dropdowns, multiselect, conditional visibility, and nested forms.

## Quick Start

### Local Mode (Claude Desktop)

```bash
# Install popup binary
cargo install --path crates/popup-gui

# Add to Claude Desktop config (~/.config/Claude/claude_desktop_config.json)
{
  "mcpServers": {
    "popup": {
      "command": "popup",
      "args": ["--mcp-server"]
    }
  }
}
```

Use `remote_popup` tool from Claude Desktop - popup appears on your machine.

### Remote Mode (Distributed)

```bash
# Install client daemon
cargo install --path crates/popup-client

# Configure client (~/.config/popup-client/config.toml)
server_url = "wss://your-worker.workers.dev/connect"
device_name = "laptop"

# Start daemon
popup-client

# Deploy Cloudflare Worker
cd cloudflare
npm install
npx wrangler deploy

# Configure Claude Desktop for remote access
{
  "mcpServers": {
    "popup-remote": {
      "url": "https://your-worker.workers.dev/mcp",
      "transport": {"type": "streamableHttp"}
    }
  }
}
```

Popups appear on machine running client daemon, triggered from anywhere via MCP.

### Simple HTTP API

```bash
# No client needed - just running popup-gui binary required

curl -X POST https://your-worker.workers.dev/popup \
  -H "Content-Type: application/json" \
  -d '{
    "definition": {
      "title": "Confirm",
      "elements": [
        {"type": "text", "content": "Delete all files?"}
      ]
    },
    "timeout_ms": 60000
  }'
```

## Architecture

### Components

- **`popup-gui`** - Native egui GUI renderer (Rust)
  - Reads JSON from stdin or file
  - Displays popup window
  - Returns JSON result to stdout
  - MCP server mode for local integration

- **`popup-client`** - WebSocket daemon (Rust)
  - Connects to Cloudflare relay
  - Spawns popup-gui subprocesses
  - Forwards results back to relay

- **`cloudflare/`** - Workers relay (TypeScript)
  - Durable Object WebSocket server
  - GitHub OAuth or header-based auth
  - MCP agent with `remote_popup` tool
  - Simple POST endpoint at `/popup`

- **`popup-common`** - Shared protocol types (Rust)
  - PopupDefinition, PopupResult
  - WebSocket message types
  - Serialization via serde

### Request Flow

**Remote Mode:**
```
Claude Desktop → Cloudflare Worker (MCP) → Durable Object
    ↓
WebSocket broadcast → popup-client daemon → popup-gui subprocess
    ↓
User interaction → JSON result → client → DO → Worker → Claude
```

**Local Mode:**
```
Claude Desktop → popup binary (MCP server) → spawns popup-gui
    ↓
User interaction → JSON result → MCP response
```

## Popup Definition Format

```json
{
  "title": "Example Popup",
  "elements": [
    {"type": "text", "content": "Information text"},
    {"type": "slider", "label": "Volume", "min": 0, "max": 100, "default": 50},
    {"type": "checkbox", "label": "Enable feature", "default": false},
    {"type": "textbox", "label": "Name", "placeholder": "Enter name", "rows": 1},
    {"type": "choice", "label": "Theme", "options": ["Light", "Dark", "Auto"], "default": 0},
    {"type": "multiselect", "label": "Features", "options": ["A", "B", "C"]},
    {
      "type": "group",
      "label": "Settings",
      "elements": [/* nested elements */]
    },
    {
      "type": "conditional",
      "condition": "Enable feature",
      "elements": [/* shown when condition true */]
    }
  ]
}
```

### Conditional Visibility

**Inline conditionals** - elements shown when option selected:
```json
{
  "type": "checkbox",
  "label": "Advanced mode",
  "conditional": [
    {"type": "slider", "label": "Complexity", "min": 1, "max": 10}
  ]
}
```

**Standalone conditionals** - supports complex logic:
```json
{
  "type": "conditional",
  "condition": {"field": "Mode", "value": "Advanced"},
  "elements": [/* shown when Mode == "Advanced" */]
}
```

**Count conditions:**
```json
{
  "type": "conditional",
  "condition": {"field": "Items", "count": ">2"},
  "elements": [/* shown when >2 items selected */]
}
```

See `examples/` directory for comprehensive examples.

## Endpoints

### OAuth MCP Endpoint (GitHub Auth)
```
GET  /authorize - OAuth consent flow
GET  /callback  - OAuth callback
POST /mcp       - MCP Streamable HTTP endpoint
```

Tool: `remote_popup` with 30-second default timeout

### Header Auth MCP Endpoint (For Letta, etc.)
```
POST /header_auth - MCP with Bearer token auth
```

Headers: `Authorization: Bearer <token>`
Tool: `remote_popup` with 5-minute default timeout

### Simple POST Endpoint (No Auth)
```
POST /popup - Direct HTTP, no MCP protocol
```

Returns popup result JSON or timeout error.

### WebSocket Endpoint
```
WS /connect - Client daemon connection
```

Accepts WebSocket upgrade, used by popup-client daemon.

## Configuration

### Cloudflare Worker Secrets

```bash
# Required for OAuth endpoint
npx wrangler secret put GITHUB_CLIENT_ID
npx wrangler secret put GITHUB_CLIENT_SECRET
npx wrangler secret put COOKIE_ENCRYPTION_KEY

# Required for header auth endpoint
npx wrangler secret put AUTH_TOKEN
```

### Durable Objects

- `PopupSession` - WebSocket server, manages client connections
- `PopupMcpAgent` - OAuth-based MCP agent
- `HeaderAuthMcpAgent` - Token-based MCP agent

### Client Configuration

`~/.config/popup-client/config.toml`:
```toml
server_url = "wss://popup-relay.example.workers.dev/connect"
device_name = "laptop"  # Optional
gui_binary = "popup"    # Optional, defaults to "popup-gui"
```

## Development

### Build

```bash
# Rust workspace
cargo build --release

# TypeScript (Cloudflare)
cd cloudflare
npm install
npm run build
```

### Test

```bash
# Rust tests
cargo test

# TypeScript tests (includes Durable Object tests)
cd cloudflare
npm test

# Manual popup test
echo '{"title":"Test","elements":[{"type":"text","content":"Hello"}]}' | popup --stdin
```

### Deploy

```bash
# Deploy to Cloudflare
cd cloudflare
npx wrangler deploy

# Install binaries locally
cargo install --path crates/popup-gui
cargo install --path crates/popup-client
```

## Examples

- `examples/simple_confirm.json` - Basic confirmation dialog
- `examples/settings.json` - Multi-input form
- `examples/conditional_settings.json` - Conditional visibility
- `cloudflare/nested_example.json` - Complex nested conditionals
- `cloudflare/test_definition.json` - Quick test example

## Use Cases

- **AI Assistant Confirmations** - "Delete 500 files?" before destructive operations
- **Form Input** - Collect structured data during AI workflows
- **Settings Configuration** - Interactive configuration dialogs
- **Human-in-the-Loop** - Get user input/approval mid-task
- **Debugging** - Display state or ask debug questions
- **Templated Workflows** - Custom tools via template system

## Integration Examples

### Python (Letta)
```python
from letta_tool import show_popup

result = show_popup({
    "title": "Approve Action",
    "elements": [
        {"type": "text", "content": "Proceed with deployment?"}
    ]
})

if result["status"] == "completed":
    # User clicked Submit
    deploy()
```

### Claude Desktop (MCP)
Just use the `remote_popup` tool - no code needed.

### Custom Templates

Create reusable popups at `~/.config/popup-mcp/popup.toml`:
```toml
[[template]]
name = "confirm_delete"
file = "confirm_delete.json"

[template.params]
item_name = { type = "string", required = true }
```

## Architecture Decisions

- **Subprocess isolation** - Each popup runs in separate process for clean lifecycle
- **First-response-wins** - Multiple clients supported, first result completes request
- **Hibernatable WebSockets** - Durable Objects persist connections across Worker restarts
- **Top-level routing** - `/connect` and `/popup` bypass OAuth middleware for WebSocket/simple use
- **Explicit binding names** - MCP agents specify DO binding to avoid collisions
- **JSON-based structure** - Clean, explicit definitions with no parsing ambiguities

## License

MIT

## Contributing

See `CLAUDE.md` for development guidance and architecture details.
