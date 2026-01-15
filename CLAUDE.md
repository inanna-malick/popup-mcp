# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Popup-MCP: Native GUI Popups via MCP

Popup-MCP is an MCP (Model Context Protocol) server that enables AI assistants to create native GUI popup windows using JSON structure. The project consists of:
- **Rust workspace**: Native GUI rendering and local MCP server

## WSL Compatibility

The popup GUI works in WSL2 with WSLg. The clipboard feature is disabled in `Cargo.toml` to avoid "Broken pipe" errors from smithay-clipboard in WSL environments. This means copy/paste will work within the popup but not with the system clipboard.

**Issue**: eframe's default clipboard integration (via smithay-clipboard) crashes in WSL with `Io error: Broken pipe (os error 32)`
**Solution**: Disable clipboard feature in eframe dependency, falling back to in-app-only clipboard
**Reference**: https://github.com/emilk/egui/issues/4938

## Common Development Commands

### Rust Workspace

```bash
# Build all workspace crates
cargo build --release

# Build specific crate
cargo build -p popup-gui --release

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
echo '{"title": "Test", "elements": [{"text": "Hello"}]}' | cargo run -p popup-gui -- --stdin

# Test with example file
cargo run -p popup-gui -- --file examples/simple_confirm.json
```

## Important Note on Buttons

**Buttons are no longer user-specifiable.** Every popup automatically includes a single "Submit" button at the bottom. Users can press the Submit button to confirm or use the Escape key to cancel. The PopupResult will include `"button": "submit"` or `"button": "cancel"` accordingly.

## High-Level Architecture

### System Overview

The project implements a local popup system with two main components:

1. **popup-common** (`crates/popup-common`): Shared types and condition evaluation
2. **popup-gui** (`crates/popup-gui`): Native egui-based popup renderer and MCP server

### Architecture Flow

```
MCP Client (Claude Desktop)
  ↓ JSON-RPC over stdio
popup binary (MCP server mode)
  ↓ Spawns itself with --stdin
popup binary (renders window)
  ↓ JSON result
MCP Client (receives result)
```

### Rust Workspace Structure

**crates/popup-common** - Shared types
- `PopupDefinition`, `Element`, `PopupResult` - Core data structures
- `condition.rs` - When clause parsing and evaluation
- Serialization via serde for JSON compatibility

**crates/popup-gui** - Native GUI implementation
- `json_parser.rs`: JSON → PopupDefinition deserialization
- `gui/mod.rs`: egui window logic and event loop
- `gui/widget_renderers.rs`: Individual widget rendering
- `mcp_server.rs`: MCP server (JSON-RPC over stdio)
- `schema.rs`: MCP tool schema definitions
- `templates.rs`: Dynamic template system with Handlebars
- Binary: `popup` (operates in 3 modes: MCP server, stdin, file)

### Protocol Flow

**Local (stdio MCP server):**
1. **MCP Request**: Client → JSON-RPC via stdin
2. **Spawn Renderer**: MCP server spawns `popup --stdin` subprocess
3. **Render**: Subprocess renders window, exits with JSON result
4. **Result**: MCP server reads stdout, returns to client via JSON-RPC

### Key Design Decisions

- **Subprocess isolation**: Each popup runs in separate process, clean lifecycle
- **JSON-based structure**: Clean, explicit definition with no parsing ambiguities
- **Type safety**: JSON schema provides clear structure validation
- **Nested support**: Natural support for conditionals and groups through JSON nesting
- **Self-spawning architecture**: MCP server spawns itself with --stdin for rendering
- **Template-driven tools**: Dynamic MCP tool generation from user config files

### Testing Strategy

**Rust Tests** (`crates/popup-gui/src/tests/`):
- `json_parser_tests.rs`: Core JSON parsing tests for all widget types
- `integration_tests.rs`: Integration tests with example files and state management
- `conditional_filtering_tests.rs`: Conditional visibility logic
- `template_tests.rs`: Template system tests

**Example JSON files** (`examples/`):
- `simple_confirm.json`: Basic confirmation dialog
- `settings.json`: Complex settings form
- `conditional_settings.json`: Settings with conditional visibility
- `choice_demo.json`: Choice widget demonstrations

## JSON Structure Reference (V2 Schema)

