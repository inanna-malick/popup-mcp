# Popup-MCP Schema v2 Proposal

## Executive Summary

Complete redesign of popup JSON schema optimizing for:
- **LLM authoring ease** - Predictable patterns, minimal ceremony
- **Deep conditional trees** - 3-4 levels naturally expressible
- **Token efficiency** - ~40% reduction vs current schema
- **Clarity** - Structure mirrors logic, self-documenting

**Breaking change - replaces current schema entirely.**

---

## Design Principles

1. **Element-as-key** - Type becomes key, not property
2. **Option-as-key nesting** - Options directly nest child elements
3. **Minimal keywords** - Reduce structural ceremony
4. **When-clause filtering** - Cross-element conditions on any element
5. **Progressive revelation** - Structure shows conditional hierarchy

---

## Core Syntax Patterns

### Element-as-Key

Element types are keys, not `type` properties:

```json
{"slider": {...}}        // NOT {"type": "slider", ...}
{"checkbox": {...}}      // NOT {"type": "checkbox", ...}
```

### Option-as-Key Nesting

Parent element options can directly nest children using option value as key:

```json
{
  "choice": "Theme",
  "options": ["Light", "Dark"],
  "Dark": [
    {"slider": "Contrast", "min": 0, "max": 100}
  ]
}
```

**Semantics:** Elements nested under "Dark" only visible when "Dark" is selected.

### When-Clause Filtering

Any element can have optional `when` condition:

```json
{"text": "Warning", "when": "@cpu > 80 && @memory > 80"}
```

**Semantics:** Additional filter beyond structural nesting. Enables cross-element conditionals.

### References and IDs

**Reference syntax**: `@id` - Reference any element by its explicit `id` property

**ID assignment**:
- IDs are **always explicit** - no auto-generation from labels
- Only assign `id` to elements that will be referenced in conditions
- IDs must be unique within a popup
- Keep IDs short and semantic: `"id": "cpu"` not `"id": "cpu_usage_percentage"`

**Why explicit IDs**:
- **Stability**: Label changes don't break references
- **Clarity**: Short IDs make conditions readable (`@cpu > 80` vs `@cpu_usage_percentage > 80`)
- **No magic**: What you write is what you get, no transformation rules to learn
- **Unambiguous**: No collision handling needed, uniqueness enforced

**Example**:
```json
{
  "slider": "CPU Usage %",
  "id": "cpu",  // Explicit ID for referencing
  "min": 0,
  "max": 100
}

{"text": "High CPU", "when": "@cpu > 80"}
```

**Built-in functions**: `count(@field)`, `selected(@field, value)`, `any(...)`, `all(...)`

---

## Element Types Reference

### Text

Static text display, optionally conditional.

**Syntax:**
```json
{"text": "Display text"}
{"text": "Conditional text", "when": "@condition"}
```

**Properties:**
- `text` (string, required) - Content to display
- `when` (expression, optional) - Visibility condition

**Examples:**
```json
{"text": "Hello World"}
{"text": "⚠ High CPU detected", "when": "@cpu > 80"}
{"text": "All systems normal", "when": "@cpu < 50 && @mem < 50"}
```

---

### Slider

Numeric input with range.

**Syntax:**
```json
{
  "slider": "Label",
  "id": "ref",
  "min": 0,
  "max": 100,
  "default": 50
}
```

**Properties:**
- `slider` (string, required) - Label text
- `id` (string, optional) - Reference ID for conditions
- `min` (number, required) - Minimum value
- `max` (number, required) - Maximum value
- `default` (number, optional) - Initial value (defaults to `min` if not specified)
- `when` (expression, optional) - Visibility condition

**With value-based reveals:**
```json
{
  "slider": "CPU %",
  "id": "cpu",
  "min": 0,
  "max": 100,
  "reveals": [
    {"text": "⚠ High", "when": "@cpu > 80"},
    {"text": "✓ Normal", "when": "@cpu < 50"}
  ]
}
```

