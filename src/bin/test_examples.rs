use std::fs;
use popup_mcp::{parse_popup_dsl, render_popup};

fn main() {
    // List all example files
    let examples = [
        "simple.popup",
        "all_features.popup",
        "adaptive_mediation.popup",
        "compact_demo.popup",
    ];
    
    println!("Testing popup examples with egui backend:\n");
    
    for example in &examples {
        let path = format!("examples/{}", example);
        println!("Testing {}...", example);
        
        match fs::read_to_string(&path) {
            Ok(content) => {
                match parse_popup_dsl(&content) {
                    Ok(definition) => {
                        println!("  Title: {}", definition.title);
                        println!("  Elements: {} items", definition.elements.len());
                        
                        // Uncomment to actually show the popup:
                        /*
                        match render_popup(definition) {
                            Ok(result) => {
                                println!("  Result: {:?}\n", result);
                            }
                            Err(e) => {
                                eprintln!("  Error rendering: {}\n", e);
                            }
                        }
                        */
                        
                        println!("  ✓ Parsed successfully\n");
                    }
                    Err(e) => {
                        eprintln!("  Error parsing: {}\n", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("  Error reading file: {}\n", e);
            }
        }
    }
    
    // Show one example popup
    println!("\nShowing all_features.popup as demo...");
    if let Ok(content) = fs::read_to_string("examples/all_features.popup") {
        if let Ok(definition) = parse_popup_dsl(&content) {
            match render_popup(definition) {
                Ok(result) => {
                    println!("\nResult from popup:");
                    println!("{:#?}", result);
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            }
        }
    }
}