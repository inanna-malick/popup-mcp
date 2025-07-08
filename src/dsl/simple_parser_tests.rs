#[cfg(test)]
mod tests {
    use crate::dsl::simple_parser::parse_popup_dsl;
    use crate::models::Element;

    #[test]
    fn test_simple_confirmation() {
        let input = r#"confirm Delete file?
Yes or No"#;
        
        match parse_popup_dsl(input) {
            Ok(popup) => {
                assert_eq!(popup.title, "Delete file?");
                assert_eq!(popup.elements.len(), 1);
                
                if let Element::Buttons(labels) = &popup.elements[0] {
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
    fn test_widget_recognition() {
        let input = r#"Settings
Volume: 0-100
Theme: Light | Dark
Notifications: yes
Auto-save: enabled
Tags: [Work, Personal, Important]
Notes: @Add your notes here
Status: Active
Modified: 3 files
[Save | Cancel]"#;
        
        match parse_popup_dsl(input) {
            Ok(popup) => {
                assert_eq!(popup.title, "Settings");
                assert_eq!(popup.elements.len(), 9);
                
                // Check widget types
                match &popup.elements[0] {
                    Element::Slider { label, min, max, default } => {
                        assert_eq!(label, "Volume");
                        assert_eq!(*min, 0.0);
                        assert_eq!(*max, 100.0);
                        assert_eq!(*default, 50.0);
                    }
                    _ => panic!("Expected slider for Volume"),
                }
                
                match &popup.elements[1] {
                    Element::Choice { label, options } => {
                        assert_eq!(label, "Theme");
                        assert_eq!(options, &["Light", "Dark"]);
                    }
                    _ => panic!("Expected choice for Theme"),
                }
                
                match &popup.elements[2] {
                    Element::Checkbox { label, default } => {
                        assert_eq!(label, "Notifications");
                        assert_eq!(*default, true);
                    }
                    _ => panic!("Expected checkbox for Notifications"),
                }
                
                match &popup.elements[3] {
                    Element::Checkbox { label, default } => {
                        assert_eq!(label, "Auto-save");
                        assert_eq!(*default, true);
                    }
                    _ => panic!("Expected checkbox for Auto-save"),
                }
                
                match &popup.elements[4] {
                    Element::Multiselect { label, options } => {
                        assert_eq!(label, "Tags");
                        assert_eq!(options, &["Work", "Personal", "Important"]);
                    }
                    _ => panic!("Expected multiselect for Tags"),
                }
                
                match &popup.elements[5] {
                    Element::Textbox { label, placeholder, .. } => {
                        assert_eq!(label, "Notes");
                        assert_eq!(placeholder, &Some("Add your notes here".to_string()));
                    }
                    _ => panic!("Expected textbox for Notes"),
                }
                
                // These should be text, not widgets
                match &popup.elements[6] {
                    Element::Text(text) => {
                        assert_eq!(text, "Status: Active");
                    }
                    _ => panic!("Expected text for Status"),
                }
                
                match &popup.elements[7] {
                    Element::Text(text) => {
                        assert_eq!(text, "Modified: 3 files");
                    }
                    _ => panic!("Expected text for Modified"),
                }
            }
            Err(e) => panic!("Failed to parse: {}", e),
        }
    }

    #[test]
    fn test_button_formats() {
        let test_cases = vec![
            ("[OK | Cancel]", vec!["OK", "Cancel"]),
            ("→ Next", vec!["Next"]),
            ("Save or Discard", vec!["Save", "Discard"]),
            ("Yes or No or Maybe", vec!["Yes", "No", "Maybe"]),
        ];
        
        for (button_line, expected_buttons) in test_cases {
            let input = format!("Test\n{}", button_line);
            
            match parse_popup_dsl(&input) {
                Ok(popup) => {
                    assert_eq!(popup.title, "Test");
                    assert_eq!(popup.elements.len(), 1);
                    
                    if let Element::Buttons(labels) = &popup.elements[0] {
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
                        panic!("Expected buttons element for: {}", button_line);
                    }
                }
                Err(e) => panic!("Failed to parse {}: {}", button_line, e),
            }
        }
    }

    #[test]
    fn test_messages() {
        let input = r#"System Update
! Critical security update
> Download size: 145MB
? Need help with installation?
• Restart required after update
This is plain text
[Install | Later]"#;
        
        match parse_popup_dsl(input) {
            Ok(popup) => {
                assert_eq!(popup.title, "System Update");
                assert_eq!(popup.elements.len(), 6);
                
                // Check message formatting
                match &popup.elements[0] {
                    Element::Text(text) => assert!(text.starts_with("⚠️")),
                    _ => panic!("Expected text for warning"),
                }
                
                match &popup.elements[1] {
                    Element::Text(text) => assert!(text.starts_with("ℹ️")),
                    _ => panic!("Expected text for info"),
                }
                
                match &popup.elements[2] {
                    Element::Text(text) => assert!(text.starts_with("❓")),
                    _ => panic!("Expected text for question"),
                }
                
                match &popup.elements[3] {
                    Element::Text(text) => assert!(text.starts_with("•")),
                    _ => panic!("Expected text for bullet"),
                }
                
                match &popup.elements[4] {
                    Element::Text(text) => assert_eq!(text, "This is plain text"),
                    _ => panic!("Expected plain text"),
                }
            }
            Err(e) => panic!("Failed to parse: {}", e),
        }
    }

    #[test]
    fn test_range_formats() {
        let test_cases = vec![
            ("0-100", 0.0, 100.0, 50.0),
            ("0..100", 0.0, 100.0, 50.0),
            ("0 to 100", 0.0, 100.0, 50.0),
            ("0-100 = 75", 0.0, 100.0, 75.0),
            ("10..20=15", 10.0, 20.0, 15.0),
            ("5 to 10 = 7", 5.0, 10.0, 7.0),
        ];
        
        for (range_str, expected_min, expected_max, expected_default) in test_cases {
            let input = format!("Test\nValue: {}\n[OK]", range_str);
            
            match parse_popup_dsl(&input) {
                Ok(popup) => {
                    match &popup.elements[0] {
                        Element::Slider { label, min, max, default } => {
                            assert_eq!(label, "Value");
                            assert_eq!(*min, expected_min);
                            assert_eq!(*max, expected_max);
                            assert_eq!(*default, expected_default);
                        }
                        _ => panic!("Expected slider for range: {}", range_str),
                    }
                }
                Err(e) => panic!("Failed to parse range {}: {}", range_str, e),
            }
        }
    }

    #[test]
    fn test_boolean_formats() {
        let true_values = vec!["yes", "true", "on", "enabled", "✓", "[x]", "(*)"];
        let false_values = vec!["no", "false", "off", "disabled", "☐", "[ ]", "( )"];
        
        for value in true_values {
            let input = format!("Test\nOption: {}\n[OK]", value);
            match parse_popup_dsl(&input) {
                Ok(popup) => {
                    match &popup.elements[0] {
                        Element::Checkbox { label, default } => {
                            assert_eq!(label, "Option");
                            assert_eq!(*default, true, "Expected {} to be true", value);
                        }
                        _ => panic!("Expected checkbox for: {}", value),
                    }
                }
                Err(e) => panic!("Failed to parse {}: {}", value, e),
            }
        }
        
        for value in false_values {
            let input = format!("Test\nOption: {}\n[OK]", value);
            match parse_popup_dsl(&input) {
                Ok(popup) => {
                    match &popup.elements[0] {
                        Element::Checkbox { label, default } => {
                            assert_eq!(label, "Option");
                            assert_eq!(*default, false, "Expected {} to be false", value);
                        }
                        _ => panic!("Expected checkbox for: {}", value),
                    }
                }
                Err(e) => panic!("Failed to parse {}: {}", value, e),
            }
        }
    }

    #[test]
    fn test_text_vs_widget_distinction() {
        let input = r#"Report
Status: Complete
Progress: 100%
Files: document.txt, image.png
Count: 42
Range: wide
Level: high
[Close]"#;
        
        match parse_popup_dsl(input) {
            Ok(popup) => {
                // All of these should be text, not widgets
                for i in 0..6 {
                    match &popup.elements[i] {
                        Element::Text(_) => {},
                        _ => panic!("Expected text element at index {}, got {:?}", i, popup.elements[i]),
                    }
                }
            }
            Err(e) => panic!("Failed to parse: {}", e),
        }
    }
}