**Note:** `reveals` array contains elements shown when slider is interacted with. Use `when` on nested elements for value-specific logic.

---

### Checkbox

Boolean input with optional nested reveals.

**Syntax:**
```json
{
  "checkbox": "Label",
  "id": "ref",
  "default": false
}
```

**Properties:**
- `checkbox` (string, required) - Label text
- `id` (string, optional) - Reference ID
- `default` (boolean, optional) - Initial state (default: false)
- `reveals` (array, optional) - Elements shown when checked
- `when` (expression, optional) - Visibility condition

**Examples:**
```json
{"checkbox": "Enable debug mode"}

{
  "checkbox": "Enable advanced",
  "id": "advanced",
  "reveals": [
    {"slider": "Level", "min": 1, "max": 10},
    {"textbox": "Config path", "placeholder": "/etc/config"}
  ]
}
```

---

### Textbox

Text input (single or multi-line).

**Syntax:**
```json
{
  "textbox": "Label",
  "id": "ref",
  "placeholder": "Hint text",
  "rows": 3
}
```

**Properties:**
- `textbox` (string, required) - Label text
- `id` (string, optional) - Reference ID
- `placeholder` (string, optional) - Placeholder text
- `rows` (number, optional) - Number of rows (1 = single-line, >1 = multi-line)
- `when` (expression, optional) - Visibility condition

**Examples:**
```json
{"textbox": "Name", "placeholder": "Enter your name"}
{"textbox": "Description", "rows": 5}
{"textbox": "Error details", "rows": 3, "when": "@has_error"}
```

---

### Choice (Single Selection)

Dropdown with single selection.

**Syntax:**
```json
{
  "choice": "Label",
  "id": "ref",
  "options": ["Option A", "Option B"],
  "default": 0
}
```

**Properties:**
- `choice` (string, required) - Label text
- `id` (string, optional) - Reference ID
- `options` (array, required) - Available options (strings)
- `default` (number, optional) - Initial selection index
- `when` (expression, optional) - Visibility condition

**With per-option reveals:**
```json
{
  "choice": "Theme",
  "id": "theme",
  "options": ["Light", "Dark", "Auto"],
  "Dark": [
    {"slider": "Contrast", "min": 0, "max": 100}
  ],
  "Auto": [
    {"checkbox": "Follow system"}
  ]
}
```

**Semantics:** Elements nested under option key only visible when that option selected.

---

### Multiselect

Multiple selection checkboxes.

**Syntax:**
```json
{
  "multiselect": "Label",
  "id": "ref",
  "options": ["Option A", "Option B"]
}
```

**Properties:**
- `multiselect` (string, required) - Label text
- `id` (string, optional) - Reference ID
- `options` (array, required) - Available options (strings)
- `when` (expression, optional) - Visibility condition
- `reveals` (array, optional) - Elements shown when any option selected

**With per-option reveals:**
```json
{
  "multiselect": "Features",
  "id": "features",
  "options": ["Analytics", "Sync", "Backup"],
  "Analytics": [
    {"checkbox": "Track usage"}
  ],
  "Sync": [
    {"textbox": "Sync interval", "placeholder": "seconds"}
  ]
}
```

**With count-based reveals:**
```json
{
  "multiselect": "Changes",
  "id": "changes",
  "options": ["Code", "Config", "Schema"],
  "reveals": [
    {"text": "⚠ Too many changes", "when": "count(@changes) >= 3"},
    {"text": "✓ Single change", "when": "count(@changes) == 1"}
  ]
}
```

---

### Group

Container for organizing elements (always visible).

**Syntax:**
```json
{
  "group": "Label",
  "elements": [...]
}
```

**Properties:**
- `group` (string, required) - Group label
- `elements` (array, required) - Child elements
- `when` (expression, optional) - Visibility condition

