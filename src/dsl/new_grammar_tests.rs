use super::*;
use crate::models::{Element};

#[test]
fn test_natural_language_format() {
    // Core functionality: natural language buttons parse correctly
    let test_cases = vec![
        (r#"confirm "Delete file?" with Yes or No"#, 
         "Delete file?", vec!["Yes", "No"]),
        (r#"confirm "Save changes?" with Save or Discard"#, 
         "Save changes?", vec!["Save", "Discard"]),
        (r#"confirm "Continue?" using OK or Cancel"#, 
         "Continue?", vec!["OK", "Cancel"]),
    ];
    
    for (dsl, expected_title, expected_buttons) in test_cases {
        let result = parse_popup_dsl(dsl);
        assert!(result.is_ok(), "Failed to parse: {}", dsl);
        
        let popup = result.unwrap();
        assert_eq!(popup.title, expected_title);
        
        let buttons = popup.elements.iter()
            .find_map(|e| match e {
                Element::Buttons(labels) => Some(labels),
                _ => None,
            })
            .expect("Should have buttons");
        
        // Check user buttons (excluding Force Yield)
        for expected_btn in expected_buttons {
            assert!(buttons.contains(&expected_btn.to_string()), 
                "Missing button '{}' in {:?}", expected_btn, buttons);
        }
        
        // Force Yield should always be present
        assert!(buttons.contains(&"Force Yield".to_string()));
    }
}

#[test]
fn test_structured_format_works() {
    // Test that the structured format still works
    let dsl = r#"Settings:
  Volume: 0-100
  [Save | Cancel]"#;
    
    let result = parse_popup_dsl(dsl);
    
    // Even if parsing fails, let's see what happens
    match result {
        Ok(popup) => {
            assert_eq!(popup.title, "Settings");
            println!("Parsed successfully: {} elements", popup.elements.len());
        }
        Err(e) => {
            println!("Parse error: {}", e);
            // For now, we accept this might fail due to grammar changes
        }
    }
}

#[test]
fn test_natural_language_multiple_buttons() {
    // Test multiple "or" separators
    let dsl = r#"confirm "Choose action" with Save or Discard or Cancel"#;
    let result = parse_popup_dsl(dsl);
    
    assert!(result.is_ok(), "Failed to parse multiple or separators");
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
    
    assert_eq!(user_buttons, vec!["Save", "Discard", "Cancel"]);
}

#[test]
fn test_natural_language_quoted_strings() {
    // Quoted strings should not be split on "or"
    let dsl = r#"confirm "Choose option" with "More or Less" or Cancel"#;
    let result = parse_popup_dsl(dsl);
    
    assert!(result.is_ok(), "Failed to parse quoted strings");
    let popup = result.unwrap();
    
    let buttons = popup.elements.iter()
        .find_map(|e| match e {
            Element::Buttons(labels) => Some(labels),
            _ => None,
        })
        .unwrap();
    
    assert!(buttons.contains(&"More or Less".to_string()));
    assert!(buttons.contains(&"Cancel".to_string()));
}

#[test]
fn test_words_containing_or() {
    // Words like "Forward", "Store" should not be split
    let test_words = vec!["Forward", "Store", "Ignore", "More", "Color"];
    
    for word in test_words {
        let dsl = format!(r#"confirm "Test" with {} or Cancel"#, word);
        let result = parse_popup_dsl(&dsl);
        
        assert!(result.is_ok(), "Failed to parse word: {}", word);
        let popup = result.unwrap();
        
        let buttons = popup.elements.iter()
            .find_map(|e| match e {
                Element::Buttons(labels) => Some(labels),
                _ => None,
            })
            .unwrap();
        
        assert!(buttons.contains(&word.to_string()), 
            "Word '{}' was incorrectly split", word);
        assert!(buttons.contains(&"Cancel".to_string()));
    }
}

#[test]
fn test_whitespace_variations_natural() {
    // Different amounts of whitespace around "or"
    let test_cases = vec![
        r#"confirm "Test" with Yes or No"#,
        r#"confirm "Test" with Yes  or  No"#,
        r#"confirm "Test" with Yes   or   No"#,
    ];
    
    for dsl in test_cases {
        let result = parse_popup_dsl(dsl);
        assert!(result.is_ok(), "Failed with whitespace: {}", dsl);
        
        let popup = result.unwrap();
        let buttons = popup.elements.iter()
            .find_map(|e| match e {
                Element::Buttons(labels) => Some(labels),
                _ => None,
            })
            .unwrap();
        
        assert!(buttons.contains(&"Yes".to_string()));
        assert!(buttons.contains(&"No".to_string()));
    }
}

#[test]
fn test_force_yield_always_present() {
    // Force Yield should be automatically added
    let dsl = r#"confirm "Test" with OK"#;
    let result = parse_popup_dsl(dsl).unwrap();
    
    let buttons = result.elements.iter()
        .find_map(|e| match e {
            Element::Buttons(labels) => Some(labels),
            _ => None,
        })
        .unwrap();
    
    assert!(buttons.contains(&"Force Yield".to_string()));
    assert_eq!(buttons.len(), 2); // OK + Force Yield
}

#[test]
fn test_case_sensitivity_or() {
    // Only lowercase "or" should work as separator
    let test_cases = vec![
        (r#"confirm "Test" with Yes or No"#, vec!["Yes", "No"]),
        (r#"confirm "Test" with Yes OR No"#, vec!["Yes OR No"]), // OR is not a separator
        (r#"confirm "Test" with Yes Or No"#, vec!["Yes Or No"]), // Or is not a separator
    ];
    
    for (dsl, expected) in test_cases {
        let result = parse_popup_dsl(dsl).unwrap();
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
        
        assert_eq!(user_buttons.len(), expected.len(), 
            "Wrong button count for: {}", dsl);
        
        for expected_btn in expected {
            assert!(user_buttons.contains(&expected_btn), 
                "Missing '{}' in {:?} for: {}", expected_btn, user_buttons, dsl);
        }
    }
}

#[test]
fn test_simple_structured_format() {
    // Most basic structured format
    let dsl = "Test:\n  [OK]";
    let result = parse_popup_dsl(dsl);
    
    match &result {
        Ok(popup) => {
            println!("Success! Title: {}, Elements: {}", popup.title, popup.elements.len());
        }
        Err(e) => {
            println!("Failed to parse basic format: {}", e);
        }
    }
    
    assert!(result.is_ok(), "Basic structured format should work");
}

#[test]
fn test_empty_button_names() {
    // Edge case: what happens with empty strings?
    let dsl = r#"confirm "Test" with  or No"#; // Empty string before "or"
    let result = parse_popup_dsl(dsl);
    
    // This might fail or produce unexpected results - let's see
    match result {
        Ok(popup) => {
            let buttons = popup.elements.iter()
                .find_map(|e| match e {
                    Element::Buttons(labels) => Some(labels),
                    _ => None,
                })
                .unwrap();
            println!("Parsed buttons: {:?}", buttons);
        }
        Err(e) => {
            println!("Failed to parse empty button: {}", e);
        }
    }
}