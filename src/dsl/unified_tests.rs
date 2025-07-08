use super::*;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "unified.pest"]
pub struct UnifiedParser;

#[test]
fn test_unified_natural_language() {
    // Natural language style
    let inputs = vec![
        "confirm Delete file? with Yes or No",
        "Delete file? with Yes or No",
        "Save changes? with Save or Discard or Cancel",
    ];
    
    for input in inputs {
        println!("\nTesting: '{}'", input);
        match UnifiedParser::parse(Rule::popup, input) {
            Ok(pairs) => {
                println!("  ✓ Parsed successfully");
                for pair in pairs {
                    debug_pair(&pair, 1);
                }
            }
            Err(e) => println!("  ✗ Failed: {}", e),
        }
    }
}

#[test]
fn test_unified_structured() {
    // Structured format
    let input = r#"Settings:
  Volume: 0-100 = 75
  Theme: Light | Dark
  Notifications: ✓
  [Save | Cancel]"#;
    
    println!("\nTesting structured format:");
    match UnifiedParser::parse(Rule::popup, input) {
        Ok(pairs) => {
            println!("  ✓ Parsed successfully");
            for pair in pairs {
                debug_pair(&pair, 1);
            }
        }
        Err(e) => println!("  ✗ Failed: {}", e),
    }
}

#[test]
fn test_unified_mixed_styles() {
    // Mix natural and structured elements
    let input = r#"confirm Save Settings?
  Volume: 0-100
  Theme: Light | Dark
  with Save or Cancel"#;
    
    println!("\nTesting mixed style:");
    match UnifiedParser::parse(Rule::popup, input) {
        Ok(pairs) => {
            println!("  ✓ Parsed successfully");
            for pair in pairs {
                debug_pair(&pair, 1);
            }
        }
        Err(e) => println!("  ✗ Failed: {}", e),
    }
}

#[test]
fn test_unified_flexibility() {
    // Various flexible formats
    let test_cases = vec![
        // Title with or without colon
        ("Test\n  [OK]", "no colon"),
        ("Test:\n  [OK]", "with colon"),
        
        // Natural language buttons in structured format
        ("Settings:\n  Volume: 0-100\n  Save or Cancel", "natural buttons in structured"),
        
        // Arrow button
        ("Quick Save:\n  → Save", "arrow button"),
        
        // Explicit buttons
        ("Form:\n  Name: @John\n  buttons: Submit or Cancel", "explicit buttons"),
    ];
    
    for (input, desc) in test_cases {
        println!("\nTesting {}: '{}'", desc, input.replace('\n', "\\n"));
        match UnifiedParser::parse(Rule::popup, input) {
            Ok(_) => println!("  ✓ Success"),
            Err(e) => println!("  ✗ Failed: {}", e),
        }
    }
}

#[test]
fn test_widget_inference() {
    // Test that widgets are inferred correctly
    let input = r#"Test:
  Slider: 0-100
  Checkbox: yes
  Choice: A | B | C
  Multi: [X, Y, Z]
  Input: @placeholder
  [OK]"#;
    
    println!("\nTesting widget inference:");
    match UnifiedParser::parse(Rule::popup, input) {
        Ok(pairs) => {
            println!("  ✓ Parsed");
            // Find the body and check widgets
            for pair in pairs {
                if pair.as_rule() == Rule::popup {
                    check_widgets(&pair);
                }
            }
        }
        Err(e) => println!("  ✗ Failed: {}", e),
    }
}

fn check_widgets(pair: &pest::iterators::Pair<Rule>) {
    for inner in pair.clone().into_inner() {
        if inner.as_rule() == Rule::body {
            for element in inner.into_inner() {
                if element.as_rule() == Rule::element {
                    for widget in element.into_inner() {
                        if widget.as_rule() == Rule::widget {
                            let mut parts = widget.into_inner();
                            let label = parts.next().unwrap();
                            let value = parts.next().unwrap();
                            println!("    Widget: {} -> {:?}", label.as_str(), value.as_rule());
                        }
                    }
                }
            }
        }
    }
}

fn debug_pair(pair: &pest::iterators::Pair<Rule>, indent: usize) {
    let indent_str = "  ".repeat(indent);
    println!("{}Rule: {:?}, Text: '{}'", indent_str, pair.as_rule(), pair.as_str());
    for inner in pair.clone().into_inner() {
        debug_pair(&inner, indent + 1);
    }
}