**Example:**
```json
{
  "group": "Resource Metrics",
  "elements": [
    {"slider": "CPU %", "id": "cpu", "min": 0, "max": 100},
    {"slider": "Memory %", "id": "mem", "min": 0, "max": 100},
    {"text": "🚨 CRITICAL", "when": "@cpu > 80 && @mem > 80"}
  ]
}
```

---

## Condition Language

Boolean expressions for `when` clauses.

### PEG Grammar

```peg
expr     = or
or       = and ("||" and)*
and      = comp ("&&" comp)*
comp     = value (op value)?
op       = "==" | "!=" | ">=" | "<=" | ">" | "<"
value    = ref | func | number | string | ident | "(" expr ")" | "!" value
ref      = "@" ident
func     = ident "(" args? ")"
args     = expr ("," expr)*
ident    = [a-zA-Z_][a-zA-Z0-9_]*
number   = [0-9]+ ("." [0-9]+)?
string   = "'" [^']* "'" | '"' [^"]* '"'
```

**Key feature**: Bare identifiers (no `@` prefix) in value positions are treated as string literals. This eliminates quote noise in common cases.

**Parse precedence**:
1. `@identifier` → field reference
2. `identifier(...)` → function call
3. `"string"` or `'string'` → explicit string literal
4. `123` or `45.67` → number literal
5. `identifier` → implicit string literal

**When quotes are required**:
- Strings with spaces: `"Very Advanced"`
- Strings with special characters: `"O'Reilly"`, `"hello@world"`
- Strings starting with numbers: `"2fast2furious"`

**Examples**:
- `@theme` → reference to "theme" field
- `Dark` → string literal "Dark"
- `selected(@theme, Dark)` → no quotes needed
- `selected(@theme, "Very Dark")` → quotes needed for space

### Built-in Functions

**`count(@field)`**
- Returns number of selected items in multiselect
- Returns 1 if checkbox checked, 0 if unchecked
- Returns 1 if choice has selection, 0 if no selection
- Can be used in comparisons: `count(@features) >= 3`

**`selected(@field, value)`**
- Returns true if specific value is selected
- Works for multiselect: `selected(@features, Analytics)`
- Works for choice: `selected(@theme, Dark)`
- Works for checkbox: `selected(@debug, "Debug mode")` (matches label)
- Note: Unquoted identifiers treated as strings (see grammar above)

**`any(expr1, expr2, ...)`**
- Returns true if any expression evaluates to truthy value
- Short-circuits: stops evaluating after first truthy result
- Example: `any(@cpu > 90, @mem > 90, @disk > 90)`
- Equivalent to: `@cpu > 90 || @mem > 90 || @disk > 90`
- Use case: Compact OR chains for multiple conditions

**`all(expr1, expr2, ...)`**
- Returns true if all expressions evaluate to truthy values
- Short-circuits: stops evaluating after first falsy result
- Example: `all(@cpu < 50, @mem < 50, @disk < 50)`
- Equivalent to: `@cpu < 50 && @mem < 50 && @disk < 50`
- Use case: Compact AND chains for multiple conditions

### Operator Precedence

From highest to lowest precedence:

1. **Grouping**: `( ... )`
2. **Negation**: `!`
3. **Comparison**: `>`, `<`, `>=`, `<=`
4. **Equality**: `==`, `!=`
5. **Logical AND**: `&&`
6. **Logical OR**: `||`

**Examples**:
- `@a && @b || @c` → `(@a && @b) || @c` (AND binds tighter)
- `!@a && @b` → `(!@a) && @b` (NOT binds tightest)
- `@x > 5 && @y > 10` → `(@x > 5) && (@y > 10)` (comparison before AND)

**Recommendation**: Use explicit parentheses for clarity when mixing operators.

### Truthiness Rules

**Truthy values**:
- Non-zero numbers: `1`, `-5`, `3.14`
- Non-empty strings: `"text"`, `Dark`
- Boolean `true`
- Checkbox checked state

