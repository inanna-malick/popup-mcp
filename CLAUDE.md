# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Popup-MCP: Simple, Ergonomic GUI Popups for AI Communication

Popup-MCP is an MCP (Model Context Protocol) server that enables AI assistants to create native GUI popup windows using a simple, natural domain-specific language (DSL). It provides structured GUI popups for high-bandwidth human→AI communication.

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

# Run the main popup binary with input
echo "confirm Delete?\nYes or No" | cargo run

# Run with validation mode to see AST
echo "Settings\nVolume: 0-100" | cargo run -- --validate

# Run the spike CLI tool
cargo run --bin spike -- checkin
cargo run --bin spike -- show examples/demo_settings.popup

# Build specific binary
cargo build --bin stdio_direct
cargo build --bin spike
```

## Development Principles

- **ALWAYS write unit tests, not main methods**. This is a critical invariant. No main methods unless explicitly requested.
- Use the existing test patterns in `src/dsl/simple_parser_tests.rs` as examples
- Prefer iterators and for loops over manual iteration in Rust
- Avoid early optimizations without benchmarks
- **Wherever possible, write unit tests instead of using cargo run to test changes**

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

4. **CLI Tools**
   - `src/main.rs`: Main binary for testing popups via stdin
   - `src/bin/spike.rs`: Interactive CLI for structured feedback collection

### Key Design Decisions

- **Smart Widget Detection**: Instead of complex grammar rules, the parser intelligently infers widget types from value patterns (e.g., `0-100` → slider, `yes/no` → checkbox)
- **Error Tolerance**: Grammar focuses on structure while parser handles edge cases and provides helpful error messages
- **Force Yield**: Every popup automatically includes a "Force Yield" escape button
- **Natural Language Support**: Multiple ways to express the same concepts (e.g., buttons can be `[A|B]`, `A or B`, `→ Continue`)

### Testing Strategy

Tests are organized by functionality in `src/dsl/`:
- `simple_parser_tests.rs`: Core parser functionality tests
- `grammar_debug_tests.rs`: Grammar rule verification
- `edge_case_tests.rs`: Error handling and edge cases
- Use `#[test]` attributes for unit tests, following existing patterns

## Latest Implementation Status

**Grammar & Parser**: Fully redesigned with simplified, error-tolerant approach
- **Grammar**: `src/simple.pest` - Clean, minimal PEG grammar focused on structure
- **Parser**: `src/dsl/simple_parser.rs` - Smart widget detection with pattern recognition
- **Architecture**: Grammar handles syntax, parser handles semantics (separation of concerns)

**Recent Improvements**:
- ✅ Fixed natural language button parsing ("Yes or No" → ["Yes", "No"])
- ✅ Added markdown header support (`# Title`, `## Title`, etc.)
- ✅ Implemented intelligent widget detection from value patterns
- ✅ Added comprehensive test coverage with systematic unit tests
- ✅ Simplified grammar to avoid complex optional prefix matching
- ✅ Support for blank lines and natural multi-line text
- ✅ Robust error handling with helpful error messages

**Key Architecture Decision**: 
Instead of complex grammar rules for prefixes (confirm, markdown), we handle them in parser logic. This makes the grammar simpler and more maintainable while providing better error tolerance.


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

- `!` → Warning message
- `>` → Information message
- `?` → Question
- `•` → Bullet point