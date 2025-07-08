#[cfg(test)]
mod tests {
    use crate::dsl::parse_popup_dsl;

    #[test]
    fn test_unified_parser_examples() {
        let examples = vec![
            (
                "Natural language",
                r#"confirm Delete file? with Yes or No"#,
                "Delete file?",
                1, // Should have 1 element (buttons)
            ),
            (
                "Structured with colon",
                r#"Settings:
  Volume: 0-100 = 75
  Theme: Light | Dark
  Notifications: ✓
  [Save | Cancel]"#,
                "Settings",
                4, // Should have 4 elements
            ),
            (
                "Structured without colon",
                r#"Quick Setup
  Name: @Your name
  → Continue"#,
                "Quick Setup",
                2, // Should have 2 elements
            ),
            (
                "Mixed styles",
                r#"confirm Save changes?
  Modified files: 3
  Size: 1.2MB
  with Save or Discard"#,
                "Save changes?",
                3, // Should have 3 elements
            ),
        ];
        
        for (name, input, expected_title, expected_elements) in examples {
            println!("\nTesting: {}", name);
            match parse_popup_dsl(input) {
                Ok(popup) => {
                    assert_eq!(popup.title, expected_title, "Title mismatch for {}", name);
                    assert_eq!(
                        popup.elements.len(), 
                        expected_elements, 
                        "Element count mismatch for {}. Elements: {:?}", 
                        name,
                        popup.elements
                    );
                    
                    // Check Force Yield is added to buttons
                    for element in &popup.elements {
                        if let crate::models::Element::Buttons(labels) = element {
                            assert!(
                                labels.contains(&"Force Yield".to_string()),
                                "Force Yield not found in buttons for {}: {:?}",
                                name,
                                labels
                            );
                        }
                    }
                }
                Err(e) => {
                    panic!("Failed to parse {}: {}", name, e);
                }
            }
        }
    }

    #[test]
    fn test_natural_language_button_parsing() {
        let input = r#"confirm Delete? with Yes or No"#;
        
        match parse_popup_dsl(input) {
            Ok(popup) => {
                assert_eq!(popup.title, "Delete?");
                assert_eq!(popup.elements.len(), 1);
                
                if let crate::models::Element::Buttons(labels) = &popup.elements[0] {
                    assert_eq!(labels.len(), 3); // Yes, No, Force Yield
                    assert!(labels.contains(&"Yes".to_string()));
                    assert!(labels.contains(&"No".to_string()));
                    assert!(labels.contains(&"Force Yield".to_string()));
                } else {
                    panic!("Expected buttons element");
                }
            }
            Err(e) => panic!("Failed to parse: {}", e),
        }
    }

    #[test]
    fn test_widget_inference() {
        let test_cases = vec![
            ("Range: 0-100", "Range", "Slider"),
            ("Enabled: yes", "Enabled", "Checkbox"),
            ("Theme: Light | Dark", "Theme", "Choice"),
            ("Tags: [A, B, C]", "Tags", "Multiselect"),
            ("Name: @Enter name", "Name", "Textbox"),
        ];
        
        for (input_line, expected_label, expected_type) in test_cases {
            let full_input = format!("Test:\n  {}\n  [OK]", input_line);
            
            match parse_popup_dsl(&full_input) {
                Ok(popup) => {
                    assert!(popup.elements.len() >= 2, "Not enough elements parsed");
                    
                    let element = &popup.elements[0];
                    match element {
                        crate::models::Element::Slider { label, .. } => {
                            assert_eq!(label, expected_label);
                            assert_eq!(expected_type, "Slider");
                        }
                        crate::models::Element::Checkbox { label, .. } => {
                            assert_eq!(label, expected_label);
                            assert_eq!(expected_type, "Checkbox");
                        }
                        crate::models::Element::Choice { label, .. } => {
                            assert_eq!(label, expected_label);
                            assert_eq!(expected_type, "Choice");
                        }
                        crate::models::Element::Multiselect { label, .. } => {
                            assert_eq!(label, expected_label);
                            assert_eq!(expected_type, "Multiselect");
                        }
                        crate::models::Element::Textbox { label, .. } => {
                            assert_eq!(label, expected_label);
                            assert_eq!(expected_type, "Textbox");
                        }
                        _ => panic!("Unexpected element type for {}: {:?}", input_line, element),
                    }
                }
                Err(e) => panic!("Failed to parse {}: {}", input_line, e),
            }
        }
    }

    #[test]
    fn test_button_formats() {
        let test_cases = vec![
            ("[OK | Cancel]", vec!["OK", "Cancel"]),
            ("→ Next", vec!["Next"]),
            ("buttons: Submit or Reset", vec!["Submit", "Reset"]),
            ("with Accept or Decline", vec!["Accept", "Decline"]),
            ("Done or Exit", vec!["Done", "Exit"]),
        ];
        
        for (button_line, expected_buttons) in test_cases {
            let full_input = format!("Test:\n  {}", button_line);
            
            match parse_popup_dsl(&full_input) {
                Ok(popup) => {
                    assert!(!popup.elements.is_empty(), "No elements parsed");
                    
                    let last_element = popup.elements.last().unwrap();
                    if let crate::models::Element::Buttons(labels) = last_element {
                        for expected in expected_buttons {
                            assert!(
                                labels.contains(&expected.to_string()),
                                "Missing button '{}' in {:?}",
                                expected,
                                labels
                            );
                        }
                        assert!(labels.contains(&"Force Yield".to_string()));
                    } else {
                        panic!("Expected buttons element, got: {:?}", last_element);
                    }
                }
                Err(e) => panic!("Failed to parse {}: {}", button_line, e),
            }
        }
    }
}