**Falsy values**:
- Number `0`
- Empty string `""`
- Boolean `false`
- Checkbox unchecked state
- Null/undefined references (non-existent IDs)

**Usage**: `@enabled` as bare condition checks truthiness. `@checkbox` is truthy when checked.

### Type Coercion

**Comparison rules**:
- Number to string: `@slider == "50"` → string `"50"` coerced to number `50`
- String to number: `@textbox == 42` → string coerced to number if possible
- Mixed types with no conversion: always false (`@slider == Advanced` → false)

**Recommendations**:
- Use numbers for numeric comparisons: `@slider >= 50`
- Use strings for text matching: `@choice == Dark`
- Explicit is better than implicit - avoid relying on coercion

### Short-Circuit Evaluation

**Logical operators short-circuit**:
- `@a && @b` → if `@a` is falsy, `@b` is never evaluated
- `@a || @b` → if `@a` is truthy, `@b` is never evaluated
- `any(@a, @b, @c)` → stops at first truthy value
- `all(@a, @b, @c)` → stops at first falsy value

**Use case**: Safely reference potentially undefined fields:
```
@optional_field && @optional_field > 50
```
If `@optional_field` doesn't exist (falsy), second part never evaluates.

### Expression Examples

```
@cpu > 80
@cpu > 80 && @mem > 80
count(@changes) >= 3
count(@changes) == 1
selected(@features, Analytics)
@level > 7 && selected(@theme, Dark)
!@enabled || @override
(@cpu > 80 || @mem > 80) && @alert_enabled
any(@cpu > 90, @mem > 90, @disk > 90)
all(@cpu < 50, @mem < 50, @disk < 50)
```

---

## Complete Examples

### Simple Confirmation

```json
{
  "title": "Delete File?",
  "elements": [
    {"text": "This action cannot be undone."},
    {"text": "Are you sure you want to delete this file?"}
  ]
}
```

### Settings with Nested Options

```json
{
  "title": "Application Settings",
  "elements": [
    {"slider": "Volume", "min": 0, "max": 100, "default": 75},
    {
      "checkbox": "Enable notifications",
      "default": true,
      "reveals": [
        {"checkbox": "Sound"},
        {"checkbox": "Badge"}
      ]
    },
    {
      "choice": "Theme",
      "id": "theme",
      "options": ["Light", "Dark", "Auto"],
      "Dark": [
        {"slider": "Contrast", "min": 0, "max": 100},
        {"checkbox": "True black"}
      ]
    }
  ]
}
```

### Per-Option Branching

```json
{
  "title": "Deployment Configuration",
  "elements": [
    {
      "choice": "Environment",
      "id": "env",
      "options": ["Development", "Staging", "Production"],
      "Production": [
        {
          "checkbox": "Enable monitoring",
          "default": true,
          "reveals": [
            {"multiselect": "Metrics", "options": ["CPU", "Memory", "Disk", "Network"]}
          ]
        },
        {"checkbox": "Enable alerting", "default": true}
      ],
      "Development": [
        {"checkbox": "Enable debug logging", "default": true},
        {"textbox": "Log level", "placeholder": "DEBUG"}
      ]
    }
  ]
}
```

### Cross-Element Conditionals

```json
{
  "title": "System Health Check",
  "elements": [
    {
      "group": "Resource Utilization",
      "elements": [
        {"slider": "CPU %", "id": "cpu", "min": 0, "max": 100},
        {"slider": "Memory %", "id": "mem", "min": 0, "max": 100},
        {"slider": "Disk I/O %", "id": "disk", "min": 0, "max": 100},
        {"text": "🚨 CRITICAL: Resource exhaustion detected", "when": "@cpu > 80 && @mem > 80"},
        {"text": "⚠ Disk pressure", "when": "@disk > 90"},
        {"text": "✓ All resources healthy", "when": "@cpu < 50 && @mem < 50 && @disk < 50"}
      ]
    },
    {
      "checkbox": "Enable emergency rate limiting",
      "when": "@cpu > 80 || @mem > 80"
    }
  ]
}
```

