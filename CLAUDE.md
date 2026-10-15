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

Every popup has exactly one submit button at the bottom; its label defaults to "Submit" and can be set with the top-level `"submit"` field (name it after the consequence, e.g. `"submit": "Deploy"`). Pressing it returns `{"status": "completed", "button": "submit", ...values}`. Escape cancels and returns `{"status": "cancelled"}` with no values.

Popups also include a collapsed "Add a note (optional)" free-text field above the submit button (disable with `"notes": false`). If the user writes anything there it comes back as `"_notes"` in the result — treat it as extra instructions the form didn't anticipate.

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
- `element_deser.rs` - Strict element-as-key (de)serialization with "did you mean" errors
- `condition.rs` + `condition.pest` - When clause parsing and evaluation
- `validate.rs` - Pre-flight structural validation (duplicate/reserved ids, bad when-refs, ranges)

**crates/popup-gui** - Native GUI implementation
- `json_parser.rs`: JSON → PopupDefinition parsing + validation entry points
- `gui/mod.rs`: egui window logic, event loop, and all widget rendering
- `transform.rs`: automatic "Other (please specify)" injection for select/multi
- `mcp_server.rs`: MCP server (JSON-RPC over stdio), validates before rendering
- `schema.rs`: MCP tool schema + the tool description that teaches tree authoring
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

## JSON Structure Reference

**Key Concepts:**
- **Element-as-key**: Widget type is the JSON key (e.g., `{"slider": "Volume"}` not `{"type": "slider"}`) and its value is the human-facing label
- **Auto IDs**: `id` is derived from the label when omitted ("Enable Feature" → `enable_feature`); explicit ids are more reliable for `when` clauses
- **Option-as-key branching**: an option's text used as a direct JSON key on select/multi maps to follow-up elements shown when that option is chosen
- **Reveals**: elements shown while a check is checked (or once a select/multi has any selection)
- **When clauses**: cross-element visibility conditions on any element
- **Strict validation**: unknown keys, typo'd branch keys, duplicate/reserved ids, and broken when clauses are rejected before rendering, with "did you mean" suggestions

### Basic Structure
```json
{
  "title": "Short sentence describing what this interaction decides",
  "submit": "Deploy",          // optional submit-button label (default "Submit")
  "notes": true,                // optional; false hides the "Add a note" field
  "elements": [ ... ]
}
```

### Element Types

#### Text / Markdown
```json
{"text": "Display text"}
{"markdown": "**Context:** framing for the questions below"}
```

#### Slider
```json
{
  "slider": "Time budget (hours)",
  "id": "budget",
  "min": 1,
  "max": 8,
  "step": 1,        // optional, default 1; whole steps return integers
  "default": 3      // optional, defaults to midpoint
}
```

#### Check
```json
{
  "check": "Set up CI now",
  "id": "ci",
  "default": true,
  "reveals": [      // shown while checked
    {"multi": "CI jobs", "id": "ci_jobs", "options": ["test", "lint", "release"]}
  ]
}
```

#### Input
```json
{
  "input": "Service name",
  "id": "name",
  "placeholder": "my-service",
  "default": "payments-webhook",  // prefill with your best guess (accept-or-correct)
  "rows": 3                        // optional, multiline
}
```

#### Select (Single Choice)
```json
{
  "select": "Fix strategy",
  "id": "strategy",
  "options": [
    {"value": "Patch fixtures", "description": "~20 min, fixtures keep drifting"},
    {"value": "Regenerate from sandbox", "description": "~1 hour, matches prod exactly"}
  ],
  "default": "Regenerate from sandbox",   // preselect your recommendation
  "style": "radio",                        // optional: "radio" | "dropdown" (auto: radio ≤6 options)
  "Regenerate from sandbox": [             // option-as-key branch
    {"input": "Sandbox key", "id": "sandbox_key"}
  ]
}
```
Options may be plain strings or `{value, description}` objects; descriptions render under each radio option. Selects with ≤6 options render as radio lists so users see every branch; longer lists become dropdowns.

