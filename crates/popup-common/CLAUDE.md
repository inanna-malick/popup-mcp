# popup-common

Shared data structures and protocol definitions for the popup-mcp system.

## Purpose

Provides core types shared across the popup-mcp architecture:
- **popup-gui** - Native renderer consumes PopupDefinition, produces PopupResult
- Condition evaluation for when clauses

## Core Types

### PopupDefinition
JSON-serializable dialog definition with optional title and element list.

```rust
pub struct PopupDefinition {
    pub title: Option<String>,
    pub elements: Vec<Element>,
}
```

### Element (V2 Schema)
Tagged enum of widget types using element-as-key deserialization. All interactive elements require an `id` field.

**Variants:**
- `Text { text, id?, when? }` - Static text display (ID optional)
- `Slider { slider, id, min, max, default?, when?, reveals? }` - Numeric input (f32)
- `Checkbox { checkbox, id, default?, when?, reveals? }` - Boolean input with optional reveals
- `Textbox { textbox, id, placeholder?, rows?, when? }` - Text input (single/multi-line)
- `Multiselect { multiselect, id, options, option_children?, reveals?, when? }` - Multiple choice
- `Choice { choice, id, options, default?, option_children?, reveals?, when? }` - Single selection
- `Group { group, elements, when? }` - Labeled container

**Key V2 Features:**

**When Clauses (`when?: Option<String>`):**
- Any element can have conditional visibility via `when` field
- Expression syntax supports: `@id`, `selected(@id, value)`, `count(@id) > 2`
- Logical operators: `&&`, `||`, `!`
- Example: `"when": "@debug && count(@features) >= 2"`

**Reveals (`reveals?: Option<Vec<Element>>`):**
- Inline conditionals attached to checkbox/multiselect/choice
- Shown when parent is active (checkbox checked, option selected, etc.)
- Replaces v1 inline `conditional` field

**Option-as-Key Nesting (`option_children?: HashMap<String, Vec<Element>>`):**
- Choice/Multiselect children use option text as direct JSON key
- Example: `"Advanced": [{"slider": "Level", "id": "level", "min": 1, "max": 10}]`
- Replaces v1 OptionValue enum with nested conditionals

**V2 JSON Format:**
```json
{
  "checkbox": "Enable debug",
  "id": "enable_debug",
  "default": false,
  "reveals": [
    {
      "slider": "Debug level",
      "id": "debug_level",
      "min": 1,
      "max": 10
    }
  ]
}
```

**Choice with Option-as-Key Nesting:**
```json
{
  "choice": "Mode",
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

### PopupState
Runtime state management for widget values. Uses HashMap keyed by element IDs (not labels).

```rust
pub struct PopupState {
    pub values: HashMap<String, ElementValue>,
    pub button_clicked: Option<String>,
}
```

**ElementValue variants:** Number, Boolean, Text, MultiChoice(Vec<bool>), Choice(Option<usize>)

**Methods:**
- `new(definition)` - Initialize from definition with defaults
- `get_*_mut(id)` - Type-safe mutable access by ID (returns Option)
- `get_*(id)` - Immutable access for conditionals
- `to_value_map()` - Convert to serde_json::Value for when clause evaluation

### When Clause Evaluation

**Condition Parsing (`condition.rs`):**
```rust
pub fn parse_condition(expr: &str) -> Result<ConditionAst>
pub fn evaluate_condition(ast: &ConditionAst, state: &HashMap<String, Value>) -> bool
```

**ConditionAst:**
- `BooleanRef(id)` - `@id` checks if value is truthy
- `Selected { id, value }` - `selected(@id, value)` checks if specific option selected
- `Count { id, op, value }` - `count(@id) > 2` counts selections
- `And(left, right)`, `Or(left, right)`, `Not(inner)` - Logical operators

**Examples:**
- `"@enable_debug"` → Check if checkbox is checked
- `"selected(@mode, Expert)"` → Check if "Expert" option selected
- `"count(@features) >= 3"` → Check if 3+ multiselect options selected
- `"@debug && !@production"` → Logical AND and NOT

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


## Example JSON (V2 Schema)

**Simple Example with When Clauses:**
```json
{
  "title": "Settings",
  "elements": [
    {
      "text": "Configure options"
    },
    {
      "checkbox": "Enable feature",
      "id": "enable_feature",
      "default": true
    },
    {
      "slider": "Level",
      "id": "level",
      "min": 0,
      "max": 100,
      "when": "@enable_feature"
    }
  ]
}
```

**Example with Reveals:**
```json
{
  "title": "Advanced Config",
  "elements": [
    {
      "checkbox": "Enable advanced",
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
    }
  ]
}
```

**Example with Option-as-Key Nesting:**
```json
{
  "title": "Mode Selection",
  "elements": [
    {
      "choice": "Mode",
      "id": "mode",
      "options": ["Basic", "Pro", "Enterprise"],
      "Pro": [
        {
          "textbox": "License key",
          "id": "license",
          "placeholder": "XXXX-XXXX"
        }
      ],
      "Enterprise": [
        {
          "textbox": "Organization",
          "id": "org",
          "placeholder": "Company name"
        }
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
- Shared types eliminate duplication between GUI and potential future integrations
- Clean separation between data model and rendering logic
- Enables type evolution without touching GUI code

**Serialization strategy:**
- Tagged enums with `#[serde(tag = "status")]` for PopupResult
- Optional fields use `#[serde(default)]` for forward compatibility
- Result variants cover all possible popup outcomes (completed/cancelled/timeout)