### Complex Nested Tree (4 Levels)

```json
{
  "title": "Production Incident Diagnosis",
  "elements": [
    {
      "choice": "Incident Type",
      "id": "type",
      "options": ["Performance", "Error Rate", "Data Integrity"],
      "Performance": [
        {
          "choice": "When did degradation start?",
          "id": "timing",
          "options": ["Sudden (after deployment)", "Gradual (over days)", "Intermittent (no pattern)"],
          "Sudden (after deployment)": [
            {
              "multiselect": "What changed in last deployment?",
              "id": "changes",
              "options": ["New code", "Configuration changes", "Database schema migration", "Infrastructure scaling", "Dependency updates"],
              "reveals": [
                {"text": "⚠ Multiple simultaneous changes detected. High complexity deployment - recommend immediate rollback.", "when": "count(@changes) >= 3"},
                {"text": "✓ Single change isolation - good signal for root cause", "when": "count(@changes) == 1"}
              ],
              "New code": [
                {
                  "multiselect": "Which services were deployed?",
                  "id": "services",
                  "options": ["Auth service", "API gateway", "Database layer", "Cache layer", "Background workers"],
                  "reveals": [
                    {"text": "✓ Isolated deployment", "when": "count(@services) == 1"},
                    {
                      "textbox": "What does this service do?",
                      "rows": 2,
                      "when": "count(@services) == 1"
                    },
                    {"text": "⚠ Broad deployment", "when": "count(@services) >= 3"},
                    {
                      "checkbox": "Can you isolate to one service?",
                      "when": "count(@services) >= 3",
                      "reveals": [
                        {
                          "choice": "Which service shows symptoms?",
                          "options": ["Auth", "API", "Database", "Cache", "Workers"]
                        }
                      ]
                    }
                  ]
                }
              ],
              "Configuration changes": [
                {
                  "choice": "What configuration changed?",
                  "id": "config_type",
                  "options": ["Connection pool size", "Timeout values", "Cache TTL", "Rate limits"],
                  "Connection pool size": [
                    {
                      "slider": "New pool size",
                      "id": "pool_size",
                      "min": 1,
                      "max": 100,
                      "reveals": [
                        {"text": "Large pool - check connection exhaustion", "when": "@pool_size > 50"},
                        {"text": "Small pool - check queueing", "when": "@pool_size < 10"}
                      ]
                    }
                  ]
                }
              ]
            }
          ]
        },
        {
          "group": "Resource Metrics (always visible)",
          "elements": [
            {"slider": "CPU usage %", "id": "cpu", "min": 0, "max": 100},
            {"slider": "Memory usage %", "id": "mem", "min": 0, "max": 100},
            {"slider": "Disk I/O %", "id": "disk", "min": 0, "max": 100},
            {
              "text": "🚨 CRITICAL: Both CPU and memory maxed out. System in death spiral.",
              "when": "@cpu > 90 && @mem > 90"
            },
            {
              "text": "Correlation: Schema migration + disk saturation = missing index?",
              "when": "@disk > 90 && selected(@changes, 'Database schema migration')"
            },
            {
              "text": "Resources healthy - check application-level bottleneck",
              "when": "@cpu < 50 && @mem < 50"
            }
          ]
        }
      ]
    }
  ]
}
```

---

## Schema Summary (TypeScript)

