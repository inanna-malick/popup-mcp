#[cfg(test)]
mod regression_tests {
    use crate::dsl::simple_parser::parse_popup_dsl;
    use crate::models::{Element, Condition};

    #[test]
    fn test_beta_tester_bug_multiple_text_fields() {
        // Bug: Text fields consuming next lines
        let dsl = "Name: @Enter name\nEmail: @your@email.com\nPhone: @555-1234";
        let popup = parse_popup_dsl(dsl).unwrap();
        
        assert_eq!(popup.elements.len(), 3, "Should have 3 textbox elements");
        
        // Verify each field is a textbox with correct placeholder
        match &popup.elements[0] {
            Element::Textbox { label, placeholder, .. } => {
                assert_eq!(label, "Name");
                assert_eq!(placeholder.as_deref(), Some("Enter name"));
            }
            _ => panic!("First element should be textbox"),
        }
        
        match &popup.elements[1] {
            Element::Textbox { label, placeholder, .. } => {
                assert_eq!(label, "Email");
                assert_eq!(placeholder.as_deref(), Some("your@email.com"));
            }
            _ => panic!("Second element should be textbox"),
        }
        
        match &popup.elements[2] {
            Element::Textbox { label, placeholder, .. } => {
                assert_eq!(label, "Phone");
                assert_eq!(placeholder.as_deref(), Some("555-1234"));
            }
            _ => panic!("Third element should be textbox"),
        }
    }

    #[test]
    fn test_beta_tester_bug_text_field_after_other_elements() {
        // Bug: "Normal" becoming "rmal" due to parser eating characters
        let dsl = r#"Mode: Basic | Advanced
Normal text here
Config: @/etc/app.conf"#;
        
        let popup = parse_popup_dsl(dsl).unwrap();
        
        assert_eq!(popup.elements.len(), 3, "Should have 3 elements");
        
        // Check that "Normal" wasn't consumed
        match &popup.elements[1] {
            Element::Text(text) => {
                assert_eq!(text, "Normal text here", "Text should not be consumed");
            }
            _ => panic!("Second element should be text"),
        }
        
        // Check textbox is correct
        match &popup.elements[2] {
            Element::Textbox { label, placeholder, .. } => {
                assert_eq!(label, "Config");
                assert_eq!(placeholder.as_deref(), Some("/etc/app.conf"));
            }
            _ => panic!("Third element should be textbox"),
        }
    }

    #[test]
    fn test_beta_tester_bug_conditional_label_matching() {
        // Bug: Conditional not working because of exact label matching
        let dsl = r#"Debug Mode: false
[if Debug Mode] {
  Level: 0-10
}"#;
        
        let popup = parse_popup_dsl(dsl).unwrap();
        
        assert_eq!(popup.elements.len(), 2, "Should have checkbox and conditional");
        
        // Check checkbox
        match &popup.elements[0] {
            Element::Checkbox { label, default } => {
                assert_eq!(label, "Debug Mode");
                assert_eq!(*default, false);
            }
            _ => panic!("First element should be checkbox"),
        }
        
        // Check conditional references normalized label
        match &popup.elements[1] {
            Element::Conditional { condition, elements } => {
                match condition {
                    Condition::Checked(label) => {
                        // After normalization, should match
                        assert_eq!(label, "Debug Mode", "Condition should reference normalized label");
                    }
                    _ => panic!("Expected Checked condition"),
                }
                assert_eq!(elements.len(), 1, "Should have one nested element");
            }
            _ => panic!("Second element should be conditional"),
        }
    }

    #[test]
    fn test_first_text_field_deletion_bug() {
        // Bug: First text field disappears in sequences
        let dsl = "Field1: @First\nField2: @Second\nField3: @Third";
        let popup = parse_popup_dsl(dsl).unwrap();
        
        assert_eq!(popup.elements.len(), 3, "Should have all 3 textboxes");
        
        // Verify first field exists
        match &popup.elements[0] {
            Element::Textbox { label, .. } => {
                assert_eq!(label, "Field1", "First field should not disappear");
            }
            _ => panic!("First element should be textbox"),
        }
    }

    #[test]
    fn test_empty_textbox_value() {
        // Edge case: empty @ value
        let dsl = "Name: @\nAge: 0-100";
        let popup = parse_popup_dsl(dsl).unwrap();
        
        assert_eq!(popup.elements.len(), 2, "Should have textbox and slider");
        
        match &popup.elements[0] {
            Element::Textbox { placeholder, .. } => {
                assert_eq!(placeholder, &None, "Empty @ should have no placeholder");
            }
            _ => panic!("First element should be textbox"),
        }
    }

    #[test]
    fn test_text_with_at_symbol_not_textbox() {
        // Edge case: @ in middle of text shouldn't create textbox
        let dsl = "Email me at: user@example.com";
        let popup = parse_popup_dsl(dsl).unwrap();
        
        assert_eq!(popup.elements.len(), 1, "Should have 1 element");
        
        // Should be parsed as labeled item with text value, not textbox
        match &popup.elements[0] {
            Element::Text(_) => {
                // This is expected - not a textbox since @ isn't at start
            }
            Element::Textbox { .. } => {
                panic!("Should not be parsed as textbox when @ is not at start of value");
            }
            _ => {}
        }
    }

    #[test]
    fn test_conditional_with_spaces_in_label() {
        // Test that conditionals work with multi-word labels
        let dsl = r#"Show advanced options: yes
[if Show advanced options] {
  Debug: 0-10
}"#;
        
        let popup = parse_popup_dsl(dsl).unwrap();
        
        match &popup.elements[1] {
            Element::Conditional { condition, .. } => {
                match condition {
                    Condition::Checked(label) => {
                        assert_eq!(label, "Show advanced options");
                    }
                    _ => panic!("Expected Checked condition"),
                }
            }
            _ => panic!("Second element should be conditional"),
        }
    }

    #[test]
    fn test_no_force_yield_in_custom_buttons() {
        // Verify Force Yield is not added to custom buttons
        let dsl = "Confirm?\n[Yes | No]";
        let popup = parse_popup_dsl(dsl).unwrap();
        
        // The parser might parse this as just buttons without separate text
        if popup.elements.len() == 1 {
            // Might be just buttons, which is fine
            match &popup.elements[0] {
                Element::Buttons(buttons) => {
                    assert_eq!(buttons.len(), 2, "Should only have Yes and No");
                    assert!(!buttons.contains(&"Force Yield".to_string()), 
                            "Force Yield should not be added");
                }
                _ => panic!("Expected buttons element, got {:?}", popup.elements[0]),
            }
        } else {
            assert_eq!(popup.elements.len(), 2, "Should have text and buttons");
            match &popup.elements[1] {
                Element::Buttons(buttons) => {
                    assert_eq!(buttons.len(), 2, "Should only have Yes and No");
                    assert!(!buttons.contains(&"Force Yield".to_string()), 
                            "Force Yield should not be added");
                }
                _ => panic!("Second element should be buttons"),
            }
        }
    }
}