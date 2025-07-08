# Popup DSL Redesign: Crystalline Expression Language

## Design Principles (VIBES Target: `<🔬🎀💠>`)

### 🔬 Crystalline Expressive Power
Multiple natural ways to express the same semantic operation, supporting different mental models:

```
# All these create the same slider:
slider "Volume" 0..100 @50
range "Volume" from 0 to 100 default 50
scale "Volume" [0-100] starting at 50
Volume: 0 to 100 (default: 50)
Volume [0..100 @50]
```

### 🎀 Independent Context Flow
Each element is self-contained and can be understood in isolation:

```
# Elements can be reordered without affecting behavior
popup "Settings" [
    volume: 0..100,
    brightness: 0..255,
    enabled: ✓,
    buttons: Save | Cancel
]
```

### 💠 Crystal Error Surface
Errors caught at parse time with automatic fixes:

```
# Parser auto-corrects common mistakes:
"checkbox enabled"     → checkbox "enabled"
slider 0-100          → slider "Untitled" 0..100
choice [a,b,c]        → choice "Untitled" ["a","b","c"]
if Volume > 50 [...]  → if value("Volume") > 50 [...]
```

## Semantic Alias System

### Core Widget Aliases

Each widget type supports multiple aliases reflecting different mental models:

#### Checkbox/Boolean
```
checkbox, check, tick, toggle, switch, bool, boolean, yes/no, y/n, enabled, ✓, ☐
```

#### Slider/Range
```
slider, range, scale, numeric, number, dial, knob, level, gauge, meter
```

#### Textbox/Input
```
textbox, text, input, field, entry, textarea, string, prompt, write
```

#### Choice/Select
```
choice, select, dropdown, pick, choose, option, radio, single
```

#### Multiselect
```
multiselect, multi, multiple, checklist, pickMany, selectMultiple, options, tags
```

#### Buttons
```
buttons, actions, button, btns, controls, options
```

### Natural Language Patterns

Support conversational syntax alongside technical syntax:

```
# Technical
slider "Volume" 0..100 @50

# Natural variations
Volume from 0 to 100 starting at 50
Volume between 0 and 100 (default 50)
Volume: 0-100, initially 50
Volume ranging 0 to 100 = 50
```

### Symbol Shortcuts

For common UI patterns, support symbolic representations:

```
✓ enabled                 → checkbox "enabled" @true
☐ notifications          → checkbox "notifications" @false
★★★☆☆                    → slider "Rating" 1..5 @3
[•••••     ]             → slider "Progress" 0..10 @5
▼ Theme                  → choice "Theme" [...]
```

## Expression Grammar Evolution

### 1. Flexible Conditionals

Support multiple conditional syntaxes:

```
# Current (rigid)
if checked("notifications") [...]

# New alternatives
if notifications [...]
if notifications is checked [...]
when notifications = true [...]
show when notifications [...]
visible if notifications [...]
{notifications} ? [...] 
notifications => [...]
```

### 2. Natural Comparisons

```
# Current
if count("tasks") > 3 [...]

# New alternatives
if tasks.count > 3 [...]
if #tasks > 3 [...]
if selected tasks more than 3 [...]
when 3+ tasks selected [...]
tasks > 3 => [...]
```

### 3. Compound Conditions

```
# Boolean operators (all equivalent)
if notifications AND sound [...]
if notifications && sound [...]
if notifications and sound [...]
if both(notifications, sound) [...]
if all[notifications, sound] [...]

# Negation
if not notifications [...]
if !notifications [...]
if notifications = false [...]
unless notifications [...]
```

### 4. Value References

Multiple ways to reference widget values:

```
# All equivalent
if value("volume") > 50 [...]
if volume > 50 [...]
if $volume > 50 [...]
if @volume > 50 [...]
if {volume} > 50 [...]
```

### 5. Range Expressions

