# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Popup-MCP: Native GUI Popups via MCP

Popup-MCP is an MCP (Model Context Protocol) server that enables AI assistants to create native GUI popup windows using JSON structure. The project consists of:
- **Rust workspace**: Native GUI rendering, local MCP server, and remote client daemon
- **Cloudflare Workers**: Distributed relay infrastructure for remote popup invocation

## Common Development Commands

### Rust Workspace

```bash
# Build all workspace crates
cargo build --release

# Build specific crate
cargo build -p popup-gui --release
cargo build -p popup-client --release

# Run all tests
cargo test

# Run tests with output (useful for debugging)
cargo test -- --nocapture

# Run a specific test
cargo test test_simple_confirmation

# Run tests in a specific module
cargo test tests::json_parser_tests

# Build and install locally
cargo install --path crates/popup-gui

# Run formatting check
cargo fmt --check

# Apply formatting
cargo fmt

# Run linter
cargo clippy

# Test popup directly from command line with JSON
echo '{"title": "Test", "elements": [{"type": "text", "content": "Hello"}]}' | cargo run -p popup-gui -- --stdin

# Test with example file
cargo run -p popup-gui -- --file examples/simple_confirm.json

# Run popup client daemon (connects to Cloudflare relay)
cargo run -p popup-client -- --server-url wss://your-worker.workers.dev/connect
```

### Cloudflare Workers (TypeScript)

```bash
# Navigate to cloudflare directory
cd cloudflare

# Run tests with Miniflare (includes Durable Object and WebSocket tests)
npm test

# Run specific test file
npm test -- durable-object.test.ts

# Run tests in watch mode
npm run test:watch

# Local development server
npm run dev

# Deploy to Cloudflare
npm run deploy

# Type check
npx tsc --noEmit
```

## Important Note on Buttons

**Buttons are no longer user-specifiable.** Every popup automatically includes a single "Submit" button at the bottom. Users can press the Submit button to confirm or use the Escape key to cancel. The PopupResult will include `"button": "submit"` or `"button": "cancel"` accordingly.

## High-Level Architecture

### System Overview

The project implements a distributed popup system with three main components:

1. **Local GUI** (`crates/popup-gui`): Native egui-based popup renderer
2. **Remote Client** (`crates/popup-client`): WebSocket daemon that connects to Cloudflare relay
3. **Cloudflare Relay** (`cloudflare/`): Durable Object-based WebSocket bridge for remote invocation

### Architecture Flow

```
MCP Client (Claude Desktop)
  ↓ (1) Streamable HTTP with GitHub OAuth
Cloudflare Worker (MCP Agent DO)
  ↓ (2) Internal HTTP to PopupSession DO
PopupSession Durable Object
  ↓ (3) WebSocket broadcast to clients
popup-client daemon (wss://)
  ↓ (4) Spawn subprocess with --stdin
popup binary (native egui window)
  ↓ (5) JSON result to stdout
popup-client daemon
  ↓ (6) WebSocket result back to DO
PopupSession DO
  ↓ (7) HTTP response to MCP Agent
MCP Client (receives result)
```

**Local MCP Usage (no Cloudflare):**
```
MCP Client
  ↓ JSON-RPC over stdio
popup binary (MCP server mode)
  ↓ Spawns itself with --stdin
popup binary (renders window)
  ↓ JSON result
MCP Client (receives result)
```

### Rust Workspace Structure

**crates/popup-common** - Shared protocol types
- `PopupDefinition`, `Element`, `PopupResult` - Core data structures
- `ServerMessage`, `ClientMessage` - WebSocket protocol types
- Serialization via serde for JSON/MessagePack compatibility

**crates/popup-gui** - Native GUI implementation
- `json_parser.rs`: JSON → PopupDefinition deserialization
- `gui/mod.rs`: egui window logic and event loop
- `gui/widget_renderers.rs`: Individual widget rendering
- `mcp_server.rs`: MCP server (JSON-RPC over stdio)
- `schema.rs`: MCP tool schema definitions
- `templates.rs`: Dynamic template system with Handlebars
- Binary: `popup` (operates in 3 modes: MCP server, stdin, file)

**crates/popup-client** - Remote client daemon
- WebSocket client connecting to Cloudflare relay
- Spawns popup-gui subprocesses for each popup request
- Manages popup lifecycle (show, monitor, close)
- Streams results back to Cloudflare relay
- Config: `~/.config/popup-client/config.toml`

### Cloudflare Workers Structure

**cloudflare/src/index.ts** - Worker entry point
- Routes `/authorize`, `/callback`, `/token` → GitHub OAuth flow
- Routes `/sse` → MCP SSE endpoint (deprecated protocol)
- Routes `/mcp` → MCP Streamable HTTP endpoint (current protocol)
- Routes `/connect` → WebSocket upgrade for popup-client daemons
- Routes `/show-popup` → Internal HTTP POST to PopupSession DO
- **Auth Model**: GitHub OAuth via `@cloudflare/workers-oauth-provider`
  - Consent dialog with approval tracking (signed cookies)
  - User props (login, name, email, accessToken) passed to MCP agent
  - Pattern from `/Users/inannamalick/dev/cf-ai/demos/remote-mcp-github-oauth`

