# Popup-MCP: Simple, Ergonomic GUI Popups for AI Communication

Popup-MCP provides structured GUI popups for high-bandwidth human→AI communication using a simple, natural syntax.

## Quick Setup

```bash
# Install globally
cargo install --git https://github.com/yourusername/popup-mcp

# Add to Claude Desktop config
# ~/Library/Application Support/Claude/claude_desktop_config.json
{
  "mcpServers": {
    "popup": {
      "command": "stdio_direct"
    }
  }
}
```

## New Simple Syntax

The DSL is designed to be natural and forgiving. Just write what feels right!

### Basic Format

```
Title
Element 1
Element 2
[Buttons]
```

### Natural Language Confirmation

```
confirm Delete file?
Yes or No
```

### Smart Widget Detection

The parser intelligently detects widget types from the value pattern:

```
Settings
Volume: 0-100
Theme: Light | Dark
Notifications: yes
Auto-save: enabled
Language: English
[Save | Cancel]
```

This creates:
- **Slider**: `Volume: 0-100` (range pattern)
- **Choice**: `Theme: Light | Dark` (pipe-separated options)
- **Checkbox**: `Notifications: yes` (boolean value)
- **Checkbox**: `Auto-save: enabled` (boolean word)
- **Text**: `Language: English` (no special pattern)
- **Buttons**: `[Save | Cancel]`

### Widget Patterns

| Pattern | Creates | Example |
|---------|---------|---------|
| `Label: 0-100` | Slider | `Volume: 0-100` |
| `Label: 0..100` | Slider | `Progress: 0..100` |
| `Label: 0 to 100` | Slider | `Score: 0 to 100` |
| `Label: 0-100 = 50` | Slider with default | `Brightness: 0-100 = 75` |
| `Label: yes/no/true/false` | Checkbox | `Subscribe: yes` |
| `Label: ✓/☐/[x]/[ ]` | Checkbox | `Complete: ✓` |
| `Label: A \| B \| C` | Choice | `Size: Small \| Medium \| Large` |
| `Label: [A, B, C]` | Multiselect | `Tags: [Work, Personal, Urgent]` |
| `Label: @hint` | Textbox | `Name: @Enter your name` |
| `Label: anything else` | Text display | `Status: Active` |

### Button Formats

All these formats work:

```
[OK | Cancel]              # Bracket format
→ Continue                 # Arrow format
Save or Discard           # Natural language
buttons: Submit or Reset  # Explicit format
```

### Messages

Use prefixes for different message types:

```
System Update
! Critical security update
> Download size: 145MB
? Need help with installation?
• Restart required
This is plain text
[Install | Later]
```

Prefixes:
- `!` → ⚠️ Warning
- `>` → ℹ️ Information
- `?` → ❓ Question
- `•` → Bullet point

## Complete Examples

### Simple Confirmation
```
confirm Delete file?
This action cannot be undone
Delete or Cancel
```

### User Profile Form
```
User Profile
Name: @Your name
Email: @your@email.com
Age: 18-100 = 25
Country: USA | Canada | UK | Other
Interests: [Sports, Music, Art, Tech]
Newsletter: yes
Bio: @Tell us about yourself...
→ Save Profile
```

### Status Report
```
System Status
! Disk space: Critical (95% full)
> CPU: 45% usage
> Memory: 8GB / 16GB
> Network: Connected
Status: Operational
Last check: 5 minutes ago
[Refresh | Details | Settings]
```

### Settings Panel
```
Preferences
Theme: Light | Dark | Auto
Font Size: 10-24 = 14
Show hints: yes
Auto-save: enabled
Save interval: 1-60 = 5
Backup location: @/path/to/backups
[Apply | Reset | Cancel]
```

## When to Use Popups

**Use conversational prompts for:**
- Simple yes/no questions
- Single text inputs
- Quick confirmations

**Use Popup when:**
- Multiple inputs needed at once
- Visual widgets communicate better than text
- You need structured data back
- The spatial layout helps understanding

## Integration Example

```python
def configure_settings():
    result = popup("""
    Settings
    Volume: 0-100 = 50
    Theme: Light | Dark
    Notifications: yes
    [Save | Cancel]
    """)
    
    # Result: {
    #   "Volume": 75,
    #   "Theme": "Dark", 
    #   "Notifications": true,
    #   "button": "Save"
    # }
    
    if result["button"] == "Save":
        save_settings(result)
```

## Key Features

1. **Smart widget detection** - The parser infers widget types from value patterns
2. **Flexible syntax** - Multiple ways to express the same thing
3. **Natural language** - Write like you think
4. **Automatic Force Yield** - Every popup gets an escape hatch
5. **Clean JSON output** - Easy to process results

## Tips

- Keep it simple - the parser is smart
- Don't worry about exact syntax - if it looks right, it probably works
- Test your popups interactively with `popup-mcp < yourfile.popup`
- Use meaningful labels - they become the JSON keys in results

## Troubleshooting

If a value isn't recognized as a widget:
- Check the pattern matches one of the widget types
- Remember that unrecognized patterns become text displays
- Use quotes if your text contains special characters

The parser is designed to be forgiving - when in doubt, just try it!