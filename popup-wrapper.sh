#!/bin/bash
# Wrapper to ensure complete process isolation for popup-mcp

# Get the directory of this script
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Try to find the popup-mcp binary
if [ -f "$DIR/target/release/popup-mcp" ]; then
    POPUP_BIN="$DIR/target/release/popup-mcp"
elif [ -f "$DIR/target/debug/popup-mcp" ]; then
    POPUP_BIN="$DIR/target/debug/popup-mcp"
else
    # Fallback to cargo run
    cd "$DIR" && exec cargo run --bin popup-mcp --quiet
fi

# Run in a clean environment to ensure no state sharing
exec env -i "$POPUP_BIN"