**cloudflare/src/popup-session.ts** - Durable Object implementation
- Hibernatable WebSocket server for persistent connections
- Manages multiple client connections with session metadata
- Broadcasts popup requests to all connected clients
- First-response-wins pattern for popup results
- Timeout handling with automatic cleanup
- State survives Worker restarts via hibernation API

**cloudflare/src/mcp-server.ts** - MCP agent implementation
- Extends `McpAgent` from `agents/mcp` package
- Implements Streamable HTTP protocol (current) and SSE (deprecated)
- Provides `show_popup` tool to MCP clients
- Receives user props from OAuth (login, name, email, accessToken)
- Forwards popup requests to PopupSession DO via internal HTTP
- Returns popup results or timeout errors

**cloudflare/src/protocol.ts** - TypeScript protocol types
- Mirrors Rust protocol types from popup-common
- Ensures wire format compatibility between Rust and TypeScript

### Protocol Flow

**Remote (Cloudflare relay):**
1. **OAuth**: User authorizes via GitHub OAuth flow, gets access token
2. **Client Connect**: popup-client → wss:// → PopupSession DO (hibernatable WebSocket)
3. **Client Ready**: Sends `ClientMessage::Ready { device_name }`
4. **MCP Request**: Claude Desktop → Streamable HTTP → MCP Agent DO
5. **Show Popup**: MCP Agent → internal HTTP → PopupSession DO → `ServerMessage::ShowPopup` → WebSocket broadcast
6. **Render**: popup-client spawns `popup --stdin`, writes JSON, monitors stdout
7. **Result**: popup exits with JSON → client sends `ClientMessage::Result` → DO
8. **Cleanup**: DO resolves promise, broadcasts `ServerMessage::ClosePopup`

**Local (stdio MCP server):**
1. **MCP Request**: Client → JSON-RPC via stdin
2. **Spawn Renderer**: MCP server spawns `popup --stdin` subprocess
3. **Render**: Subprocess renders window, exits with JSON result
4. **Result**: MCP server reads stdout, returns to client via JSON-RPC

### Key Design Decisions

- **Hibernatable WebSockets**: Durable Objects persist connections across Worker restarts
- **First-response-wins**: Multiple clients can connect; first result completes the request
- **Subprocess isolation**: Each popup runs in separate process, clean lifecycle
- **Shared protocol types**: popup-common ensures Rust/TypeScript compatibility
- **JSON-based structure**: Clean, explicit definition with no parsing ambiguities
- **Type safety**: JSON schema provides clear structure validation
- **Nested support**: Natural support for conditionals and groups through JSON nesting
- **Self-spawning architecture**: MCP server spawns itself with --stdin for rendering
- **Template-driven tools**: Dynamic MCP tool generation from user config files
- **OAuth integration**: GitHub OAuth for remote access with user context propagation

### Testing Strategy

**Rust Tests** (`crates/popup-gui/src/tests/`):
- `json_parser_tests.rs`: Core JSON parsing tests for all widget types
- `integration_tests.rs`: Integration tests with example files and state management
- `conditional_filtering_tests.rs`: Conditional visibility logic
- `template_tests.rs`: Template system tests

**TypeScript Tests** (`cloudflare/test/`):
- `durable-object.test.ts`: Durable Object lifecycle, hibernation, broadcast
- `websocket.test.ts`: WebSocket protocol message handling
- `integration.test.ts`: End-to-end popup flow
- `mcp-server.test.ts`: MCP SSE endpoint behavior
- `auth.test.ts`: Bearer token authentication
- Uses Miniflare for local Durable Object testing

**Example JSON files** (`examples/`):
- `simple_confirm.json`: Basic confirmation dialog
- `settings.json`: Complex settings form
- `conditional_settings.json`: Settings with conditional visibility
- `choice_demo.json`: Choice widget demonstrations

## JSON Structure Reference

### Basic Structure
```json
{
  "title": "Popup Title",
  "elements": [
    // Array of element objects
  ]
}
```

### Element Types

#### Text
```json
{"type": "text", "content": "Display text"}
```

#### Slider
```json
{
  "type": "slider",
  "label": "Volume",
  "min": 0,
  "max": 100,
  "default": 50  // Optional, defaults to midpoint
}
```

#### Checkbox
```json
{
  "type": "checkbox",
  "label": "Enable feature",
  "default": true  // Optional, defaults to false
}
```

#### Textbox
```json
{
  "type": "textbox",
  "label": "Name",
  "placeholder": "Enter your name",  // Optional
  "rows": 5  // Optional, for multiline
}
```

