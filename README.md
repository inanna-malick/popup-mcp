# popup-mcp

An MCP (Model Context Protocol) server that enables AI assistants to create native GUI popup windows using a simple, natural domain-specific language (DSL).

## Features

- **Simple, natural syntax** - Write popups like you think
- **Smart widget detection** - Parser intelligently infers widget types from values
- **Conditional UI** - Show/hide elements based on user selections
- **Rich controls** - Sliders, checkboxes, choices, multiselect, text inputs
- **Enhanced widgets** - Sliders show percentages, multiselect has All/None buttons, text fields show character count
- **Message formatting** - Info, warnings, questions with automatic icons
- **Keyboard navigation** - Full support for Tab, Arrow keys, Escape
- **Native GUI** - Fast, responsive popups using egui
- **JSON output** - Clean, structured results
- **MCP integration** - Works with Claude Desktop and other AI assistants
- **Force Yield** - Every popup includes an automatic escape button

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

| Value Pattern | Widget Type | Features | Example |
|--------------|-------------|----------|---------|
| `0-100`, `0..100`, `0 to 100` | Slider | Shows value & percentage | `Volume: 0-100` |
| `yes`, `no`, `true`, `false`, `✓`, `[x]` | Checkbox | Keyboard toggle with Space | `Enabled: yes` |
| `A | B | C` | Choice (radio) | Arrow key navigation | `Size: Small | Medium | Large` |
| `[A, B, C]` | Multiselect | All/None buttons, count display | `Tags: [Bug, Feature, Urgent]` |
| `@placeholder` | Text input | Character count display | `Name: @Enter your name` |
| Anything else | Text display | Static text | `Status: Active` |

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

## Conditional UI

Create dynamic interfaces that adapt based on user selections:

### Basic Conditional

```
Advanced Settings
Show advanced: no
[if Show advanced] {
  Debug level: 0-10
  Log file: @/var/log/debug.log
  Verbose output: yes
}
[Apply | Cancel]
```

### Conditional with Comparison

```
Theme Settings
Theme: Light | Dark | Auto
[if Theme = Dark] {
  Contrast: Normal | High
  Blue light filter: 0-100 = 30
}
[if Theme = Auto] {
  > Theme will change based on system settings
}
[Save | Reset]
```

### Multiselect Count Condition

```
Notification Settings
Alert types: [Email, SMS, Push, In-app]
[if Alert types > 2] {
  ! Warning: Too many alerts may be overwhelming
  Daily limit: 1-50 = 10
}
[if Alert types has Email] {
  Email address: @your@email.com
}
[Update | Cancel]
```

### Nested Conditionals

```
Developer Options
Enable dev mode: no
[if Enable dev mode] {
  API endpoint: Production | Staging | Local
  [if API endpoint = Local] {
    Local port: @8080
    Use HTTPS: no
  }
  Show debug panel: yes
}
[Apply | Reset]
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

# With a title
popup-mcp --title "My Popup" < my_popup.popup

# Or with echo
echo "confirm Save changes?\nYes or No" | popup-mcp

# Validate DSL without showing popup
echo "Volume: 0-100" | popup-mcp --validate
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
5. **Keyboard shortcuts**:
   - `Tab` / `Shift+Tab` - Navigate between widgets
   - `Arrow keys` - Navigate choices and multiselect options
   - `Space` - Toggle checkboxes and select options
   - `Enter` - Activate focused button
   - `Escape` - Cancel popup (returns "Cancel" as button)
6. **Conditional UI** - Use `[if condition] { }` to create dynamic interfaces
7. **Force Yield** - Every popup automatically includes this escape button

## Development

```bash
# Run tests
cargo test

# Run specific test module
cargo test dsl::simple_parser_tests

# Build
cargo build --release

# Build MCP server
cargo build --bin stdio_direct

# Install locally
cargo install --path .

# Run MCP server directly for testing
cargo run --bin stdio_direct
```

## Troubleshooting

### Popup doesn't appear
- Check that the DSL syntax is valid
- Ensure each element is on its own line
- Run with `--validate` flag to check parsing

### Widget not detected correctly
- Check the value pattern matches expected format
- For textboxes, prefix with `@` symbol
- For multiselect, use square brackets `[A, B, C]`

### Conditional not working
- Ensure condition references exact label name
- Check that closing brace `}` is on its own line
- For comparisons, use exact option text

### MCP integration issues
- Verify `stdio_direct` binary is in PATH
- Check Claude Desktop config points to correct binary
- Review server logs for error messages

## License

MIT