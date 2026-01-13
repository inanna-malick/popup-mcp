# popup-mcp

**Native GUI popups via MCP** - Display interactive popup windows from AI assistants through the Model Context Protocol.

Create rich dialogue trees with form elements (text, sliders, checkboxes, dropdowns, multiselect) and cascading conditional branches that adapt based on user selections.

## Installation

```bash
# Clone and install
git clone https://github.com/inanna-malick/popup-mcp.git
cd popup-mcp
cargo install --path crates/popup-gui
```

## Setup with Claude Desktop

```bash
# Add MCP server
claude mcp add popup --scope user -- popup --mcp-server

# Restart Claude Desktop
```

The `popup` tool will be available for creating GUI interactions.

## Quick Example

```bash
# Test a simple popup
echo '{"title": "Hello", "elements": [{"text": "World!"}]}' | popup --stdin

# Try example files
popup --file examples/simple_confirm.json
```

## Documentation

For complete documentation including JSON schema, element types, conditional visibility, templates, and examples:

**[Full Documentation](https://tidepool.leaflet.pub/3mcbegnuf2k2i)**

## Contributing

See `CLAUDE.md` for development guidance.

## License

MIT