#### Choice (Single Selection)
```json
{
  "type": "choice",
  "label": "Theme",
  "options": ["Light", "Dark", "Auto"]
}
```

#### Multiselect
```json
{
  "type": "multiselect",
  "label": "Features",
  "options": ["Feature A", "Feature B", "Feature C"]
}
```

#### Group
```json
{
  "type": "group",
  "label": "Settings",
  "elements": [
    // Nested elements
  ]
}
```

#### Conditional
```json
{
  "type": "conditional",
  "condition": "Checkbox Label",  // Simple form
  "elements": [
    // Elements shown when condition is true
  ]
}
```

Complex conditions:
```json
{
  "type": "conditional",
  "condition": {
    "type": "selected",
    "label": "Mode",
    "value": "Advanced"
  },
  "elements": [...]
}
```

```json
{
  "type": "conditional",
  "condition": {
    "type": "count",
    "label": "Items",
    "value": 5,
    "op": ">"  // Operators: >, <, >=, <=, =
  },
  "elements": [...]
}
```

## Example Popups

### Simple Confirmation
```json
{
  "title": "Delete File?",
  "elements": [
    {"type": "text", "content": "This action cannot be undone."}
  ]
}
```

### Settings Form
```json
{
  "title": "Settings",
  "elements": [
    {"type": "slider", "label": "Volume", "min": 0, "max": 100, "default": 75},
    {"type": "checkbox", "label": "Notifications", "default": true},
    {"type": "choice", "label": "Theme", "options": ["Light", "Dark", "Auto"]}
  ]
}
```

### Conditional UI
```json
{
  "title": "Advanced Settings",
  "elements": [
    {"type": "checkbox", "label": "Show advanced", "default": false},
    {
      "type": "conditional",
      "condition": "Show advanced",
      "elements": [
        {"type": "slider", "label": "Debug level", "min": 0, "max": 10},
        {"type": "textbox", "label": "Log file", "placeholder": "/tmp/debug.log"}
      ]
    }
  ]
}
```

## Cloudflare Deployment

### Prerequisites

1. **GitHub OAuth App** (for remote MCP access)
   - Create at: https://github.com/settings/developers
   - Set Authorization callback URL: `https://your-worker.workers.dev/callback`
   - Note Client ID and Client Secret

2. **Cloudflare Secrets** (set via wrangler)
   ```bash
   cd cloudflare
   npx wrangler secret put GITHUB_CLIENT_ID
   npx wrangler secret put GITHUB_CLIENT_SECRET
   npx wrangler secret put COOKIE_ENCRYPTION_KEY  # Generate with: openssl rand -hex 32
   ```

3. **Deploy Worker**
   ```bash
   npm run deploy
   ```

### Claude Desktop Configuration

Add to Claude Desktop config (`~/Library/Application Support/Claude/claude_desktop_config.json` on macOS):

```json
{
  "mcpServers": {
    "popup-remote": {
      "url": "https://your-worker.workers.dev/mcp",
      "transport": {
        "type": "streamableHttp"
      }
    }
  }
}
```

On first use, Claude Desktop will redirect to GitHub OAuth consent flow.

### popup-client Configuration

1. Create `~/.config/popup-client/config.toml`:
   ```toml
   server_url = "wss://your-worker.workers.dev/connect"
   device_name = "laptop"
   ```

2. Start daemon:
   ```bash
   popup-client
   ```

The client maintains persistent WebSocket connection and renders popups when MCP requests arrive.

## Template System

The `popup` binary supports dynamic template loading for common popup patterns.

**Setup:**
1. Create `~/.config/popup-mcp/popup.toml` config file
2. Add template definitions with parameters
3. Create template JSON files with Handlebars placeholders
4. Start MCP server - each template becomes a tool

**Example popup.toml:**
```toml
[[template]]
name = "confirm_delete"
description = "Confirm destructive action"
file = "confirm_delete.json"

[template.params.item_name]
type = "string"
description = "Name of item to delete"
required = true

[[template]]
name = "quick_settings"
description = "Quick settings dialog"
file = "quick_settings.json"
```

**Template JSON (confirm_delete.json):**
```json
{
  "title": "Delete {{item_name}}?",
  "elements": [
    {"type": "text", "content": "Permanently delete {{item_name}}?"}
  ]
}
```

**MCP Tool Usage:**
Each template becomes an MCP tool. When invoked, Handlebars substitutes variables and renders the popup.

## Development Principles

- **ALWAYS write unit tests, not main methods**. No main methods unless explicitly requested.
- Use the existing test patterns in `src/tests/` as examples
- Prefer iterators and for loops over manual iteration in Rust
- Avoid early optimizations without benchmarks
- **Wherever possible, write unit tests instead of using cargo run to test changes**
- Match existing Rust/TypeScript patterns (see crate CLAUDE.md files for specifics)