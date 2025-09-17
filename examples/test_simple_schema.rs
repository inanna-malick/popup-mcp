//! Example demonstrating the radically simplified schema functions

use popup_mcp::{get_simple_input_schema, get_simple_popup_tool_schema};

fn main() {
    println!("Testing radically simplified schema (text, textbox, multiselect only):\n");

    // Get the simplified tool schema
    let tool_schema = get_simple_popup_tool_schema();
    println!("Simplified tool schema:");
    println!("{}\n", serde_json::to_string_pretty(&tool_schema).unwrap());

    // Get just the simplified input schema
    let input_schema = get_simple_input_schema();
    println!("Simplified input schema only:");
    println!("{}", serde_json::to_string_pretty(&input_schema).unwrap());

    // Show what elements are supported
    println!("\n\nSupported elements in simplified schema:");
    println!("- text: Static text display");
    println!("- textbox: Text input field (with optional placeholder and rows)");
    println!("- multiselect: Multiple selection list");
    println!("\nExcluded from simplified schema:");
    println!("- checkbox, slider, choice (single select)");
    println!("- group, conditional");
}