```typescript
type PopupDefinition = {
  title?: string;
  elements: Element[];
}

type Element =
  | TextElement
  | SliderElement
  | CheckboxElement
  | TextboxElement
  | ChoiceElement
  | MultiselectElement
  | GroupElement;

type TextElement = {
  text: string;
  when?: Condition;
}

type SliderElement = {
  slider: string;
  id?: string;
  min: number;
  max: number;
  default?: number;
  reveals?: Element[];
  when?: Condition;
}

type CheckboxElement = {
  checkbox: string;
  id?: string;
  default?: boolean;
  reveals?: Element[];
  when?: Condition;
}

type TextboxElement = {
  textbox: string;
  id?: string;
  placeholder?: string;
  rows?: number;
  when?: Condition;
}

type ChoiceElement = {
  choice: string;
  id?: string;
  options: string[];
  default?: number;
  when?: Condition;
  [optionValue: string]: Element[] | any;  // Option-as-key nesting
}

type MultiselectElement = {
  multiselect: string;
  id?: string;
  options: string[];
  reveals?: Element[];
  when?: Condition;
  [optionValue: string]: Element[] | any;  // Option-as-key nesting
}

type GroupElement = {
  group: string;
  elements: Element[];
  when?: Condition;
}

type Condition = string;  // Boolean expression string
```

---

## Implementation Notes

### Parser Strategy

1. **Deserialize JSON** - Standard serde parsing into Rust structs
2. **Option-as-key detection** - Any string key not in known properties = option nesting
3. **Condition evaluation** - PEG parser for `when` expressions (pest crate)
4. **Reference resolution** - Build ID → element map during initialization
5. **Visibility computation** - Evaluate conditions each frame, filter elements

### Migration from v1

**No migration support - breaking change**

Users must rewrite popups in new schema. Complexity reduced enough that manual rewrite is faster than building migration tool.

### Error Handling

**Parser errors:**
- Missing required properties: Clear message with expected schema
- Unknown element type: List valid types
- Invalid condition syntax: Point to grammar docs + highlight error location
- Unresolved reference: Show available IDs (e.g., "Unknown ID 'cpu_usage'. Did you mean 'cpu'? Available: cpu, mem, disk")
- Duplicate IDs: Error with both locations

**Runtime errors:**
- Reference to non-existent ID: Skip condition (treat as false), log warning
- Type mismatch in condition: Skip condition, log warning
- Circular dependencies: Detect during initialization, error

**ID-specific validation:**
- Uniqueness check at parse time
- References validated against declared IDs
- Suggest similar IDs for typos (Levenshtein distance)

### Performance Considerations

- **Condition evaluation:** Parse expressions once at load, cache AST
- **Reference lookups:** HashMap by ID, O(1) access
- **Visibility filtering:** Single pass per frame, early exit on false conditions
- **Deep nesting:** No performance penalty - filtered before rendering

---

## Design Rationale

### Why Element-as-Key?

**Before:** `{"type": "slider", "label": "Volume", ...}`
**After:** `{"slider": "Volume", ...}`

**Savings:** 13 characters per element → 30-40% token reduction
**Clarity:** Type is first thing you see, not buried in properties
**Ergonomics:** Less typing, less nesting

### Why Option-as-Key?

**Before:**
```json
{
  "type": "choice",
  "label": "Theme",
  "options": ["Light", "Dark"],
  "conditionals": [
    {
      "condition": {"value": "Dark"},
      "elements": [...]
    }
  ]
}
```

**After:**
```json
{
  "choice": "Theme",
  "options": ["Light", "Dark"],
  "Dark": [...]
}
```

**Savings:** 60+ characters for per-option conditionals
**Clarity:** Visual nesting shows relationship immediately
**Locality:** Related elements stay together

### Why Flat `when` Instead of Wrapper?

**Before:** `{"if": "@cpu > 80", "then": {"text": "Warning"}}`
**After:** `{"text": "Warning", "when": "@cpu > 80"}`

**Savings:** 17 characters per conditional element
**Clarity:** Condition is property, not structure
**Consistency:** Works on any element uniformly

### Why PEG for Conditions?

