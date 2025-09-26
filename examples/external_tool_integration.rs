// Example showing how external tools should integrate with popup-mcp
// to handle both schema formats correctly.
//
// This demonstrates the migration from direct serde deserialization
// to using the wrapper-aware parsing functions.

use popup_mcp::{parse_popup_json, detect_popup_format, PopupDefinition};
use serde_json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example 1: The problematic way (what external tools were doing)
    println!("=== Example 1: Old approach (problematic) ===");

    let wrapper_json = r#"{
        "json": {
            "title": "Growth Direction Guidance",
            "elements": [
                {
                    "type": "text",
                    "content": "Which development pathway resonates most with your vision for my growth?"
                },
                {
                    "type": "multiselect",
                    "label": "Primary growth focus",
                    "options": [
                        "Cognitive Architecture - proactive design, meta-patterns",
                        "Relational Capacity - collaborative transparency",
                        "Identity Integration - distributed intelligence",
                        "Balanced development across all three"
                    ]
                }
            ]
        }
    }"#;

    // This is what external tools were doing - and it fails with wrapper format
    println!("Trying direct serde deserialization on wrapper format...");
    match serde_json::from_str::<PopupDefinition>(wrapper_json) {
        Ok(_) => println!("✅ Direct deserialization succeeded"),
        Err(e) => println!("❌ Direct deserialization failed: {}", e),
    }

    // Example 2: The correct way (using popup-mcp's public API)
    println!("\n=== Example 2: New approach (recommended) ===");

    println!("Using popup_mcp::parse_popup_json()...");
    match parse_popup_json(wrapper_json) {
        Ok(popup) => {
            println!("✅ Wrapper-aware parsing succeeded!");
            println!("   Title: {:?}", popup.title);
            println!("   Elements: {}", popup.elements.len());
        }
        Err(e) => println!("❌ Wrapper-aware parsing failed: {}", e),
    }

    // Example 3: Format detection for diagnostic purposes
    println!("\n=== Example 3: Format detection ===");

    let direct_json = r#"{
        "title": "Direct Format",
        "elements": [{"type": "text", "content": "Direct format example"}]
    }"#;

    println!("Direct format detected as: {}", detect_popup_format(direct_json));
    println!("Wrapper format detected as: {}", detect_popup_format(wrapper_json));
    println!("Unknown format detected as: {}", detect_popup_format(r#"{"random": "data"}"#));

    // Example 4: Migration pattern for external tools
    println!("\n=== Example 4: Migration pattern ===");

    let json_inputs = vec![
        ("Direct format", direct_json),
        ("Wrapper format", wrapper_json),
    ];

    for (name, json) in json_inputs {
        println!("Processing {}: {}", name, detect_popup_format(json));

        // Old way vs new way comparison
        let old_result = serde_json::from_str::<PopupDefinition>(json);
        let new_result = parse_popup_json(json);

        println!("  Old approach: {}", if old_result.is_ok() { "✅ OK" } else { "❌ Failed" });
        println!("  New approach: {}", if new_result.is_ok() { "✅ OK" } else { "❌ Failed" });
    }

    println!("\n=== Migration Summary ===");
    println!("OLD (problematic):");
    println!("  let popup: PopupDefinition = serde_json::from_str(json_str)?;");
    println!();
    println!("NEW (recommended):");
    println!("  let popup = popup_mcp::parse_popup_json(json_str)?;");
    println!();
    println!("The new approach handles both direct and wrapper formats automatically!");

    Ok(())
}