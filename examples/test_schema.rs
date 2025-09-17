//! Example demonstrating use of schema functions as a library consumer

use popup_mcp::{get_input_schema, get_popup_tool_schema, get_schema_description};

fn main() {
    println!("Testing schema generation functions:\n");

    // Get the complete MCP tool schema
    let tool_schema = get_popup_tool_schema();
    println!("Complete tool schema:");
    println!("{}\n", serde_json::to_string_pretty(&tool_schema).unwrap());

    // Get just the input schema
    let input_schema = get_input_schema();
    println!("Input schema only:");
    println!("{}\n", serde_json::to_string_pretty(&input_schema).unwrap());

    // Get human-readable description
    let description = get_schema_description();
    println!("Human-readable description:");
    println!("{}", description);
}
