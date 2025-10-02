# popup-common

Shared data structures and protocol definitions for the popup-mcp system.

## Purpose

Provides core types and protocol definitions shared across the entire popup-mcp architecture:
- **popup-gui** - Native renderer consumes PopupDefinition, produces PopupResult
- **popup-client** - WebSocket daemon uses protocol messages to coordinate with relay
- **Cloudflare relay** - Durable Object uses protocol for client communication
- **TypeScript mirror** - cloudflare/src/protocol.ts maintains wire format compatibility

## Core Types

### PopupDefinition
JSON-serializable dialog definition with optional title and element list.

```rust
pub struct PopupDefinition {
    pub title: Option<String>,
    pub elements: Vec<Element>,
}
```

### Element
Tagged enum of widget types. Supports nesting via Group and Conditional variants.

**Variants:**
- `Text { content: String }` - Static text display
- `Slider { label, min, max, default? }` - Numeric input (f32)
- `Checkbox { label, default? }` - Boolean input
- `Textbox { label, placeholder?, rows? }` - Text input (single/multi-line)
- `Multiselect { label, options }` - Multiple choice checkboxes
- `Choice { label, options, default? }` - Single selection dropdown
- `Group { label, elements }` - Labeled container
- `Conditional { condition, elements }` - Conditional visibility

### Condition
Controls conditional element visibility.

**Patterns:**
- `Simple(String)` - Label check: true if checkbox checked or multiselect has selection
- `Field { field, value }` - Value check: true if field equals value
- `Count { field, count }` - Quantity check: parses expressions like `">2"`, `"<=5"`

**ComparisonOp:** Parses `>`, `<`, `>=`, `<=`, `=` from count strings.

### PopupState
Runtime state management for widget values. Uses HashMap keyed by label.

```rust
pub struct PopupState {
    pub values: HashMap<String, ElementValue>,
    pub button_clicked: Option<String>,
}
```

**ElementValue variants:** Number, Boolean, Text, MultiChoice(Vec<bool>), Choice(Option<usize>)

**Methods:**
- `new(definition)` - Initialize from definition with defaults
- `get_*_mut(label)` - Type-safe mutable access (returns Option)
- `get_*(label)` - Immutable access for conditionals

### PopupResult
Serialized user interaction result.

**Variants:**
- `Completed { values, button }` - User submitted with values
- `Cancelled` - User closed/cancelled
- `Timeout { message }` - No response within timeout

**Construction methods:**
- `from_state(state)` - Basic serialization (indices for multiselect/choice)
- `from_state_with_context(state, definition)` - Rich format (slider "50/100", option texts)
- `from_state_with_active_elements(state, definition, active_labels)` - Filtered by visibility

## Protocol

### ServerMessage
Cloudflare DO → popup-client

```rust
enum ServerMessage {
    ShowPopup { id: String, definition: PopupDefinition, timeout_ms: u64 },
    ClosePopup { id: String },
    Ping,
}
```

### ClientMessage
popup-client → Cloudflare DO

```rust
enum ClientMessage {
    Ready { device_name: Option<String> },
    Result { id: String, result: PopupResult },
    Pong,
}
```

Messages use `#[serde(tag = "type", rename_all = "snake_case")]` for clean JSON.

## Example JSON

```json
{
  "title": "Settings",
  "elements": [
    { "type": "text", "content": "Configure options" },
    { "type": "checkbox", "label": "Enable feature", "default": true },
    {
      "type": "conditional",
      "condition": "Enable feature",
      "elements": [
        { "type": "slider", "label": "Level", "min": 0, "max": 100 }
      ]
    }
  ]
}
```

## Dependencies

- `serde`, `serde_json` - Serialization only
- No platform-specific code
- Pure data structures, no I/O or GUI dependencies

## Design Notes

**Why separate crate:**
- Shared types eliminate duplication between Rust crates
- TypeScript can mirror protocol types for wire compatibility
- Clean separation between data model and rendering/networking logic
- Enables protocol evolution without touching GUI or network code

**Serialization strategy:**
- Tagged enums with `#[serde(tag = "type", rename_all = "snake_case")]` for clean JSON
- Optional fields use `#[serde(default)]` for forward compatibility
- Result variants cover all possible popup outcomes (completed/cancelled/timeout)
