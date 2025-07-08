use super::*;
use pest::Parser;

#[test]
fn test_title_parsing() {
    // Test title rule
    let test_cases = vec![
        "Test",
        "Test:",  // This might consume the colon!
        "Delete File?",
    ];
    
    for input in test_cases {
        println!("\nTesting title with: '{}'", input);
        match PopupParser::parse(Rule::title, input) {
            Ok(pairs) => {
                println!("  Success!");
                for pair in pairs {
                    debug_print_pair(&pair, 1);
                }
            }
            Err(e) => {
                println!("  Failed: {}", e);
            }
        }
    }
}

#[test]
fn test_structured_popup_manually() {
    // Manually construct what structured_popup expects
    let input = "Test:\n  [OK]";
    println!("\nManually testing structured_popup components:");
    
    // The structured_popup rule expects: title ~ ws? ~ ":" ~ ws? ~ NEWLINE? ~ body
    // So title should NOT include the colon
    
    // Check if "Test:" matches title (it shouldn't!)
    match PopupParser::parse(Rule::title, "Test:") {
        Ok(pairs) => {
            println!("  'Test:' matched title - this is the problem!");
            for pair in pairs {
                debug_print_pair(&pair, 2);
            }
        }
        Err(_) => {
            println!("  'Test:' correctly did NOT match title");
        }
    }
}

fn debug_print_pair(pair: &pest::iterators::Pair<Rule>, indent: usize) {
    let indent_str = "  ".repeat(indent);
    println!("{}Rule: {:?}, Text: '{}'", indent_str, pair.as_rule(), pair.as_str());
    for inner in pair.clone().into_inner() {
        debug_print_pair(&inner, indent + 1);
    }
}