use super::*;
use crate::models::{Element};

#[test]
fn test_button_parsing_edge_cases() {
    // Test edge cases with "or" in button text
    let test_cases = vec![
        // Normal cases that should split
        (r#"confirm "Test" with Yes or No"#, vec!["Yes", "No"]),
        (r#"confirm "Test" with Accept or Reject"#, vec!["Accept", "Reject"]),
        
        // Edge cases with "or" in quoted strings (should NOT split)
        (r#"Test:
  buttons: ["More or Less", "Cancel"]"#, vec!["More or Less", "Cancel"]),
        
        // Multiple "or" separators
        (r#"Test:
  OK or Cancel or Maybe"#, vec!["OK", "Cancel", "Maybe"]),
        
        // Mixed separators
        (r#"Test:
  [Save | Discard or Cancel]"#, vec!["Save", "Discard", "Cancel"]),
        
        // Words containing "or" should work
        (r#"Test:
  buttons: [Store, Ignore, Forward]"#, vec!["Store", "Ignore", "Forward"]),
        
        // Test single word buttons
        (r#"Test:
  [Forward]"#, vec!["Forward"]),
    ];
    
    for (dsl, expected) in test_cases {
        eprintln!("\nTesting: {}", dsl);
        let result = parse_popup_dsl(dsl);
        assert!(result.is_ok(), "Failed to parse: {}", dsl);
        
        let popup = result.unwrap();
        let buttons = popup.elements.iter()
            .find_map(|e| match e {
                Element::Buttons(labels) => Some(labels),
                _ => None,
            })
            .expect("Should have buttons");
        
        let user_buttons: Vec<&str> = buttons.iter()
            .filter(|b| b.as_str() != "Force Yield")
            .map(|s| s.as_str())
            .collect();
        
        eprintln!("  Parsed: {:?}", user_buttons);
        assert_eq!(user_buttons.len(), expected.len(), 
            "Expected {} buttons, got {:?}", expected.len(), user_buttons);
        
        for (i, expected_button) in expected.iter().enumerate() {
            assert_eq!(user_buttons.get(i).copied(), Some(*expected_button),
                "Button {} should be '{}'", i, expected_button);
        }
    }
}

#[test]
fn test_or_word_boundary() {
    // Test that words containing "or" are handled correctly
    let test_cases = vec![
        ("Forward", vec!["Forward"]),
        ("Store", vec!["Store"]),
        ("Ignore", vec!["Ignore"]),
        ("More", vec!["More"]),
        ("Or", vec!["Or"]),  // "Or" by itself should be one button
        ("OR", vec!["OR"]),  // Capital OR
    ];
    
    for (button_text, expected) in test_cases {
        let dsl = format!("Test:\n  buttons: [{}]", button_text);
        let result = parse_popup_dsl(&dsl).unwrap();
        
        let buttons = result.elements.iter()
            .find_map(|e| match e {
                Element::Buttons(labels) => Some(labels),
                _ => None,
            })
            .unwrap();
        
        let user_buttons: Vec<&str> = buttons.iter()
            .filter(|b| b.as_str() != "Force Yield")
            .map(|s| s.as_str())
            .collect();
        
        assert_eq!(user_buttons, expected, 
            "Word '{}' should parse as {:?}", button_text, expected);
    }
}

#[test]
fn test_whitespace_variations() {
    // Test different whitespace around "or"
    let test_cases = vec![
        ("Yes or No", vec!["Yes", "No"]),
        ("Yes  or  No", vec!["Yes", "No"]),
        ("Yes\tor\tNo", vec!["Yes", "No"]),
        ("Yes   or   No", vec!["Yes", "No"]),
    ];
    
    for (input, expected) in test_cases {
        let dsl = format!(r#"confirm "Test" with {}"#, input);
        let result = parse_popup_dsl(&dsl);
        
        assert!(result.is_ok(), "Failed to parse with input: '{}'", input);
        
        let popup = result.unwrap();
        let buttons = popup.elements.iter()
            .find_map(|e| match e {
                Element::Buttons(labels) => Some(labels),
                _ => None,
            })
            .unwrap();
        
        let user_buttons: Vec<&str> = buttons.iter()
            .filter(|b| b.as_str() != "Force Yield")
            .map(|s| s.as_str())
            .collect();
        
        assert_eq!(user_buttons, expected, 
            "Input '{}' should parse to {:?}", input, expected);
    }
}

#[test]
fn test_case_sensitivity() {
    // Test case variations of "or"
    let test_cases = vec![
        ("Yes or No", vec!["Yes", "No"]),
        ("Yes OR No", vec!["Yes OR No"]),  // Capital OR should NOT split
        ("Yes Or No", vec!["Yes Or No"]),  // Capital Or should NOT split
    ];
    
    for (input, expected) in test_cases {
        let dsl = format!(r#"confirm "Test" with {}"#, input);
        let result = parse_popup_dsl(&dsl).unwrap();
        
        let buttons = result.elements.iter()
            .find_map(|e| match e {
                Element::Buttons(labels) => Some(labels),
                _ => None,
            })
            .unwrap();
        
        let user_buttons: Vec<&str> = buttons.iter()
            .filter(|b| b.as_str() != "Force Yield")
            .map(|s| s.as_str())
            .collect();
        
        assert_eq!(user_buttons, expected, 
            "Input '{}' should parse to {:?}", input, expected);
    }
}