**Alternatives considered:**
- JSON objects: `{"field": "cpu", "op": ">", "value": 80}` - verbose
- Limited DSL: `"cpu > 80"` - not extensible to boolean logic

**PEG advantages:**
- Familiar syntax: `@cpu > 80 && @mem > 80`
- Full boolean logic: AND, OR, NOT, parentheses
- Functions: `count()`, `selected()`, `any()`, `all()`
- Extensible: Easy to add operators/functions
- Fast parsing: pest library optimized

### Why Explicit IDs?

**Alternatives considered:**
- Auto-generation from labels via slug transform
- Label-direct references (`@'Label Text'`)
- Positional references (`@0`, `@1`)

**Explicit ID advantages:**
- **Stability**: Label text changes don't break references
- **Readability**: Short semantic IDs (`@cpu`) vs verbose auto-gen (`@cpu_usage_percentage`)
- **No magic**: Direct mapping, no transformation rules to remember
- **Unambiguous**: Uniqueness enforced, no collision handling needed
- **Semantic work**: Choosing IDs forces meaningful naming (feature, not bug)

---

## ID Best Practices

### When to Add IDs

**Add `id` property when element will be referenced in conditions:**
```json
{"slider": "CPU %", "id": "cpu", ...}  // Will reference in conditions

{"slider": "Volume", ...}  // Never referenced - no ID needed
```

### Naming Conventions

**Keep IDs short and semantic:**
- ✅ Good: `"id": "cpu"`, `"id": "theme"`, `"id": "env"`
- ❌ Verbose: `"id": "cpu_usage_percentage"`, `"id": "selected_theme_option"`

**Use domain language:**
- ✅ `"id": "exec"` for executive function slider
- ✅ `"id": "phenom"` for phenomenology choice
- ❌ Generic: `"id": "slider1"`, `"id": "choice_a"`

**Common abbreviations:**
- `cpu`, `mem`, `disk` - resources
- `env` - environment
- `exec` - executive
- `auth` - authentication
- `config` - configuration

### ID Scope

**IDs must be unique within a popup** - siblings, nested elements, conditionals all share namespace.

**Good practice**: Prefix nested IDs with parent context when ambiguity possible:
```json
{
  "choice": "Paradigm",
  "id": "paradigm",
  "Qualitative": [
    {"multiselect": "Approaches", "id": "qual_approaches"}  // Prefixed
  ],
  "Quantitative": [
    {"multiselect": "Approaches", "id": "quant_approaches"}  // Prefixed
  ]
}
```

---

## Design Decisions Resolved

**ID strategy:** Explicit only, no auto-generation. See "Why Explicit IDs?" and "ID Best Practices" sections above.

**Mixed reveals:** Multiselect supports both general `reveals` array AND per-option nesting via option-as-key. Use `reveals` with `when` clauses for count-based logic, option-keys for per-selection conditionals.

**Slider references:** Sliders with `id` are immediately referenceable with their default value (which defaults to `min` if not specified). No need to wait for user interaction.

**Group element:** Retained for semantic chunking and visual organization. Provides structural clarity even when not using conditional logic

---

## Success Metrics

**Quantitative:**
- 40% token reduction vs v1 schema (measured on incident diagnosis example)
- 50% fewer keywords (`type`, `condition`, `elements`, `conditional`, etc.)
- 100% feature parity with v1 (all conditionals expressible)

**Qualitative:**
- LLM can generate complex trees without errors
- Human can read and understand structure at a glance
- Modifications (add/remove/reorder) are straightforward
- Error messages pinpoint issues clearly

---

## Timeline

**Implementation estimate: 2-3 days**

1. Define Rust types with serde (4 hours)
2. Implement PEG parser for conditions with pest (6 hours)
3. Port GUI rendering logic to new schema (4 hours)
4. Write comprehensive tests (4 hours)
5. Update MCP schema + documentation (2 hours)

**No migration period needed - immediate cutover acceptable for single-user tool.**
