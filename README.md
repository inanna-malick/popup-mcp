# popup-mcp

**Native GUI popups via MCP** - Display interactive popup windows from AI assistants through the Model Context Protocol.

## Overview

popup-mcp enables AI assistants to create native GUI popups with rich form elements including text, sliders, checkboxes, dropdowns, multiselect, and conditional visibility. Build dialogue trees with cascading conditional branches that adapt based on user selections.

Built with Rust (egui GUI) for native rendering and cross-platform support.

## Quick Start

### Install

```bash
# Install the popup binary
cargo install --path crates/popup-gui
```

### Test It

```bash
# Test with a simple popup
echo '{"title": "Hello", "elements": [{"text": "World!"}]}' | popup --stdin

# Try an example file
popup --file examples/simple_confirm.json
```

### Integrate with Claude Desktop

Add to your Claude Desktop config (`~/.config/Claude/claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "popup": {
      "command": "popup",
      "args": ["--mcp-server"]
    }
  }
}
```

Restart Claude Desktop. The `popup` tool will be available for creating GUI interactions.

## Popup Definition Format

Define popups using JSON with a title and array of elements. The V2 schema uses **element-as-key** format where the widget type is the JSON key, and all interactive elements require an `id` field.

```json
{
  "title": "Settings",
  "elements": [
    {
      "text": "Configure your preferences"
    },
    {
      "slider": "Volume",
      "id": "volume",
      "min": 0,
      "max": 100,
      "default": 75
    },
    {
      "checkbox": "Enable notifications",
      "id": "notifications",
      "default": true
    },
    {
      "textbox": "Username",
      "id": "username",
      "placeholder": "Enter username"
    },
    {
      "choice": "Theme",
      "id": "theme",
      "options": ["Light", "Dark", "Auto"]
    },
    {
      "multiselect": "Features",
      "id": "features",
      "options": ["A", "B", "C"]
    }
  ]
}
```

### Element Types

| Element | Description | Required Fields | Optional Fields |
|---------|-------------|-----------------|-----------------|
| **text** | Static text display | `text` (label) | `id`, `when` |
| **slider** | Numeric range selector | `slider` (label), `id`, `min`, `max` | `default`, `when`, `reveals` |
| **checkbox** | Boolean toggle | `checkbox` (label), `id` | `default`, `when`, `reveals` |
| **textbox** | Text input field | `textbox` (label), `id` | `placeholder`, `rows`, `when` |
| **choice** | Single selection dropdown | `choice` (label), `id`, `options` | `default`, `when`, `reveals`, option children |
| **multiselect** | Multiple selection list | `multiselect` (label), `id`, `options` | `when`, `reveals`, option children |
| **group** | Container for nested elements | `group` (label), `elements` | `when` |

### Conditional Visibility

Build rich dialogue trees where choices reveal additional options dynamically using **when clauses**, **reveals**, and **option-as-key nesting**.

#### When Clauses

Any element can have a `when` field for conditional visibility:

```json
{
  "checkbox": "Enable debug mode",
  "id": "debug_mode",
  "default": false
},
{
  "slider": "Log level",
  "id": "log_level",
  "min": 0,
  "max": 5,
  "when": "@debug_mode"
}
```

**When Clause Syntax:**
- `@id` - Boolean check (checkbox checked, has selections, etc.)
- `selected(@id, value)` - Check if specific value selected
- `count(@id) > 2` - Count-based check with operators: `>`, `<`, `>=`, `<=`, `==`
- `@id1 && @id2` - Logical AND
- `@id1 || @id2` - Logical OR
- `!@id` - Logical NOT

**Examples:**
```json
{
  "text": "Advanced mode with multiple features",
  "when": "@advanced && count(@features) > 2"
}
```

```json
{
  "slider": "Expert complexity",
  "id": "complexity",
  "min": 1,
  "max": 10,
  "when": "selected(@mode, Expert)"
}
```

#### Reveals (Inline Conditionals)

Elements that appear when a checkbox is checked or an option is selected:

```json
{
  "checkbox": "Enable advanced mode",
  "id": "enable_advanced",
  "default": false,
  "reveals": [
    {
      "slider": "Complexity",
      "id": "complexity",
      "min": 1,
      "max": 10
    },
    {
      "textbox": "Config file",
      "id": "config_file",
      "placeholder": "/etc/config"
    }
  ]
}
```

#### Option-as-Key Nesting

Choice and multiselect options can have children using the option text as a JSON key:

```json
{
  "choice": "Installation type",
  "id": "install_type",
  "options": ["Standard", "Custom"],
  "Custom": [
    {
      "textbox": "Install path",
      "id": "install_path",
      "placeholder": "/opt/app"
    }
  ]
}
```

Multiselect with per-option children:

```json
{
  "multiselect": "Features",
  "id": "features",
  "options": ["Basic", "Database", "Authentication"],
  "Database": [
    {
      "choice": "DB Type",
      "id": "db_type",
      "options": ["PostgreSQL", "MySQL", "MongoDB"]
    }
  ],
  "Authentication": [
    {
      "choice": "Auth Provider",
      "id": "auth_provider",
      "options": ["OAuth", "SAML", "LDAP"]
    }
  ]
}
```

### Cascading Conditionals

Combine all three conditional approaches for deep dialogue trees:

```json
{
  "title": "Project Setup",
  "elements": [
    {
      "choice": "Project type",
      "id": "project_type",
      "options": ["Simple", "Advanced"],
      "Advanced": [
        {
          "multiselect": "Features",
          "id": "features",
          "options": ["Database", "Authentication", "API"],
          "Database": [
            {
              "choice": "Database type",
              "id": "db_type",
              "options": ["PostgreSQL", "MySQL", "MongoDB"]
            }
          ],
          "Authentication": [
            {
              "choice": "Auth provider",
              "id": "auth_provider",
              "options": ["OAuth", "SAML", "LDAP"]
            }
          ]
        },
        {
          "checkbox": "Enable monitoring",
          "id": "enable_monitoring",
          "default": false,
          "when": "count(@features) >= 2",
          "reveals": [
            {
              "slider": "Monitoring interval (seconds)",
              "id": "monitor_interval",
              "min": 10,
              "max": 300,
              "default": 60
            }
          ]
        }
      ]
    }
  ]
}
```

This creates a responsive dialogue:
- Choosing "Advanced" reveals the features multiselect
- Selecting "Database" or "Authentication" reveals specific configuration options
- The monitoring checkbox appears only when 2+ features are selected
- Checking the monitoring checkbox reveals the interval slider

All interactions happen in a single popup with real-time conditional visibility.

## Examples

The `examples/` directory contains sample popups:

- **simple_confirm.json** - Basic confirmation dialog
- **settings.json** - Multi-input settings form
- **conditional_settings.json** - Settings with conditional sections
- **choice_demo.json** - Demonstrations of choice widgets

Run any example:
```bash
popup --file examples/settings.json
```

## Use Cases

- **AI Assistant Confirmations** - Get user approval before destructive operations
- **Form Input** - Collect structured data during AI workflows
- **Settings Configuration** - Interactive configuration dialogs
- **Human-in-the-Loop** - Pause AI workflow for user input/decisions
- **Debugging** - Display state or collect debug information
- **Guided Workflows** - Multi-step processes with conditional branches

## Template System

Create reusable popup templates with variables.

**1. Create config** (`~/.config/popup-mcp/popup.toml`):
```toml
[[template]]
name = "confirm_delete"
description = "Confirm file deletion"
file = "confirm_delete.json"

[template.params.filename]
type = "string"
description = "File to delete"
required = true

[template.params.size]
type = "string"
description = "File size"
required = false
```

**2. Create template** (`~/.config/popup-mcp/confirm_delete.json`):
```json
{
  "title": "Delete {{filename}}?",
  "elements": [
    {
      "text": "Permanently delete {{filename}} ({{size}})?",
      "id": "warning"
    }
  ]
}
```

**3. Use from MCP**:
Templates automatically become MCP tools with parameters based on your config.

## Development

### Build

```bash
# Build all crates
cargo build --release

# Build specific crate
cargo build -p popup-gui --release
```

### Test

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_simple_confirmation

# Test JSON parsing
cargo test tests::json_parser_tests
```

### Lint and Format

```bash
cargo fmt --check
cargo fmt
cargo clippy
```

## Architecture

**Local Mode:**
```
MCP Client (Claude Desktop)
  ↓ JSON-RPC over stdio
popup binary (MCP server mode)
  ↓ Spawns subprocess with --stdin
popup-gui subprocess
  ↓ Native egui window
User interaction
  ↓ JSON result to stdout
MCP Client receives result
```

The popup binary operates in multiple modes:
- **MCP server** (`--mcp-server`) - JSON-RPC over stdio for MCP clients
- **Stdin mode** (`--stdin`) - Read JSON from stdin, display popup, write result to stdout
- **File mode** (`--file`) - Read JSON from file, display popup

The MCP server mode spawns itself with `--stdin` for each popup request, providing clean process isolation.

### Crates

- **popup-common** - Shared types (`PopupDefinition`, `PopupResult`, protocol messages)
- **popup-gui** - Native GUI renderer (egui), MCP server, JSON parser, template system
- **popup-client** - WebSocket daemon for remote relay (see Remote Mode below)

## Remote Mode

For distributed deployments where popups appear on remote machines, see **[cloudflare/README.md](cloudflare/README.md)** for:

- Cloudflare Workers deployment
- WebSocket relay with Durable Objects
- GitHub OAuth and Bearer token authentication
- HTTP API for non-MCP integrations
- Client daemon (`popup-client`) setup

Remote mode enables use cases like:
- Triggering popups from cloud-based AI agents
- Multi-user popup relay systems
- Integration with services like Letta

## Contributing

See `CLAUDE.md` for development guidance and architecture details.

## License

MIT
