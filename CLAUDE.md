# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Popup-MCP: Native GUI Popups via MCP

Popup-MCP is an MCP (Model Context Protocol) server that enables AI assistants to create native GUI popup windows using a simple, natural domain-specific language (DSL).

## Common Development Commands

```bash
# Build the project
cargo build --release

# Run all tests
cargo test

# Run tests with output (useful for debugging) 
cargo test -- --nocapture

# Run a specific test
cargo test test_simple_confirmation

# Run tests in a specific module
cargo test dsl::simple_parser_tests

# Build and install locally
cargo install --path .

# Test popup directly from command line
echo "confirm Delete?\nYes or No" | cargo run

# Test with validation mode to see AST
echo "Settings\nVolume: 0-100" | cargo run -- --validate

# Build MCP server binary
cargo build --bin popup-mcp-server

# Run MCP server directly
cargo run --bin stdio_direct
```

## High-Level Architecture

### Core Components

1. **DSL Parser** (`src/dsl/`)
   - `simple.pest`: PEG grammar defining the DSL syntax structure
   - `simple_parser.rs`: Smart parser that detects widget types from value patterns
   - Separation of concerns: Grammar handles syntax, parser handles semantic widget detection

2. **GUI Renderer** (`src/gui/`)
   - `mod.rs`: Main popup window logic using egui framework
   - `widget_renderers.rs`: Individual widget rendering implementations
   - Native GUI popups that return structured JSON results

3. **MCP Server** (`src/bin/stdio_direct.rs`)
   - Implements Model Context Protocol for AI assistant integration
   - Handles JSON-RPC communication with Claude Desktop
   - Provides `popup` tool for creating GUI popups from DSL

4. **Models** (`src/models.rs`)
   - Core data structures: `PopupDefinition`, `Element`, `PopupResult`
   - Supports various widget types: Slider, Checkbox, Choice, Multiselect, Textbox, Buttons

### Key Design Decisions

- **Smart Widget Detection**: Instead of complex grammar rules, the parser intelligently infers widget types from value patterns (e.g., `0-100` → slider, `yes/no` → checkbox)
- **Error Tolerance**: Grammar focuses on structure while parser handles edge cases and provides helpful error messages
- **Force Yield**: Every popup automatically includes a "Force Yield" escape button
- **Natural Language Support**: Multiple ways to express the same concepts (e.g., buttons can be `[A|B]`, `A or B`, `→ Continue`)
- **Title as Parameter**: Title is not part of the DSL grammar but passed as a parameter to the parser

### Testing Strategy

Tests are organized by functionality in `src/dsl/`:
- `simple_parser_tests.rs`: Core parser functionality tests
- `conditional_tests.rs`: Conditional logic testing (`[if condition] { ... }`)
- `grammar_debug_tests.rs`: Grammar rule verification
- `exact_ast_tests.rs`: Precise AST structure validation

## DSL Widget Pattern Reference

The parser automatically detects widget types from value patterns:

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

- `[OK | Cancel]` - Bracket format
- `→ Continue` - Arrow format  
- `Save or Discard` - Natural language
- `buttons: Submit or Reset` - Explicit format

### Message Prefixes

- `!` → Warning message (⚠️)
- `>` → Information message (ℹ️)
- `?` → Question (❓)
- `•` → Bullet point

### Conditional Blocks

```
Advanced: no
[if Advanced] {
  Debug level: 0-10
  Log file: @/tmp/debug.log
}
```

Supports conditions:
- Simple: `[if Advanced]`
- Negation: `[if not Advanced]`
- Comparison: `[if Value > 50]`, `[if Theme = Dark]`
- Count: `[if Tags > 2]`
- Has: `[if Tags has Important]`

## Development Principles

- **ALWAYS write unit tests, not main methods**. No main methods unless explicitly requested.
- Use the existing test patterns in `src/dsl/simple_parser_tests.rs` as examples
- Prefer iterators and for loops over manual iteration in Rust
- Avoid early optimizations without benchmarks
- **Wherever possible, write unit tests instead of using cargo run to test changes**