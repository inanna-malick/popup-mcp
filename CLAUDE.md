# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Popup-MCP: Native GUI Popups via MCP

Popup-MCP is an MCP (Model Context Protocol) server that enables AI assistants to create native GUI popup windows using JSON structure.

## Common Development Commands

```bash
# Build the project
cargo build --release

# Run all tests
cargo test

# Run tests with output (useful for debugging) 
cargo test -- --nocapture

# Run a specific test
cargo test test_simple_confirmation

# Run tests in a specific module
cargo test tests::json_parser_tests

# Build and install locally
cargo install --path .

# Run formatting check
cargo fmt --check

# Apply formatting
cargo fmt

# Run linter
cargo clippy

# Test popup directly from command line with JSON
echo '{"title": "Test", "elements": [{"type": "text", "content": "Hello"}]}' | cargo run -- --test

# Test with example file
cargo run -- --test examples/simple_confirm.json

# Run MCP server (default mode)
cargo run

# List available templates
cargo run -- --list-templates

# Run with template filtering
cargo run -- --include-only settings,feedback
cargo run -- --exclude debug,admin
```

## Important Note on Buttons

**Buttons are no longer user-specifiable.** Every popup automatically includes a single "Submit" button at the bottom. Users can press the Submit button to confirm or use the Escape key to cancel. The PopupResult will include `"button": "submit"` or `"button": "cancel"` accordingly.

## High-Level Architecture

### Core Components

1. **JSON Parser** (`src/json_parser.rs`)
   - Direct deserialization from JSON to `PopupDefinition`
   - Clean, explicit structure with no ambiguity
   - Supports nested conditionals and groups naturally

2. **GUI Renderer** (`src/gui/`)
   - `mod.rs`: Main popup window logic using egui framework
   - `widget_renderers.rs`: Individual widget rendering implementations
   - Native GUI popups that return structured JSON results

3. **MCP Server** (`src/mcp_server.rs`)
   - Implements Model Context Protocol for AI assistant integration
   - Handles JSON-RPC communication with Claude Desktop
   - Provides `popup` tool for creating GUI popups from JSON
   - Main entry point is `src/main.rs` which runs as MCP server by default

4. **Models** (`src/models.rs`)
   - Core data structures: `PopupDefinition`, `Element`, `PopupResult`
   - All types have Serialize/Deserialize for JSON compatibility
   - Supports various widget types: Slider, Checkbox, Choice, Multiselect, Textbox, Buttons, Group, Conditional

5. **Template System** (`src/templates.rs`)
   - Predefined popup templates for common use cases
   - Can be filtered with `--include-only` or `--exclude` flags
   - Templates loaded from configuration files

### Key Design Decisions

- **JSON-based structure**: Clean, explicit definition with no parsing ambiguities
- **Type safety**: JSON schema provides clear structure validation
- **Nested support**: Natural support for conditionals and groups through JSON nesting
- **Simple implementation**: No complex parsing logic, just JSON deserialization

### Testing Strategy

Tests are organized in `src/tests/`:
- `json_parser_tests.rs`: Core JSON parsing tests for all widget types
- `integration_tests.rs`: Integration tests with example files and state management
- `template_tests.rs`: Template system tests

Example JSON files for testing in `examples/`:
- `simple_confirm.json`: Basic confirmation dialog
- `settings.json`: Complex settings form
- `conditional_settings.json`: Settings with conditional visibility
- `feedback_form.json`: User feedback collection
- `system_status.json`: System status display
- `user_profile.json`: User profile form

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

## Development Principles

- **ALWAYS write unit tests, not main methods**. No main methods unless explicitly requested.
- Use the existing test patterns in `src/tests/` as examples
- Prefer iterators and for loops over manual iteration in Rust
- Avoid early optimizations without benchmarks
- **Wherever possible, write unit tests instead of using cargo run to test changes**