#### Multi (Multiple Choice)
```json
{
  "multi": "Features",
  "id": "features",
  "options": ["Basic", "Advanced"],
  "default": ["Basic"],                    // option texts pre-checked on open
  "Advanced": [                            // shown while "Advanced" is checked
    {"slider": "Level", "id": "level", "min": 1, "max": 5}
  ]
}
```

#### Group
```json
{"group": "Deployment", "elements": [ ... ]}
```

#### Page (Wizard Mode)

Top-level only; when used, every top-level element must be a page. One page shows at a time with Back/Next, and the submit button appears on the last page. (Pages with `next` edges become a decision tree instead, which only goes forward and has no Back.) A page's own `when` skips it dynamically based on earlier answers.

```json
{
  "title": "Set up the new project",
  "elements": [
    {"page": "Basics", "elements": [ ... ]},
    {"page": "Web details", "when": "selected(kind, Web service)", "elements": [ ... ]},
    {"page": "Tooling", "elements": [ ... ]}
  ]
}
```

See `examples/project_setup_wizard.json`.

#### Decision trees (`next` edges on pages)

Give a page `next` edges and the pages stop being a sequence and become a graph. Each edge names a target and the condition for taking it, described in words about that page's answers:

```json
{"page": "What's happening", "id": "symptom", "elements": [ ... ],
 "next": [
   {"to": "rollback", "when": "recent_deploy && selected(user_impact, Checkout fails outright)"},
   {"to": "outage",   "if": "users cannot complete a payment at all"},
   {"to": "degraded", "if": "payments still go through but are slower than usual"}
 ]}
```

- `to` is another top-level page's id (auto-derived from its label when omitted) or `"submit"` to finish. A page with no `next` is terminal.
- An edge carries exactly one of `if` (judged semantically from the answers and the page note) or `when` (the ordinary condition syntax). `when` edges are evaluated first, in code, and taken silently.
- Write `if` as the **condition** that makes the branch apply, not the action it triggers. Condition-described options measure substantially better than action-described ones.
- The rest of the judgment packet is **compiled from the tree**, so the authoring model writes only the one line: each option carries the destination page's questions (so the branch is judged on where it leads), the two exits are supplied, and earlier pages are capped to recent context. See `popup-common/src/route.rs`.
- **Routing only goes forward.** One Jev call (TypeSafe System One) per hop decides the edge and the next page opens by itself: no chooser, no Back, no step counter. The page says why it opened. Every page carries a "None of these" button, and that plus the note field is the user's way out.
- One packet carries four things over the same state: the route Choice, a Noul per edge ("does this condition hold on its own"), a contradiction Noul for the note, and prefill Choices for the destinations' selects. Extra questions in one packet are nearly free; extra calls are not.
- The outcome order: an exit that won hands back; otherwise the Choice acts if it clears confidence, then mass, then margin against the exits **summed**; otherwise exactly one edge Noul at 0.7+ acts; several at 0.7+ open the first and queue the rest; nothing holding hands back with `leaning`.
- An answer travels with its option's description, not just the text. Measured: the same pick went from 0.16 to 0.93 on the right branch once the description rode along, because the description is the part that says what the option means.
- Routing needs a key in `TYPESAFE_API_KEY` or `~/.config/typesafe/jev-key.env`. Without one, or if the call fails, the popup hands back with `exit_reason: "unavailable"` rather than asking the user to route by hand. Calls are logged to `~/.config/popup-mcp/jev-ledger.jsonl`.
- Only pages actually visited contribute values to the result.

See `examples/incident_triage_tree.json`.

### Auto-Injected "Other" Option

Every select/multi automatically receives an "Other (please specify)" option with an attached text input. Never add one manually. When chosen, the result contains the selection plus `"<id>_other_text"` with the custom text.

### When Clauses (Conditional Visibility)

Any element can have a `"when"` field:

```json
{"slider": "Debug level", "id": "debug_level", "min": 0, "max": 10, "when": "show_advanced"}
```

