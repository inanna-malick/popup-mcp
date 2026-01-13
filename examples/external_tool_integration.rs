//! Example showing how external tools should integrate with popup-mcp
//! using the strict canonical format.

use popup_mcp::{parse_popup_json, PopupDefinition};
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Strict Canonical Format Example ===");

    // The canonical format requires both "title" and "elements"
    let canonical_json = json!({
        "title": "Strict Format Demo",
        "elements": [
            {
                "text": "This must follow the exact PopupDefinition structure.",
                "id": "info_text"
            },
            {
                "check": "I understand",
                "id": "understand_check",
                "default": false
            }
        ]
    });

    let json_str = serde_json::to_string_pretty(&canonical_json)?;
    println!("Input JSON:\n{}", json_str);

    // Use popup_mcp::parse_popup_json() which now enforces strict parsing
    println!("\nParsing with popup_mcp::parse_popup_json()...");
    match parse_popup_json(&json_str) {
        Ok(popup) => {
            println!("✅ Parsing succeeded!");
            println!("   Title: {}", popup.title);
            println!("   Elements: {}", popup.elements.len());
        }
        Err(e) => println!("❌ Parsing failed: {}", e),
    }

    // Demonstrating that missing required fields will fail
    println!("\n=== Error Case: Missing Title ===");
    let missing_title = json!({
        "elements": [{"text": "Missing title"}]
    });
    let missing_title_str = serde_json::to_string(&missing_title)?;
    
    match parse_popup_json(&missing_title_str) {
        Ok(_) => println!("❌ Unexpectedly succeeded!"),
        Err(e) => println!("✅ Correctly failed: {}", e),
    }

    println!("\n=== Summary ===");
    println!("popup-mcp now uses a strict canonical format for robustness.");
    println!("Required fields: 'title' (string) and 'elements' (array).");
    println!("Recommended usage:");
    println!("  let popup = popup_mcp::parse_popup_json(json_str)?;");

    Ok(())
}
