#!/bin/bash

# Test the MCP server with sample requests

echo "Testing popup-mcp MCP server..."

# Test initialize
echo '{"jsonrpc": "2.0", "method": "initialize", "params": {}, "id": 1}' | cargo run --bin stdio_direct 2>/dev/null | jq .

# Test tools/list
echo '{"jsonrpc": "2.0", "method": "tools/list", "params": {}, "id": 2}' | cargo run --bin stdio_direct 2>/dev/null | jq .

# Test popup_show with simple popup
echo "Testing popup_show tool..."
DSL='popup "Test Popup" [
    text "This is a test from MCP!"
    choice "Select option:" ["Option A", "Option B"]
    buttons ["OK", "Cancel"]
]'

JSON_DSL=$(echo "$DSL" | jq -Rs .)
REQUEST="{\"jsonrpc\": \"2.0\", \"method\": \"tools/call\", \"params\": {\"name\": \"popup_show\", \"arguments\": {\"dsl\": $JSON_DSL}}, \"id\": 3}"

echo "$REQUEST" | cargo run --bin stdio_direct 2>/dev/null | jq .