**Syntax:**
- `some_id` — truthy check (checkbox checked, multi has selections, input non-empty). A legacy `@some_id` prefix is also accepted
- `selected(id, value)` — specific option selected; the value may be an unquoted phrase (`selected(strategy, Regenerate from sandbox)`); quote it only if it contains a closing paren
- `count(id) > 2` — selection count, with `>`, `<`, `>=`, `<=`, `==`, `!=`
- Numeric comparisons against sliders: `severity >= 7`, negative numbers allowed (`lufs > -9`)
- Combine with `&&`, `||`, `!`, and parentheses; `any(...)`/`all(...)` also work

When clauses are validated up front: parse errors, references to unknown ids, `selected()` values that aren't options of the target, and `selected()` on non-select elements are all returned as errors before the popup renders.

**Visibility scoping:** conditions only see visible elements' values. An id hidden inside an untaken branch reads as unset (falsy, unselected, count 0) even if it has a default, so cross-branch references like `"when": "selected(creds, \"I'll provide fresh keys\")"` are safe without repeating the ancestor branch's condition.

### Result Format

```json
{"status": "completed", "button": "submit", "strategy": "Regenerate from sandbox", "budget": 3, ...}
{"status": "cancelled"}
```

Value semantics (deliberate signals, not noise):
- **absent key** — element was never shown (its branch wasn't taken)
- **`""` for input** — shown but left blank; **`null` for select** — shown, nothing chosen; **`[]` for multi** — shown, nothing checked; **`false` for check** — shown, unchecked
- select returns option text; multi returns an array of option texts
- slider returns a number (integer when whole)
- `"_notes"` — free-form note the user added via the always-available note field
- `"_labels"` — map of returned ids to their human-facing labels (self-documenting results)
- `"_route"` (decision trees only) — the path taken, one entry per hop: `{"from", "to", "via", "matched", "confidence", "masses", "holds", "prefilled", "note_overrides_answers"}`. `via` is `when` (mechanical), `jev`/`noul` (judged), `queue` (a branch that also matched), `user` or `error`. `several` lists branches that matched alongside this one. A last hop with `"to": "none_of_these"` means the popup handed back; `exit_reason` is `none_fit`, `unclear`, `low_confidence` (with `leaning`) or `unavailable`.
- Reserved ids (cannot be used by elements): `status`, `button`, `error`, `_notes`, `_labels`, `_route`

## Example Popups

### Branching Interview (the core pattern)

Anticipate answers; nest the follow-ups each answer would trigger:

```json
{
  "title": "How should I handle the failing payment-webhook tests?",
  "submit": "Go with this plan",
  "elements": [
    {"markdown": "**Context:** 4 tests fail after the Stripe API version bump."},
    {"select": "Fix strategy", "id": "strategy",
     "options": [
       {"value": "Patch fixtures", "description": "~20 min, drift remains"},
       {"value": "Regenerate from sandbox", "description": "~1 hour, drift removed"}
     ],
     "default": "Regenerate from sandbox",
     "Patch fixtures": [
       {"check": "Add a schema-drift test", "id": "drift_test", "default": true}
     ],
     "Regenerate from sandbox": [
       {"check": "Also regenerate refund fixtures", "id": "refunds", "reveals": [
         {"slider": "Extra time (min)", "id": "extra", "min": 10, "max": 60, "step": 10}
       ]}
     ]},
    {"text": "Note: sandbox keys required", "when": "selected(strategy, Regenerate from sandbox)"}
  ]
}
```

See `examples/fix_strategy_interview.json` for the full version, and the other files in `examples/` for more patterns (settings forms, conditional UI, count-based conditions).

## Claude Desktop Configuration

Add to Claude Desktop config (`~/Library/Application Support/Claude/claude_desktop_config.json` on macOS):

```json
{
  "mcpServers": {
    "popup": {
      "command": "/Users/you/.cargo/bin/popup"
    }
  }
}
```

MCP server mode is the binary's default (no flags). Use the absolute path: GUI apps don't inherit your shell's PATH.

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