**Key Concepts:**
- **Element-as-key**: Widget type is the JSON key (e.g., `{"slider": "Volume"}` not `{"type": "slider"}`)
- **Required IDs**: All interactive elements must have an `"id"` field for state management
- **When clauses**: Replace standalone Conditional elements with `"when"` field on any element
- **Reveals**: Inline conditionals on checkboxes/multiselect/choice using `"reveals"` field
- **Option-as-key nesting**: Choice/Multi children use option text as direct JSON keys

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
{
  "text": "Display text",
  "id": "message"  // Optional for text elements
}
```

#### Slider
```json
{
  "slider": "Volume",
  "id": "volume",
  "min": 0,
  "max": 100,
  "default": 50  // Optional, defaults to midpoint
}
```

#### Check
```json
{
  "check": "Enable feature",
  "id": "enable_feature",
  "default": true  // Optional, defaults to false
}
```

With reveals (shown when checked):
```json
{
  "check": "Enable advanced",
  "id": "enable_advanced",
  "default": false,
  "reveals": [
    {
      "slider": "Advanced level",
      "id": "advanced_level",
      "min": 1,
      "max": 10
    }
  ]
}
```

#### Input
```json
{
  "input": "Name",
  "id": "user_name",
  "placeholder": "Enter your name",  // Optional
  "rows": 5  // Optional, for multiline
}
```

#### Choice (Single Selection)
```json
{
  "select": "Theme",
  "id": "theme",
  "options": ["Light", "Dark", "Auto"]
}
```

With per-option children (option-as-key nesting):
```json
{
  "select": "Mode",
  "id": "mode",
  "options": ["Simple", "Advanced"],
  "Advanced": [
    {
      "slider": "Complexity",
      "id": "complexity",
      "min": 1,
      "max": 10
    }
  ]
}
```

#### Multi
```json
{
  "multi": "Features",
  "id": "features",
  "options": ["Feature A", "Feature B", "Feature C"]
}
```

With per-option children (option-as-key nesting):
```json
{
  "multi": "Features",
  "id": "features",
  "options": ["Basic", "Advanced", "Expert"],
  "Advanced": [
    {
      "slider": "Advanced Level",
      "id": "advanced_level",
      "min": 1,
      "max": 5
    }
  ],
  "Expert": [
    {
      "input": "Expert Config",
      "id": "expert_config",
      "placeholder": "Enter configuration"
    }
  ]
}
```

#### Group
```json
{
  "group": "Settings",
  "elements": [
    // Nested elements
  ]
}
```

#### When Clauses (Conditional Visibility)

Any element can have a `"when"` field for conditional visibility:

```json
{
  "slider": "Debug level",
  "id": "debug_level",
  "min": 0,
  "max": 10,
  "when": "show_advanced"  // Simple boolean check
}
```

**When Clause Syntax:**
- `@id` - Boolean check (true if checkbox checked, multiselect has selections, etc.)
- `selected(id, value)` - Check if specific value is selected
- `count(@id) > 2` - Check selection count with operators: `>`, `<`, `>=`, `<=`, `==`
- `@id1 && @id2` - Logical AND
- `@id1 || @id2` - Logical OR
- `!@id` - Logical NOT

**Complex When Examples:**
```json
{
  "text": "Advanced mode active",
  "when": "enable_advanced && selected(mode, Pro)"
}
```

```json
{
  "text": "Many items selected",
  "when": "count(features) >= 3"
}
```

## Example Popups

### Simple Confirmation
```json
{
  "title": "Delete File?",
  "elements": [
    {
      "text": "This action cannot be undone.",
      "id": "warning"
    }
  ]
}
```

### Settings Form
```json
{
  "title": "Settings",
  "elements": [
    {
      "slider": "Volume",
      "id": "volume",
      "min": 0,
      "max": 100,
      "default": 75
    },
    {
      "check": "Notifications",
      "id": "notifications",
      "default": true
    },
    {
      "select": "Theme",
      "id": "theme",
      "options": ["Light", "Dark", "Auto"]
    }
  ]
}
```

### Conditional UI (When Clauses)
```json
{
  "title": "Advanced Settings",
  "elements": [
    {
      "check": "Show advanced",
      "id": "show_advanced",
      "default": false
    },
    {
      "slider": "Debug level",
      "id": "debug_level",
      "min": 0,
      "max": 10,
      "when": "show_advanced"
    },
    {
      "input": "Log file",
      "id": "log_file",
      "placeholder": "/tmp/debug.log",
      "when": "show_advanced"
    }
  ]
}
```

### Reveals (Inline Conditionals)
```json
{
  "title": "Feature Configuration",
  "elements": [
    {
      "check": "Enable advanced mode",
      "id": "enable_advanced",
      "default": false,
      "reveals": [
        {
          "slider": "Complexity",
          "id": "complexity",
          "min": 1,
          "max": 10
        }
      ]
    },
    {
      "select": "Profile",
      "id": "profile",
      "options": ["Basic", "Pro"],
      "Pro": [
        {
          "input": "License Key",
          "id": "license_key",
          "placeholder": "XXXX-XXXX-XXXX"
        }
      ]
    }
  ]
}
```

### Complex When Expressions
```json
{
  "title": "Multi-Conditional Example",
  "elements": [
    {
      "check": "Debug mode",
      "id": "debug",
      "default": false
    },
    {
      "multi": "Features",
      "id": "features",
      "options": ["Analytics", "Sync", "Backup"]
    },
    {
      "text": "Debug mode active with multiple features",
      "when": "debug && count(features) > 1"
    },
    {
      "select": "Mode",
      "id": "mode",
      "options": ["Simple", "Advanced", "Expert"]
    },
    {
      "slider": "Expert complexity",
      "id": "expert_complexity",
      "min": 1,
      "max": 10,
      "when": "selected(mode, Expert)"
    }
  ]
}
```

## Claude Desktop Configuration

Add to Claude Desktop config (`~/Library/Application Support/Claude/claude_desktop_config.json` on macOS):

```json
{
  "mcpServers": {
    "popup": {
      "command": "popup"
    }
  }
}
```

Note: MCP server mode is the default when no arguments are provided.

Restart Claude Desktop and the `popup` tool will be available.

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
    {
      "text": "Permanently delete {{item_name}}?",
      "id": "warning"
    }
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