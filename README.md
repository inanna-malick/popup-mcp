# popup-mcp

An MCP (Model Context Protocol) server that enables AI assistants to create native GUI popup windows with conditional UI support using a simple domain-specific language (DSL).

## Features

- **Conditional UI Elements** (NEW!)
  - `if checked("name")` - Show content when checkbox is checked
  - `if selected("name", "value")` - Show content for specific choice selection
  - `if count("name") > N` - Show content based on multiselect count
- **Rich controls**: text, sliders, checkboxes, radio buttons, text inputs, groups, multiselect
- **Simple DSL** for defining popup layouts with conditional logic
- **Native GUI** using imgui-rs (immediate mode, cyberpunk aesthetic)
- **JSON output** of user selections
- **Automatic button validation** - ensures every popup has at least one button
- **MCP integration** for use with Claude Desktop and other AI assistants

## DSL Syntax

```
popup "Title" [
    text "Some explanation"
    
    slider "Energy Level" 0..10 default=5
    
    checkbox "Enable feature" default=true
    
    choice "Select option:" [
        "Option A",
        "Option B", 
        "Option C"
    ]
    
    # NEW: Conditional elements
    if selected("Select option:", "Option A") [
        text "You selected A!"
        checkbox "A-specific setting"
    ]
    
    # NEW: Multiselect widget
    multiselect "Active components:" [
        "Component 1",
        "Component 2",
        "Component 3"
    ]
    
    if count("Active components:") > 1 [
        text "Multiple components selected!"
    ]
    
    textbox "Your name:" placeholder="Enter name..."
    
    textbox "Notes:" rows=3
    
    group "Settings" [
        slider "Volume" 0..100
        checkbox "Mute"
    ]
    
    # REQUIRED: Every popup must have at least one button
    buttons ["OK", "Cancel", "Help"]
]
```

### Button Requirement

**Important**: Every popup MUST include at least one button to provide users with an exit path. If no buttons are defined, the parser automatically adds a "Continue" button with a warning message.

```
# This popup will get an automatic "Continue" button:
popup "Info" [
    text "System ready"
    # Warning: no buttons defined!
]

# Better - explicit button:
popup "Info" [
    text "System ready"
    buttons ["OK"]
]

# Best - meaningful action:
popup "Confirm" [
    text "Delete this file?"
    buttons ["Delete", "Cancel"]
]
```

## Usage

### As a CLI tool

```bash
# Read DSL from stdin
cat examples/simple.popup | cargo run

# Or pipe DSL directly
echo 'popup "Test" [text "Hello!" buttons ["OK"]]' | cargo run
```

### As an MCP server

1. Build the MCP server:
   ```bash
   cargo build --bin stdio_direct
   ```

2. Add to Claude Desktop config (~/.config/Claude/claude_desktop_config.json):
   ```json
   {
     "mcpServers": {
       "popup-mcp": {
         "command": "/path/to/popup-mcp/target/debug/stdio_direct",
         "args": [],
         "env": {}
       }
     }
   }
   ```

3. Restart Claude Desktop

4. Use the `popup` tool in Claude to create popups!

## Examples

See the `examples/` directory for sample popup definitions.

### Conditional UI Example

```
popup "Adaptive Interface" [
    choice "State:" ["Stuck", "Conflicted", "Exploring"]
    
    if selected("State:", "Stuck") [
        text ">>> MICRO-MOVEMENT PROTOCOL <<<"
        checkbox "Can stand up?"
        checkbox "Can get water?"
        
        if checked("Can stand up?") [
            text "Great! Next step: move to a different room"
        ]
    ]
    
    if selected("State:", "Conflicted") [
        multiselect "Active headmates:" [
            "[lotus] Body-Agent",
            "[temple] Order-Seeker",
            "[flower] Comfort-Seeker"
        ]
        
        if count("Active headmates:") > 2 [
            text "Complex negotiation needed"
            slider "Tension level" 0..10
        ]
    ]
    
    buttons ["Execute", "Defer"]
]
```

### Button Best Practices

1. **Always include explicit buttons** - Don't rely on the automatic fallback
2. **Use meaningful labels** - "Save"/"Cancel" instead of "OK"/"Cancel"
3. **Consider the context** - "Continue" for wizards, "Apply" for settings, "Done" for info
4. **Order matters** - Primary action first, then secondary, then cancel/abort
5. **Conditional buttons are risky** - If all buttons are in conditionals, add a root-level fallback

This creates a dynamic interface that changes based on user selections, perfect for:
- Multi-step wizards
- Contextual help systems
- Adaptive questionnaires
- State-dependent workflows

## Architecture

- `src/dsl/` - Pest-based DSL parser
- `src/gui/` - imgui-rs rendering engine  
- `src/models.rs` - Data structures
- `src/bin/stdio_direct.rs` - MCP server implementation

## License

MIT