```
# All create slider 0-100 with default 50
0..100 @50
0 to 100 = 50
[0,100] default:50
0-100 (50)
from 0 to 100 starting at 50
min:0 max:100 val:50
```

### 6. List Expressions

```
# All create the same list
["Work", "Rest", "Play"]
[Work, Rest, Play]
Work | Rest | Play
Work / Rest / Play
- Work - Rest - Play
• Work • Rest • Play
```

## Error Recovery Mechanisms

### 1. Smart Quote Inference

```
popup Title [...]        → popup "Title" [...]
checkbox enabled         → checkbox "enabled"
buttons [OK, Cancel]     → buttons ["OK", "Cancel"]
```

### 2. Context-Aware Type Inference

```
Volume: 0-100           → slider "Volume" 0..100
Theme: Dark|Light       → choice "Theme" ["Dark", "Light"]
Tasks: Email|Code|Docs  → multiselect "Tasks" ["Email", "Code", "Docs"]
Enabled: yes            → checkbox "Enabled" @true
```

### 3. Missing Element Defaults

```
slider 0..100           → slider "Value" 0..100
choice ["A","B"]        → choice "Select" ["A", "B"]
0..100                  → slider "Untitled" 0..100
```

### 4. Typo Correction

Using Levenshtein distance for common typos:

```
chekbox "test"          → checkbox "test"
choise "theme"          → choice "theme"
mutliselect "tags"      → multiselect "tags"
butons ["OK"]           → buttons ["OK"]
```

### 5. Format Auto-Detection

Detect and transform between formats:

```
# Mixed format input
popup "Test" [
    Volume: 0..100,
    if volume > 50 [Loud: yes]
    buttons: OK
]

# Auto-transformed to consistent format
popup "Test" [
    slider "Volume" 0..100,
    if value("Volume") > 50 [
        checkbox "Loud" @true
    ],
    buttons ["OK"]
]
```

## Inline Documentation Support

Allow helpful hints without affecting semantics:

```
popup "Settings" [
    # Audio settings
    slider "Volume" 0..100 @50 ? "System volume level",
    checkbox "Mute" @false ? "Disable all audio",
    
    // Visual settings  
    choice "Theme" ["Light", "Dark", "Auto"] ? "Color scheme",
    
    /* Advanced */
    group "Advanced" [
        textbox "Custom CSS" ? "Override styles"
    ],
    
    buttons ["Save", "Cancel"]
]
```

## Examples of Crystalline Flexibility

### Example 1: Simple Confirmation

All these produce identical UI:

```
# Formal
popup "Confirm" [
    text "Delete this file?",
    buttons ["Yes", "No"]
]

# Natural
[Confirm: Delete this file? Yes|No]

# Symbolic
popup "⚠️" [
    "Delete this file?",
    ✓ ✗
]

# Conversational
confirm "Delete this file?" with Yes or No
```

### Example 2: Settings Panel

```
# Technical style
popup "Settings" [
    slider "volume" 0..100 @75,
    checkbox "notifications" @true,
    choice "theme" ["Light", "Dark"],
    buttons ["Save", "Cancel"]
]

# Natural style
[Settings:
    volume: 0 to 100 (currently 75)
    notifications: ✓
    theme: Light | Dark
    Save or Cancel
]

# Mixed style with conditions
popup "Settings" [
    Volume from 0 to 100 = 75,
    ✓ Enable notifications,
    when notifications [
        Sound: Low|Medium|High
    ],
    Theme: Light/Dark,
    [Save] [Cancel]
]
```

## Implementation Strategy

1. **Alias Resolution Layer**: Map all aliases to canonical form during parsing
2. **Format Detection**: Analyze input to determine which format is being used
3. **Error Recovery Pipeline**: Apply fixes in order: quotes → types → typos → structure
4. **Semantic Normalization**: Convert all variations to consistent AST
5. **Backward Compatibility**: Maintain support for current syntax

This design achieves true crystalline expressiveness while maintaining semantic precision.