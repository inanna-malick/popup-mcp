# Popup Client Daemon

WebSocket client daemon that connects to the Cloudflare Worker relay and spawns local GUI popups on demand.

## Features

- Long-lived WebSocket connection to relay server
- Automatic reconnection with exponential backoff
- Spawns popup-gui subprocess for each popup request
- Bearer token authentication
- Configurable via TOML file and environment variables

## Setup

### 1. Create Config File

Create `~/.config/popup-client/config.toml`:

```toml
# WebSocket server URL (Cloudflare Worker endpoint)
server_url = "wss://your-worker.workers.dev/connect"

# Optional device name for identification
device_name = "laptop-1"

# Path to popup-gui binary (default: "popup-gui" from PATH)
# gui_binary = "/path/to/popup-gui"
```

Or copy the example:

```bash
mkdir -p ~/.config/popup-client
cp config.toml.example ~/.config/popup-client/config.toml
# Edit config.toml with your settings
```

### 2. Set Auth Token

```bash
export POPUP_AUTH_TOKEN="your-secret-token-here"
```

This must match the `POPUP_AUTH_SECRET` configured in your Cloudflare Worker.

### 3. Run the Daemon

```bash
cargo run --release -p popup-client
```

Or after installation:

```bash
popup-client
```

## CLI Options

```
popup-client [OPTIONS]

Options:
      --server-url <SERVER_URL>
          WebSocket server URL (overrides config)

      --device-name <DEVICE_NAME>
          Device name for identification (overrides config)

      --config <CONFIG>
          Path to config file (default: ~/.config/popup-client/config.toml)

  -h, --help
          Print help
```

## Protocol Flow

1. **Connect**: Daemon connects to WebSocket server with auth header
2. **Ready**: Sends `{"type": "ready", "device_name": "..."}` message
3. **Show Popup**: Receives `{"type": "show_popup", "id": "uuid", "definition": {...}, "timeout_ms": 30000}`
4. **Spawn GUI**: Launches `popup-gui --stdin` subprocess with JSON definition
5. **Collect Result**: Reads JSON result from subprocess stdout
6. **Send Result**: Transmits `{"type": "result", "id": "uuid", "result": {...}}` back to server
7. **Close Popup**: Receives `{"type": "close_popup", "id": "uuid"}` when another client responds
8. **Cleanup**: Kills subprocess if still running

## Subprocess Management

- Each popup gets a unique subprocess
- GUI receives definition via stdin, outputs result to stdout
- When `close_popup` arrives, daemon kills subprocess if still running
- If user closes GUI (cancel/escape), subprocess exits and daemon sends that result
- Multiple concurrent popups supported (tracked by UUID)

## Reconnection

- Auto-reconnects on disconnect with exponential backoff (1s → 60s max)
- Resets backoff on successful connection
- Active popups are lost on reconnect (stateless across connections)

## Environment Variables

- `POPUP_AUTH_TOKEN` (required): Bearer token for authentication
- `RUST_LOG` (optional): Log level (e.g., `info`, `debug`)

## Example with Logging

```bash
RUST_LOG=info POPUP_AUTH_TOKEN="secret" popup-client
```

## Systemd/Launchd Setup

Service files for auto-start on boot can be added later. For now, run manually or use a terminal multiplexer.
