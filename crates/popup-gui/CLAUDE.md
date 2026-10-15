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
- Validates every definition BEFORE spawning a window; validation failures
  return `isError: true` with per-element fixes ("did you mean ..."), so the
  calling model can retry without the user ever seeing a broken popup
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
- `parse_popup_json(json_str) -> Result<PopupDefinition>` - parse only
- `parse_and_validate_popup_json(json_str)` - parse + `validate_definition` with
  aggregated, fixable error messages (used by --stdin/--file modes)

**gui/mod.rs** - All window + widget rendering
- `PopupApp` - egui application state (definition, PopupState, theme, note text)
- Single-column layout in author order; consecutive simple checkboxes wrap horizontally
- Select rendering: radio list with inline option descriptions when <=6 options
  (or `"style": "radio"`), dropdown otherwise
- Sliders honor `step` (default 1) and display integers for whole steps
- Bottom panel: collapsed "Add a note" field (unless `notes: false`) + submit
  button labeled from `definition.submit_label()`
- Escape -> `PopupResult::Cancelled`; submit -> `PopupResult::from_state(state, definition, active_ids)`
- Three navigation modes: single screen, linear wizard (pages without `next`),
  and decision tree (`GraphNav`: any page with `next`).
- Graph mode is **forward-only**: no Back, no step counter, no chooser. Next
  builds the packet (the user's commit point) and `resolve_route` runs the call
  on a worker thread; `poll_route` folds the answer in and acts at once, so the
  next page opens by itself. `arrived` is the dim line under the page heading
  saying which condition was matched, and `inferred` marks widgets prefilled
  from what the user wrote. `queue` holds branches that also matched, walked
  when the current branch runs out. Every other path calls `hand_back`, which
  logs why and finishes - including no key and a failed call.
- `route_nav::with_descriptions` sends an option's description alongside its
  text. Without it the state loses the words the user actually read, which is
  worth a measured 0.16 vs 0.93 on the right branch.

**templates.rs** - Dynamic template system
- Loads from `~/.config/popup-mcp/popup.toml` config file
- Each template defines: name, description, file path, parameters
- Uses Handlebars for variable substitution in JSON templates
- `load_templates()` - Discovers and loads all configured templates
- `generate_tool_schema()` - Creates MCP tool schema from template config
- Templates become first-class MCP tools with typed parameters

**jev.rs** - Jev (TypeSafe System One) client for decision-tree routing
- Blocking POST per routing decision, retries on 429/503/529
- Key from `TYPESAFE_API_KEY` or `~/.config/typesafe/jev-key.env`; `from_env()`
  returns `None` when unset so routing degrades to unaided user choice
- Every call logged to `~/.config/popup-mcp/jev-ledger.jsonl`
- `live_routing_picks_the_described_edge` is `#[ignore]`d; run with
  `cargo test -p popup-gui -- --ignored` when a key is present

**gui/route_nav.rs** - Graph-mode navigation helpers (pure, no window)
- `is_graph`, `page_labels`, `page_index`, `ids_in_page`
- `build_path` - the visited pages as Jev sees them (answers keyed by label)
- `visited_active_ids` - unvisited pages' defaults never reach the result
- `with_route_log` - attaches `_route` to a completed result

**theme.rs** - Visual styling
- Dark theme configuration
- Visuals, spacing, colors
- Applied on egui context creation

**schema.rs** - MCP tool schema + description
- The tool description is effectively a prompt teaching the calling model to
  author conversation trees; keep its result-format claims in sync with
  `PopupResult::from_state`
- Schema examples are tested: they must parse and validate (`test_schema_examples_are_valid_definitions`)

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
- Submit: `PopupResult::from_state(state, definition, active_ids)` - only visible
  elements; absent = branch not taken, "" = blank input, null = unselected select
- Cancel/Escape: `PopupResult::Cancelled`
- User note (bottom panel) returned as `_notes` when non-empty

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
