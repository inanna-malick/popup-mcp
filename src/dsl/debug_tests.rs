use super::*;
use pest::Parser;

#[test]
fn test_grammar_parsing_directly() {
    // Test parsing with the grammar directly
    let test_cases = vec![
        ("Test:", Rule::title),
        ("Test:\n  [OK]", Rule::structured_popup),
        ("confirm \"Test\" with OK", Rule::natural_popup),
        ("[Test: OK]", Rule::bracket_popup),
    ];
    
    for (input, rule) in test_cases {
        println!("\nTesting '{}' with rule {:?}", input, rule);
        match PopupParser::parse(rule, input) {
            Ok(pairs) => {
                println!("  Success!");
                for pair in pairs {
                    println!("  Matched: {:?}", pair.as_str());
                }
            }
            Err(e) => {
                println!("  Failed: {}", e);
            }
        }
    }
}

#[test]
fn test_popup_rule() {
    // Test the top-level popup rule
    let test_cases = vec![
        "Test:\n  [OK]",
        "confirm \"Test\" with OK",
        "[Test: OK]",
    ];
    
    for input in test_cases {
        println!("\nTesting popup rule with: '{}'", input);
        match PopupParser::parse(Rule::popup, input) {
            Ok(pairs) => {
                println!("  Success!");
                for pair in pairs {
                    println!("  Rule: {:?}, Text: '{}'", pair.as_rule(), pair.as_str());
                    for inner in pair.into_inner() {
                        println!("    Rule: {:?}, Text: '{}'", inner.as_rule(), inner.as_str());
                    }
                }
            }
            Err(e) => {
                println!("  Failed: {}", e);
            }
        }
    }
}

#[test]
fn test_structured_popup_components() {
    // Test individual components
    println!("\nTesting title:");
    match PopupParser::parse(Rule::title, "Test") {
        Ok(_) => println!("  Title parsing works"),
        Err(e) => println!("  Title parsing failed: {}", e),
    }
    
    println!("\nTesting body with button:");
    match PopupParser::parse(Rule::body, "  [OK]") {
        Ok(_) => println!("  Body parsing works"),
        Err(e) => println!("  Body parsing failed: {}", e),
    }
    
    println!("\nTesting element:");
    match PopupParser::parse(Rule::element, "[OK]") {
        Ok(_) => println!("  Element parsing works"),
        Err(e) => println!("  Element parsing failed: {}", e),
    }
    
    println!("\nTesting buttons:");
    match PopupParser::parse(Rule::buttons, "[OK]") {
        Ok(_) => println!("  Buttons parsing works"),
        Err(e) => println!("  Buttons parsing failed: {}", e),
    }
}

#[test]
fn test_whitespace_handling() {
    // Test if whitespace is handled correctly
    let inputs = vec![
        "Test:\n  [OK]",
        "Test:\n[OK]",
        "Test: [OK]",
        "Test:\n\t[OK]",
        "Test:\n    [OK]",
    ];
    
    for input in inputs {
        println!("\nTesting whitespace with: '{}'", input.escape_debug());
        match PopupParser::parse(Rule::structured_popup, input) {
            Ok(_) => println!("  Success!"),
            Err(e) => println!("  Failed: {}", e),
        }
    }
}

#[test]
fn test_minimal_structured() {
    // Absolute minimal test
    let input = "T:\n  [O]";
    println!("\nTesting minimal: '{}'", input);
    
    match PopupParser::parse(Rule::popup, input) {
        Ok(pairs) => {
            println!("Success!");
            for pair in pairs {
                debug_print_pair(&pair, 0);
            }
        }
        Err(e) => {
            println!("Failed: {}", e);
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

#[test]
fn test_text_value_parsing() {
    // Test text_value rule specifically
    let test_cases = vec![
        "Test",
        "Test:",
        "\"Test\"",
        "'Test'",
        "Test with spaces",
    ];
    
    for input in test_cases {
        println!("\nTesting text_value with: '{}'", input);
        match PopupParser::parse(Rule::text_value, input) {
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