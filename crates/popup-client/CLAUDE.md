# popup-client

WebSocket daemon that connects to Cloudflare relay and renders popups via subprocess spawning.

## Purpose

Remote client component of popup-mcp distributed architecture:
- Persistent WebSocket connection to Cloudflare PopupSession Durable Object
- Listens for ShowPopup messages from relay
- Spawns popup-gui subprocess for each request
- Monitors subprocess stdout for JSON result
- Sends result back to relay via WebSocket
- Handles timeout and cleanup

## Architecture

### Main Flow

1. **Connection Setup**
   - Read config from `~/.config/popup-client/config.toml`
   - Parse server_url (must be `wss://` for TLS)
   - Connect WebSocket with TLS via `tokio-tungstenite` + `rustls`
   - Send `ClientMessage::Ready` with optional device_name

2. **Request Handling**
   - Receive `ServerMessage::ShowPopup { id, definition, timeout_ms }`
   - Serialize definition to JSON
   - Spawn `popup-gui --stdin` subprocess
   - Write JSON to subprocess stdin
   - Read result from subprocess stdout
   - Parse PopupResult JSON
   - Send `ClientMessage::Result { id, result }` to relay

3. **Cleanup**
   - Receive `ServerMessage::ClosePopup { id }` → kill subprocess if running
   - Handle subprocess exit → send result or error
   - Timeout → send `PopupResult::Timeout` if no response

### Configuration

**Default location:** `~/.config/popup-client/config.toml`

```toml
server_url = "wss://popup-relay.example.workers.dev/connect"
device_name = "laptop"  # Optional, shown in relay logs
```

**Platform-specific paths:**
- macOS/Linux: `~/.config/popup-client/config.toml`
- Windows: `%APPDATA%\popup-client\config.toml`

**Example in repo:** `config.toml.example`

**CLI override:** Use `--config <path>` to specify alternate config file location

### WebSocket Protocol

Uses popup-common protocol types:

**Sent by client:**
- `Ready { device_name? }` - Initial handshake
- `Result { id, result }` - Popup completion
- `Pong` - Response to Ping

**Received from server:**
- `ShowPopup { id, definition, timeout_ms }` - Request to show popup
- `ClosePopup { id }` - Cancel running popup
- `Ping` - Keepalive

### Subprocess Management

**Spawning:**
```rust
Command::new("popup-gui")
    .arg("--stdin")
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::inherit())
    .spawn()
```

**Communication:**
- Write PopupDefinition JSON to stdin
- Read PopupResult JSON from stdout line-by-line
- Stderr inherits terminal (visible in daemon output)

**Lifecycle:**
- Subprocess exit code 0 → parse stdout as PopupResult
- Non-zero exit → treat as Cancelled
- Timeout → send Timeout result, kill subprocess
- ClosePopup → kill subprocess, no result sent

## Error Handling

**Connection errors:**
- Invalid server_url → exit with error message
- TLS handshake failure → exit with TLS error
- WebSocket upgrade failure → exit with protocol error

**Protocol errors:**
- Invalid JSON from relay → log warning, continue
- Unknown message type → log warning, ignore

**Subprocess errors:**
- popup-gui not found → send error result to relay
- Invalid JSON from subprocess → send Cancelled result
- Subprocess crash → send Cancelled result

## Common Commands

```bash
# Build
cargo build -p popup-client --release

# Run with default config (~/.config/popup-client/config.toml)
cargo run -p popup-client

# Run with custom config file
cargo run -p popup-client -- --config /path/to/config.toml

# Run with explicit server URL (overrides config)
cargo run -p popup-client -- --server-url wss://popup-relay.example.workers.dev/connect

# Set device name (overrides config)
cargo run -p popup-client -- --device-name "work-laptop"

# Combine flags
cargo run -p popup-client -- --server-url wss://example.workers.dev/connect --device-name "test-machine"

# Install locally
cargo install --path crates/popup-client

# Run installed binary
popup-client

# Test (requires running relay server)
cargo test -p popup-client
```

## CLI Arguments

```
popup-client [OPTIONS]

Options:
  --server-url <URL>       WebSocket URL (wss:// only, overrides config)
  --device-name <NAME>     Device identifier (overrides config)
  --config <PATH>          Path to config file (default: ~/.config/popup-client/config.toml)
  -h, --help              Print help
```

**Precedence:** CLI flags > config file > defaults

## Dependencies

- `tokio` - Async runtime
- `tokio-tungstenite` - WebSocket client with TLS support
- `rustls` / `rustls-native-certs` - TLS implementation
- `serde_json` - JSON serialization
- `popup-common` - Protocol types
- `anyhow` - Error handling
- `toml` - Config parsing
- `dirs` - Cross-platform config directory

## Design Principles

- **Persistent connection** - Single long-lived WebSocket, not per-request
- **Stateless operation** - No local state beyond active subprocess map
- **Clean subprocess isolation** - Each popup is independent process
- **First-response-wins** - Relay handles multiple clients, this client just responds
- **TLS required** - Only wss:// allowed, not ws:// (security by default)
- **Fail gracefully** - Connection drops → log error, exit (reconnection manual for now)
- **Config flexibility** - CLI overrides for dev/testing without modifying config file

## Integration with popup-gui

The client requires popup-gui binary to be available in PATH or same directory:
1. Client receives ShowPopup message from relay
2. Spawns `popup --stdin` subprocess
3. Writes PopupDefinition JSON to subprocess stdin
4. Reads PopupResult JSON from subprocess stdout
5. Forwards result back to relay

**Installation pattern:**
```bash
# Install both binaries
cargo install --path crates/popup-gui
cargo install --path crates/popup-client

# Verify popup binary is accessible
which popup  # Should show installed location

# Start client daemon
popup-client
```

## Future Enhancements

- Automatic reconnection with exponential backoff
- Multiple concurrent popup support (currently one at a time)
- Popup queue when multiple requests arrive
- Config file validation on startup
- Systemd/launchd service files for auto-start
- Binary auto-discovery (check multiple paths for popup binary)
