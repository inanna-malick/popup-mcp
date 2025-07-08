# popup-mcp

An MCP (Model Context Protocol) server that enables AI assistants to create native GUI popup windows using a simple, natural domain-specific language (DSL).

## Features

- **Simple, natural syntax** - Write popups like you think
- **Smart widget detection** - Parser intelligently infers widget types from values
- **Flexible formats** - Multiple ways to express the same thing
- **Rich controls** - Sliders, checkboxes, choices, multiselect, text inputs
- **Message formatting** - Info, warnings, questions with automatic icons
- **Native GUI** - Fast, responsive popups using egui
- **JSON output** - Clean, structured results
- **MCP integration** - Works with Claude Desktop and other AI assistants

## Installation

```bash
# Install from source
cargo install --path .

# Or install from git
cargo install --git https://github.com/yourusername/popup-mcp
```

### Claude Desktop Integration

Add to your Claude Desktop config (`~/Library/Application Support/Claude/claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "popup": {
      "command": "stdio_direct"
    }
  }
}
```

## Simple DSL Syntax

The new DSL is designed to be natural and forgiving. No complex syntax to remember!

### Basic Example

```
confirm Delete file?
Yes or No
```

### Settings Example

```
Settings
Volume: 0-100 = 75
Theme: Light | Dark
Notifications: yes
Auto-save: enabled
[Save | Cancel]
```

This automatically creates:
- Slider for Volume (detects range pattern)
- Choice for Theme (detects pipe-separated options)
- Checkbox for Notifications (detects boolean)
- Checkbox for Auto-save (detects boolean word)
- Buttons at the bottom

### Smart Widget Detection

The parser automatically detects widget types based on value patterns:

| Value Pattern | Widget Type | Example |
|--------------|-------------|---------|
| `0-100`, `0..100`, `0 to 100` | Slider | `Volume: 0-100` |
| `yes`, `no`, `true`, `false`, `✓`, `[x]` | Checkbox | `Enabled: yes` |
| `A | B | C` | Choice (radio) | `Size: Small | Medium | Large` |
| `[A, B, C]` | Multiselect | `Tags: [Bug, Feature, Urgent]` |
| `@placeholder` | Text input | `Name: @Enter your name` |
| Anything else | Text display | `Status: Active` |

### Button Formats

All these work:

```
[OK | Cancel]              # Bracket format
→ Continue                 # Arrow format
Save or Discard           # Natural language
```

### Messages with Icons

```
System Update
! Critical security update    # ⚠️ Warning
> Download size: 145MB       # ℹ️ Info
? Need help?                 # ❓ Question
• Restart required           # • Bullet
[Install | Later]
```

## More Examples

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
! Disk space low (95% full)
> CPU Usage: 45%
> Memory: 8GB / 16GB
Status: Healthy
Uptime: 24 days
[Refresh | Details]
```

### Quick Survey

```
Feedback
How was your experience?
Rating: 1-10 = 7
Would recommend: yes
Comments: @Optional feedback...
[Submit | Skip]
```

## Command Line Usage

Test your popups directly:

```bash
# From a file
popup-mcp < my_popup.popup

# Or with echo
echo "confirm Save changes?\nYes or No" | popup-mcp
```

## Output Format

The popup returns JSON with user selections:

```json
{
  "Volume": 75,
  "Theme": "Dark",
  "Notifications": true,
  "button": "Save"
}
```

## Tips

1. **Keep it simple** - The parser is smart and will figure out what you mean
2. **Use meaningful labels** - They become the keys in the JSON output
3. **One element per line** - Makes it easy to read and write
4. **Test interactively** - Use the command line to quickly test popups

## Development

```bash
# Run tests
cargo test

# Build
cargo build --release

# Install locally
cargo install --path .
```

## License

MIT