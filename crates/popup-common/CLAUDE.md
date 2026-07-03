# popup-common

Shared data structures and protocol definitions for the popup-mcp system.

## Purpose

Provides core types shared across the popup-mcp architecture:
- **popup-gui** - Native renderer consumes PopupDefinition, produces PopupResult
- Condition evaluation for when clauses

## Core Types

### PopupDefinition
JSON-serializable dialog definition.

```rust
pub struct PopupDefinition {
    pub title: String,
    pub elements: Vec<Element>,
    pub submit: Option<String>, // submit button label (default "Submit")
    pub notes: bool,            // show the "Add a note" field (default true)
}
```

Construct with `PopupDefinition::new(title, elements)` in tests.

### Element
Enum of widget types using element-as-key (de)serialization: the JSON key is the widget type and its value is the label. Deserialization is **strict** - unknown keys (including typo'd option-as-key branch keys) are errors with "did you mean" suggestions.

**Variants (JSON key = first field):**
- `Text { text, id?, when? }` - Static text
- `Markdown { markdown, id?, when? }` - Rich text block
- `Slider { slider, id, min, max, default?, step?, when? }` - Numeric input (f32); step defaults to 1
- `Check { check, id, default, reveals, when? }` - Boolean with nested reveals
- `Input { input, id, placeholder?, default?, rows?, when? }` - Text input; `default` prefills
- `Multi { multi, id, options, option_children, reveals, when? }` - Multiple choice
- `Select { select, id, options, default?, style?, option_children, reveals, when? }` - Single choice; `style` is `radio` or `dropdown`
- `Group { group, id?, elements, when? }` - Labeled container

**Accessors:** `element.id()`, `element.when()`, `element.label()`, `element.kind()`, `element.child_lists()`. Free functions `find_element_by_id(elements, id)` and `walk_elements(elements, f)` traverse the whole tree (reveals, option children, groups) - use these instead of hand-rolled recursion.

Ids are auto-derived from labels at deserialization when omitted ("Enable Feature" -> `enable_feature`).

**Option-as-Key Nesting (`option_children: HashMap<String, Vec<Element>>`):**
- Select/Multi children use option text as direct JSON key
- Example: `"Advanced": [{"slider": "Level", "id": "level", "min": 1, "max": 10}]`

### Validation (`validate.rs`)

`validate_definition(&PopupDefinition) -> Vec<ValidationIssue>` runs pre-flight checks so structural mistakes surface as fixable errors instead of mis-rendered popups:
- duplicate ids; reserved ids (`status`, `button`, `error`, `_notes`)
- slider `min < max`, default in range, positive step
- select default must be one of the options (with closest-match suggestion)
- when clauses must parse, and every referenced id must exist (with suggestion)

`suggest()` / `edit_distance()` provide the "did you mean" machinery, shared with element_deser.

### PopupState
Runtime state management for widget values. Uses HashMap keyed by element IDs (not labels).

```rust
pub struct PopupState {
    pub values: HashMap<String, ElementValue>,
    pub button_clicked: Option<String>,
    pub notes: String, // free-form user note
}
```

**ElementValue variants:** Number, Boolean, Text, MultiChoice(Vec<bool>), Choice(Option<usize>)

**Methods:**
- `new(definition)` - Initialize from definition with defaults (input `default` prefills Text)
- `get_*_mut(id)` - Type-safe mutable access by ID (returns Option)
- `get_*(id)` - Immutable access for conditionals
- `to_value_map()` - Convert to serde_json::Value for when clause evaluation

### When Clause Evaluation

**Condition Parsing (`condition.rs` + `condition.pest`):**
```rust
pub fn parse_condition(expr: &str) -> Result<ConditionExpr>
pub fn evaluate_condition(expr: &ConditionExpr, state: &HashMap<String, Value>) -> bool
pub fn collect_refs(expr: &ConditionExpr, out: &mut Vec<String>) // ids referenced (for validation)
```

**Visibility scoping (`visibility.rs`):** `compute_visibility` / `collect_active_ids` / `visible_value_map` compute the active element set and the value map conditions see, at fixed point. Conditions only see visible elements' values - hidden ids read as unset, matching the result payload's "absent = branch not taken" semantics.

**Syntax notes:**
- Bare identifiers are references; a legacy `@id` prefix is accepted and stripped
- `selected(id, value)`: value may be an unquoted phrase with spaces (`selected(strategy, Regenerate from sandbox)`) - it is parsed as a literal, not a reference
- Numbers may be negative (`lufs > -9`)
- The whole input must parse: trailing garbage is an error (`enabled && (` fails instead of silently evaluating `enabled`)

**Examples:**
- `"enable_debug"` -> Check if checkbox is checked
- `"selected(mode, Expert)"` -> Check if "Expert" option selected
- `"count(features) >= 3"` -> Check if 3+ multiselect options selected
- `"debug && !production"` -> Logical AND and NOT

### PopupResult
Serialized user interaction result.

**Variants:**
- `Completed { values, button }` - User submitted with values
- `Cancelled` - User pressed Escape (no values)

**Construction:** single entry point `PopupResult::from_state(state, definition, active_ids)`.

Value semantics (kept in sync with the MCP tool description in popup-gui/src/schema.rs):
- absent key = element never shown; `""` = input shown but blank; `null` = select shown, unselected
- select -> option text; multi -> array of option texts
- slider -> integer when whole, float otherwise (no blanket integer truncation)
- `_notes` = trimmed free-form note, present only when non-empty

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
      "check": "Enable feature",
      "id": "enable_feature",
      "default": true
    },
    {
      "slider": "Level",
      "id": "level",
      "min": 0,
      "max": 100,
      "when": "enable_feature"
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
      "check": "Enable advanced",
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
      "select": "Mode",
      "id": "mode",
      "options": ["Basic", "Pro", "Enterprise"],
      "Pro": [
        {
          "input": "License key",
          "id": "license",
          "placeholder": "XXXX-XXXX"
        }
      ],
      "Enterprise": [
        {
          "input": "Organization",
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
- Result variants cover all popup outcomes (completed/cancelled)
