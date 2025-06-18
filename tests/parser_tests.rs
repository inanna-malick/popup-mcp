use popup_mcp::{parse_popup_dsl, models::*};
use pest::Parser;

#[derive(pest_derive::Parser)]
#[grammar = "src/popup.pest"]
struct TestParser;

#[test]
fn test_classic_syntax_basic() {
    let input = r#"popup "Test Title" [
        text "Hello World"
        buttons ["OK"]
    ]"#;
    
    let result = parse_popup_dsl(input).unwrap();
    assert_eq!(result.title, "Test Title");
    assert_eq!(result.elements.len(), 2);
}

#[test]
fn test_simplified_syntax_basic() {
    let input = "[Simple Test:
text \"Hello\"
buttons [\"OK\"]
]";
    
    let result = parse_popup_dsl(input).unwrap();
    assert_eq!(result.title, "Simple Test");
    assert_eq!(result.elements.len(), 2);
}

#[test]
fn test_simple_text_elements() {
    // Test without leading whitespace first
    let input = "[Form:
Name: [textbox]
buttons [\"Submit\"]
]";
    
    eprintln!("Testing input: '{}'", input);
    
    let result = parse_popup_dsl(input);
    match result {
        Ok(popup) => {
            assert_eq!(popup.title, "Form");
            assert!(popup.elements.len() >= 2);
            
            match &popup.elements[0] {
                Element::Textbox { label, .. } => assert_eq!(label, "Name"),
                other => panic!("Expected textbox, got {:?}", other),
            }
        },
        Err(e) => panic!("Parse failed: {}", e),
    }
}

#[test]
fn test_all_simple_text_widgets() {
    let input = "[Widget Test:
Display: [text]
Input: [textbox]
Toggle: [checkbox]
YesNo: [Y/N]
buttons [\"Done\"]
]";
    
    let result = parse_popup_dsl(input);
    match result {
        Ok(popup) => {
            assert_eq!(popup.title, "Widget Test");
            assert!(popup.elements.len() >= 5);
        },
        Err(e) => panic!("Parse failed: {}", e),
    }
}

#[test]
fn test_mixed_syntax() {
    let input = "[Mixed Form:
text \"Instructions here\"
Name: [textbox]
checkbox \"I agree\" @true
Options: [Y/N]
buttons [\"Submit\", \"Cancel\"]
]";
    
    let result = parse_popup_dsl(input);
    match result {
        Ok(popup) => {
            assert_eq!(popup.title, "Mixed Form");
            assert!(popup.elements.len() >= 5);
        },
        Err(e) => panic!("Parse failed: {}", e),
    }
}

#[test]
fn test_error_invalid_widget_type() {
    let input = "[Bad:
Name: [invalid]
]";
    
    let result = parse_popup_dsl(input);
    assert!(result.is_err());
}

#[test]
fn test_spike_example_minimal() {
    // Test the actual SPIKE format but simpler
    let input = "[SPIKE: Test
Thing 1: [textbox]
Ready: [Y/N]
]";
    
    let result = parse_popup_dsl(input);
    match result {
        Ok(popup) => {
            assert_eq!(popup.title, "SPIKE: Test");
            // Should have at least 2 elements + auto-added buttons
            assert!(popup.elements.len() >= 3);
        },
        Err(e) => panic!("Parse failed: {}", e),
    }
}

#[test]
fn test_whitespace_handling() {
    // Test various whitespace scenarios
    let inputs = vec![
        "[Test:Name: [textbox]]",  // No whitespace
        "[Test:\nName: [textbox]\n]",  // Newlines only
        "[Test: Name: [textbox]]",  // Space after colon
        "[Test:\n  Name: [textbox]\n]",  // Indented
    ];
    
    for (i, input) in inputs.iter().enumerate() {
        let result = parse_popup_dsl(input);
        match result {
            Ok(popup) => {
                assert_eq!(popup.title, "Test", "Failed at input {}", i);
                assert!(popup.elements.len() >= 1, "Failed at input {}", i);
            },
            Err(e) => panic!("Parse failed at input {}: {}", i, e),
        }
    }
}

#[test]
fn test_auto_force_yield() {
    let input = "[Test:
buttons [\"OK\"]
]";
    
    let result = parse_popup_dsl(input).unwrap();
    
    match &result.elements[0] {
        Element::Buttons(buttons) => {
            assert!(buttons.contains(&"Force Yield".to_string()), 
                "Force Yield should be automatically added");
        },
        _ => panic!("Expected buttons element"),
    }
}

#[test]
fn test_auto_buttons_when_missing() {
    let input = "[No Buttons:
text \"Just text\"
]";
    
    let result = parse_popup_dsl(input).unwrap();
    
    let has_buttons = result.elements.iter().any(|e| matches!(e, Element::Buttons(_)));
    assert!(has_buttons, "Buttons should be automatically added when missing");
}

#[test]
fn test_pest_bare_text_rule() {
    // Test bare text rule
    let test_cases = vec![
        "Name: [textbox]",
        "Ready: [Y/N]",
        "Toggle: [checkbox]",
        "Just plain text",
    ];
    
    for test_case in test_cases {
        let result = TestParser::parse(Rule::bare_text, test_case);
        assert!(result.is_ok(), "bare_text rule failed for: '{}'", test_case);
    }
}

#[test] 
fn test_pest_element_rule() {
    // Test that element rule recognizes bare_text
    let test_cases = vec![
        "Name: [textbox]",
        "Ready: [Y/N]",
    ];
    
    for test_case in test_cases {
        let result = TestParser::parse(Rule::element, test_case);
        assert!(result.is_ok(), "element rule failed for: '{}'", test_case);
    }
}

#[test]
fn test_pest_simplified_popup_rule() {
    let input = "[Test:\nName: [textbox]\n]";
    let result = TestParser::parse(Rule::simplified_popup, input);
    assert!(result.is_ok(), "simplified_popup rule failed: {:?}", result.err());
}