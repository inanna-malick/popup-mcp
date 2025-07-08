# Popup-MCP DSL: Crystalline Expression Language

The Popup-MCP DSL has been redesigned as a **Crystalline Expression Language** - offering multiple natural ways to express the same UI elements while maintaining semantic precision.

## Key Features

### 🔬 Semantic Alias System
Every widget type supports multiple aliases that reflect different mental models:

```popup
# All create checkboxes:
checkbox "Notifications"
toggle "Dark Mode"  
switch "Auto-save"
bool "Debug"
✓ Enabled

# All create sliders:
slider "Volume" 0..100
range "Brightness" 0..255
scale "Speed" 1..10
gauge "Progress" 0..100
```

### 🎀 Natural Language Patterns
Express UI elements in conversational syntax:

```popup
Settings:
  Volume from 0 to 100 starting at 75
  Brightness between 0 and 255
  Theme: Light | Dark
  ✓ Enable notifications
  
  when notifications:
    Sound level: Low | Medium | High
    
  [Save | Cancel]
```

### 💠 Error Recovery & Tolerance
The parser automatically fixes common mistakes:

```popup
# Missing quotes → auto-added
checkbox enabled → checkbox "enabled"

# Typos → auto-corrected  
chekbox "test" → checkbox "test"
choise "theme" → choice "theme"

# Missing elements → smart defaults
slider 0..100 → slider "Value" 0..100 @50
```

## Widget Reference

### Checkbox/Boolean
**Aliases:** `checkbox`, `check`, `tick`, `toggle`, `switch`, `bool`, `boolean`, `yes/no`, `enabled`

```popup
# Explicit syntax
checkbox "Dark Mode" 
toggle "Notifications" default=true

# Inferred from value
Dark Mode: yes
Notifications: ✓
Auto-save: on

# Symbolic
✓ Subscribe to newsletter
☐ Share analytics
[x] Remember me
```

### Slider/Range
**Aliases:** `slider`, `range`, `scale`, `numeric`, `dial`, `gauge`, `level`, `meter`

```popup
# Explicit syntax
slider "Volume" 0..100 @75
range "Brightness" 0..255 default:200

# Inferred from value  
Volume: 0-100 = 75
Speed: 1..10

# Natural language
Volume from 0 to 100 starting at 75
Brightness between 0 and 255 (default: 200)
```

### Textbox/Input
**Aliases:** `textbox`, `input`, `field`, `entry`, `textarea`, `prompt`, `write`

```popup
# Explicit syntax
textbox "Username" @Enter username
input "Email" @user@example.com

# Inferred from @ prefix
Username: @Enter your name
Comments: @Tell us what you think
```

### Choice/Select
**Aliases:** `choice`, `select`, `dropdown`, `pick`, `choose`, `radio`

```popup
# Explicit syntax
choice "Theme" ["Light", "Dark", "Auto"]
select "Country" ["USA", "Canada", "Mexico"]

# Inferred from options
Theme: Light | Dark | Auto
Size: Small / Medium / Large
```

### Multiselect
**Aliases:** `multiselect`, `multi`, `checklist`, `tags`, `options`

```popup
# Explicit syntax
multiselect "Features" ["A", "B", "C"]
tags "Categories" ["Work", "Personal", "Urgent"]

# Inferred from brackets
Features: [Email, SMS, Push]
Skills: [Python, Rust, JavaScript]
```

### Buttons
**Aliases:** `buttons`, `actions`, `btns`, `controls`

```popup
# Multiple styles
buttons: [Save, Cancel]
[OK | Cancel]
→ Continue
Save or Cancel

# Separator style
---
Submit | Reset | Back
```

## Conditional Logic

### Basic Conditions
```popup
# Simple conditions
when notifications:
  Sound: on
  
if Theme = Dark:
  Contrast: High | Normal
  
unless Beta:
  > Stable features only
```

### Value References
```popup
# Multiple ways to reference values
when volume > 50:          # Direct reference
when $volume > 50:         # $ prefix
when {volume} > 50:        # Braces
when #tasks > 3:           # Count syntax
when tasks.count > 3:      # Property access
```

### Compound Conditions
```popup
when notifications and sound:
  Volume: 0-100
  
if Theme = Dark && HighContrast:
  ! UI may look different
  
unless Free or Trial:
  Premium features: ✓
```

## Format Flexibility

### Structured Format
```popup
Settings:
  Volume: 0-100
  Theme: Light | Dark
  [Save | Cancel]
```

### Inline Format
```popup
[Quick Setup: Volume: 0-100, Theme: Light|Dark, OK or Cancel]
```

### Natural Language
```popup
confirm "Delete file?" with Yes or No
```

### Mixed Styles
```popup
User Profile:
  Name: @Full name
  Email: @user@example.com
  
  Plan: Free | Pro | Enterprise
  
  when Plan = Pro:
    Seats from 1 to 10 = 1
    ✓ Priority support
    
  → Continue
```

## Comments

```popup
Settings:
  # Single line comment
  Volume: 0-100  # Inline comment
  
  // C-style comment
  Theme: Light | Dark
  
  /* Multi-line
     comment */
  Debug: no
  
  [Save]
```

## Best Practices

1. **Choose the style that feels natural** - The DSL supports multiple paradigms
2. **Mix and match** - You can combine different syntax styles in one popup
3. **Let type inference work** - The parser is smart about detecting widget types
4. **Don't worry about quotes** - They're added automatically when needed
5. **Use semantic aliases** - Pick the alias that best describes your intent

## Migration from Old Syntax

The new DSL maintains compatibility with core concepts while being more flexible:

| Old Syntax | New Syntax Options |
|------------|-------------------|
| `popup "Title" [...]` | `Title:` or `[Title: ...]` |
| `slider "Volume" min:0 max:100` | `Volume: 0-100` or `Volume from 0 to 100` |
| `checkbox "Enabled" default:true` | `Enabled: yes` or `✓ Enabled` |
| `if checked("x")` | `when x:` or `if x:` |

## Examples

See [`examples/comprehensive_dsl_demo.popup`](../examples/comprehensive_dsl_demo.popup) for a complete demonstration of all features.