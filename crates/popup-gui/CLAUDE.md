# popup-gui

Native GUI popup renderer using egui framework. Renders JSON-defined popups as native windows.

## Purpose

Provides the visual rendering layer for popup-mcp:
- Standalone binary accepting JSON via stdin or file
- Widget rendering with state management
- Result serialization to JSON on exit
- Template system for common patterns
- MCP server integration (stdio protocol)

## Architecture

### Binary Modes

The `popup` binary operates in three modes based on CLI flags:

**1. MCP Server Mode (default - no flags)**
- JSON-RPC over stdio (implemented in mcp_server.rs)
- Provides `popup` tool (raw JSON DSL) + dynamic template tools
- Loads templates from `~/.config/popup-mcp/popup.toml`
- Each template becomes its own MCP tool with typed parameters
- Spawns itself with `--stdin` flag to render popups

**2. Stdin Mode (`--stdin`)**
- Reads PopupDefinition JSON from stdin
- Renders native window via egui
- Prints PopupResult JSON to stdout on exit
- Used by MCP server and popup-client for actual rendering

**3. File Mode (`--file <path>`)**
- Reads PopupDefinition JSON from file
- Renders and prints result (same as stdin mode)
- Useful for testing and standalone usage

### Core Modules

**json_parser.rs**
- Deserializes PopupDefinition from JSON
- Validates structure and types
- Entry point: `parse_popup_definition(json_str) -> Result<PopupDefinition>`

**gui/mod.rs** - Main rendering logic
- `PopupWindow` - egui application state
- Event loop with conditional filtering
- Submit/Cancel button handling
- Escape key for cancel
- Returns PopupResult on close

**gui/widget_renderers.rs** - Individual widget implementations
- `render_element(ui, element, state, active_labels) -> bool`
- Returns true if widget value changed (triggers UI update)
- Handles conditional visibility via active_labels set
- Widget-specific rendering:
  - Text: `ui.label()`
  - Slider: `ui.add(Slider::new())`
  - Checkbox: `ui.checkbox()`
  - Textbox: `ui.text_edit_singleline()` or `TextEdit::multiline()`
  - Choice: `ComboBox::from_label()`
  - Multiselect: Vertical list of checkboxes
  - Group: Collapsing header with nested elements
  - Conditional: Delegates to child elements if condition true

**templates.rs** - Dynamic template system
- Loads from `~/.config/popup-mcp/popup.toml` config file
- Each template defines: name, description, file path, parameters
- Uses Handlebars for variable substitution in JSON templates
- `load_templates()` - Discovers and loads all configured templates
- `generate_tool_schema()` - Creates MCP tool schema from template config
- Templates become first-class MCP tools with typed parameters

**theme.rs** - Visual styling
- Dark theme configuration
- Visuals, spacing, colors
- Applied on egui context creation

**schema.rs** - MCP tool schemas
- JSON Schema definitions for MCP tools
- Defines parameters and return types
- Used by MCP clients for validation

### Testing

**tests/json_parser_tests.rs** - Core JSON parsing
- Widget type parsing tests
- Invalid JSON handling
- Edge cases (missing fields, wrong types)

**tests/integration_tests.rs** - End-to-end flows
- Example file parsing (simple_confirm.json, settings.json, etc.)
- State initialization tests
- Conditional filtering logic

**tests/conditional_filtering_tests.rs** - Conditional visibility
- Simple conditions (checkbox labels)
- Field conditions (value matching)
- Count conditions (comparison operators)
- Nested conditionals

**tests/template_tests.rs** - Template system
- Template retrieval tests
- Template validation tests

## Key Patterns

### Conditional Rendering
1. Parse definition into elements tree
2. Initialize PopupState from defaults
3. Each frame: compute active_labels set based on current state
4. Pass active_labels to render_element - only render if label in set
5. Update triggers full recompute of active_labels

### State Management
- Central PopupState in PopupWindow
- Widgets mutate via `state.get_*_mut(label)`
- Changes detected via return value from render_element
- On change: recompute conditionals, mark UI dirty

### Result Construction
- Submit: `PopupResult::from_state_with_active_elements()`
- Cancel/Escape: `PopupResult::Cancelled`
- Includes only visible elements in result

## Common Commands

```bash
# Build and install the binary
cargo install --path crates/popup-gui

# Run with example file
cargo run -p popup-gui -- --file examples/simple_confirm.json

# Run with stdin
echo '{"title":"Test","elements":[{"text":"Hello"}]}' | cargo run -p popup-gui -- --stdin

# Run MCP server mode (default - uses stdio for JSON-RPC)
cargo run -p popup-gui

# List available templates (requires ~/.config/popup-mcp/popup.toml)
cargo run -p popup-gui -- --list-templates

# Filter templates in MCP server
cargo run -p popup-gui -- --include-only confirm_delete,feedback
cargo run -p popup-gui -- --exclude experimental_tool

# Test all
cargo test -p popup-gui

# Test specific module
cargo test -p popup-gui tests::json_parser_tests

# Test with output
cargo test -p popup-gui -- --nocapture
```

## Dependencies

- `eframe` / `egui` - Native GUI framework
- `serde_json` - JSON parsing
- `popup-common` - Shared types
- `clap` - CLI argument parsing
- `anyhow` - Error handling

## File Organization

```
src/
├── main.rs              # CLI entry point
├── lib.rs               # Library root
├── mcp_server.rs        # MCP JSON-RPC server
├── json_parser.rs       # JSON → PopupDefinition
├── templates.rs         # Named template definitions
├── schema.rs            # MCP tool schemas
├── theme.rs             # Visual styling
├── gui/
│   ├── mod.rs           # PopupWindow + event loop
│   └── widget_renderers.rs  # Individual widget rendering
└── tests/
    ├── json_parser_tests.rs
    ├── integration_tests.rs
    ├── conditional_filtering_tests.rs
    └── template_tests.rs
```

## Template System

**Config location:** `~/.config/popup-mcp/popup.toml`

**Example popup.toml:**
```toml
[[template]]
name = "confirm_delete"
description = "Confirm destructive action with item name"
file = "confirm_delete.json"

[template.params.item_name]
type = "string"
description = "Name of item to delete"
required = true

[[template]]
name = "quick_settings"
description = "Quick settings dialog"
file = "quick_settings.json"
# No parameters - static template
```

**Template JSON files** use Handlebars syntax:
```json
{
  "title": "Delete {{item_name}}?",
  "elements": [
    {
      "text": "This will permanently delete {{item_name}}.",
      "id": "warning"
    }
  ]
}
```

**MCP Integration:**
- Each template becomes an MCP tool named after template.name
- Parameters defined in popup.toml become tool input schema
- Tool invocation → Handlebars substitution → popup rendering

## Design Principles

- **No parsing ambiguity** - JSON structure is explicit and typed
- **Type-safe state access** - get_*_mut() returns Option<&mut T>
- **Conditional reevaluation** - Full recompute on each state change
- **Minimal UI framework usage** - Straightforward egui patterns
- **Test-driven** - Unit tests for parsing, integration tests for rendering
- **No main() test methods** - Use unit tests, not cargo run for validation
- **Self-spawning architecture** - MCP server spawns itself with --stdin for rendering
- **Template-driven tools** - Dynamic MCP tool generation from user configs
