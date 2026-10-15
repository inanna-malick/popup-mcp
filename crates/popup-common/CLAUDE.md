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
- `Page { page, id?, elements, when?, next }` - Wizard step / decision-tree node (top level only). `next: Vec<Edge>` turns the pages into a graph; empty means linear wizard behaviour.

### Edge (`Edge { to, r#if, when }`)
One way out of a page. `to` is a page id or `"submit"`. Exactly one of `if` (a condition described in words, judged semantically) or `when` (ordinary condition syntax, evaluated in code) - deserialization rejects both or neither. `Element::page_id()` gives a page's routing id (explicit, else derived from the label).

### Routing (`route.rs`)
Pure, network-free decision logic, so routing is testable without a GUI or an API key:
- `decide(elements, page, visible_values, path) -> RouteDecision` - `Mechanical` when a `when` edge holds, `Ask(packet)` when semantic edges exist, `Submit` otherwise
- `route_packet(elements, page, path)` **compiles** the Jev request from the tree so the authoring model writes only `{"to", "if"}` per edge. What the compiler adds, and why (each follows a measured Jev practice):
  - state is the current page's answers keyed by **label**, since the conditions are written in words and must match the state's language
  - each option is the author's condition alone (condition-described beat action-described 0.95 vs 0.72 in the source research) plus `the branch then asks`, the destination page's question labels, so a branch is judged on where it leads rather than on its name (the taxonomy-walk pattern)
  - two described exits, `none_fit` and `unclear`, because "no branch applies" and "not enough said to tell" mean different things to the caller
  - `earlier_pages` capped to `EARLIER_PAGES_KEPT`; accuracy falls as state fills with detail the question does not need
  - three companion question groups ride in the same packet, since extra questions are nearly free and extra calls are not: `holds:<edge>` Nouls (does this condition hold on its own), a `contradicts` Noul for the note, and `prefill:<element id>` Choices over a destination select's options with a `not_implied` exit
- `RouteAnswer::outcome(edges) -> Outcome` is the decision surface: `Act` (open a page), `Several` (open the first, queue the rest), `HandBack`. Order: an exit that won hands back; else the gated Choice acts; else exactly one Noul at `HOLD_FLOOR` acts; else several queue; else hand back with `leaning`.
- `suggestion()` gates on confidence, then mass, then `margin()`, which compares the leader against the better of the runner-up edge and **both exits summed** (they are separate options because they mean different things to the caller, but both mean "do not open a page"). Also `exit_mass()`, `exit_reason()`, `mass(key)`, `edge_key()` for duplicate targets.

**Accessors:** `element.id()`, `element.when()`, `element.label()`, `element.kind()`, `element.child_lists()`. Free functions `find_element_by_id(elements, id)` and `walk_elements(elements, f)` traverse the whole tree (reveals, option children, groups) - use these instead of hand-rolled recursion.

Ids are auto-derived from labels at deserialization when omitted ("Enable Feature" -> `enable_feature`).

**Option-as-Key Nesting (`option_children: HashMap<String, Vec<Element>>`):**
- Select/Multi children use option text as direct JSON key
- Example: `"Advanced": [{"slider": "Level", "id": "level", "min": 1, "max": 10}]`

### Validation (`validate.rs`)

`validate_definition(&PopupDefinition) -> Vec<ValidationIssue>` runs pre-flight checks so structural mistakes surface as fixable errors instead of mis-rendered popups:
- duplicate ids; reserved ids (`status`, `button`, `error`, `_notes`, `_labels`, `_route`)
- edge targets must name a top-level page or `"submit"` (with closest-match suggestion)
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
- `_route` = the hops taken in a decision tree (added by the GUI's `route_nav::with_route_log`)

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
