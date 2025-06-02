# popup-mcp

An MCP (Model Context Protocol) server that enables AI assistants to create native GUI popup windows using a simple domain-specific language (DSL).

## Features

- **Simple DSL** for defining popup layouts
- **Native GUI** using imgui-rs (immediate mode, cyberpunk aesthetic)
- **Rich controls**: text, sliders, checkboxes, radio buttons, text inputs, groups
- **JSON output** of user selections
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
    
    textbox "Your name:" placeholder="Enter name..."
    
    textbox "Notes:" rows=3
    
    group "Settings" [
        slider "Volume" 0..100
        checkbox "Mute"
    ]
    
    buttons ["OK", "Cancel", "Help"]
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

4. Use the `popup_show` tool in Claude to create popups!

## Examples

See the `examples/` directory for sample popup definitions.

## Architecture

- `src/dsl/` - Pest-based DSL parser
- `src/gui/` - imgui-rs rendering engine  
- `src/models.rs` - Data structures
- `src/bin/stdio_direct.rs` - MCP server implementation

## License

MIT
