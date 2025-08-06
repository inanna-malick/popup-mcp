#[cfg(test)]
mod tests {
    use crate::dsl::simple_parser::parse_popup_dsl_with_title;
    use crate::models::Element;

    #[test]
    fn test_parse_without_title() {
        let input = "Debug: yes\n[OK | Cancel]";
        let result = parse_popup_dsl_with_title(input, None);
        
        assert!(result.is_ok());
        let popup = result.unwrap();
        
        // Should use default title "Popup"
        assert_eq!(popup.title, "Popup");
        
        // Should have 2 elements
        assert_eq!(popup.elements.len(), 2);
        
        // First element should be checkbox
        assert!(matches!(&popup.elements[0], Element::Checkbox { label, .. } if label == "Debug"));
        
        // Second element should be buttons
        assert!(matches!(&popup.elements[1], Element::Buttons(_)));
    }

    #[test]
    fn test_parse_with_custom_title() {
        let input = "Volume: 0-100\nTheme: Light | Dark";
        let result = parse_popup_dsl_with_title(input, Some("Settings".to_string()));
        
        assert!(result.is_ok());
        let popup = result.unwrap();
        
        // Should use provided title
        assert_eq!(popup.title, "Settings");
        
        // Should have 2 elements
        assert_eq!(popup.elements.len(), 2);
        
        // First element should be slider
        assert!(matches!(&popup.elements[0], Element::Slider { label, .. } if label == "Volume"));
        
        // Second element should be choice
        assert!(matches!(&popup.elements[1], Element::Choice { label, .. } if label == "Theme"));
    }

    #[test]
    fn test_conditional_on_first_line() {
        // This used to fail when first line was treated as title
        let input = "[if Advanced] { Debug: yes }\n[Save]";
        let result = parse_popup_dsl_with_title(input, Some("Config".to_string()));
        
        assert!(result.is_ok());
        let popup = result.unwrap();
        
        assert_eq!(popup.title, "Config");
        assert_eq!(popup.elements.len(), 2);
        
        // First element should be conditional
        assert!(matches!(&popup.elements[0], Element::Conditional { .. }));
        
        // Second element should be buttons  
        assert!(matches!(&popup.elements[1], Element::Buttons(_)));
    }

    #[test]
    fn test_buttons_on_first_line() {
        // This used to fail when "Yes or No" was parsed as title
        let input = "Yes or No";
        let result = parse_popup_dsl_with_title(input, Some("Confirm".to_string()));
        
        assert!(result.is_ok());
        let popup = result.unwrap();
        
        assert_eq!(popup.title, "Confirm");
        assert_eq!(popup.elements.len(), 1);
        
        // Should have buttons with Yes and No
        if let Element::Buttons(labels) = &popup.elements[0] {
            assert!(labels.contains(&"Yes".to_string()));
            assert!(labels.contains(&"No".to_string()));
            assert!(labels.contains(&"Force Yield".to_string())); // Auto-added
        } else {
            panic!("Expected buttons");
        }
    }

    #[test]
    fn test_empty_input_with_title() {
        let input = "";
        let result = parse_popup_dsl_with_title(input, Some("Empty Popup".to_string()));
        
        assert!(result.is_ok());
        let popup = result.unwrap();
        
        assert_eq!(popup.title, "Empty Popup");
        assert_eq!(popup.elements.len(), 0);
    }

    #[test]
    fn test_complex_popup_with_title() {
        let input = r#"! Warning: This will delete all data
Confirm deletion: yes
[if Confirm deletion] {
    Backup first: no
    Location: @/path/to/backup
}
[Delete | Cancel]"#;
        
        let result = parse_popup_dsl_with_title(input, Some("Dangerous Operation".to_string()));
        
        assert!(result.is_ok());
        let popup = result.unwrap();
        
        assert_eq!(popup.title, "Dangerous Operation");
        
        // Should have warning, checkbox, conditional, and buttons
        assert!(popup.elements.len() >= 3);
        
        // First should be warning message
        if let Element::Text(text) = &popup.elements[0] {
            assert!(text.contains("Warning"));
        } else {
            panic!("Expected warning text");
        }
    }

    #[test]
    fn test_multiline_text_without_title_confusion() {
        let input = "This is a description\nthat spans multiple lines\nSettings: yes";
        let result = parse_popup_dsl_with_title(input, Some("Info".to_string()));
        
        assert!(result.is_ok());
        let popup = result.unwrap();
        
        assert_eq!(popup.title, "Info");
        // All lines should be parsed as elements, not confused with title
        assert!(popup.elements.len() >= 2);
    }

    #[test]
    fn test_backward_compatibility() {
        // Test that parse_popup_dsl (without title param) still works
        use crate::dsl::simple_parser::parse_popup_dsl;
        
        let input = "Option: A | B | C";
        let result = parse_popup_dsl(input);
        
        assert!(result.is_ok());
        let popup = result.unwrap();
        
        // Should use default title
        assert_eq!(popup.title, "Popup");
        
        // Should parse the choice widget
        assert!(matches!(&popup.elements[0], Element::Choice { .. }));
    }
}