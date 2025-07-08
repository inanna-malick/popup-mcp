use super::*;
use proptest::prelude::*;
use crate::models::{Element, PopupDefinition};
    
    // Test semantic aliases for widget types
    #[test]
    fn test_checkbox_aliases() {
        let aliases = vec![
            "checkbox", "check", "tick", "toggle", "switch", 
            "bool", "boolean", "yes/no", "y/n", "enabled"
        ];
        
        for alias in aliases {
            let dsl = format!("Test:\n  {} \"Enabled\"", alias);
            let result = parse_popup_dsl(&dsl);
            assert!(result.is_ok(), "Failed to parse alias: {}", alias);
            
            let popup = result.unwrap();
            assert_eq!(popup.elements.len(), 2); // Widget + Force Yield button
            
            match &popup.elements[0] {
                Element::Checkbox { label, .. } => {
                    assert_eq!(label, "Enabled");
                }
                _ => panic!("Expected checkbox for alias: {}", alias),
            }
        }
    }
    
    #[test]
    fn test_slider_aliases() {
        let aliases = vec![
            "slider", "range", "scale", "numeric", "number",
            "dial", "knob", "level", "gauge", "meter"
        ];
        
        for alias in aliases {
            let dsl = format!("Test:\n  {} \"Volume\" 0..100", alias);
            let result = parse_popup_dsl(&dsl);
            assert!(result.is_ok(), "Failed to parse alias: {}", alias);
            
            let popup = result.unwrap();
            match &popup.elements[0] {
                Element::Slider { label, min, max, .. } => {
                    assert_eq!(label, "Volume");
                    assert_eq!(*min, 0.0);
                    assert_eq!(*max, 100.0);
                }
                _ => panic!("Expected slider for alias: {}", alias),
            }
        }
    }
    
    // Test natural language patterns
    #[test]
    fn test_natural_slider_patterns() {
        let patterns = vec![
            ("Volume from 0 to 100", 0.0, 100.0, None),
            ("Volume between 0 and 100", 0.0, 100.0, None),
            ("Volume ranging 0 to 100", 0.0, 100.0, None),
            ("Volume from 0 to 100 starting at 50", 0.0, 100.0, Some(50.0)),
            ("Volume from 0 to 100 initially 75", 0.0, 100.0, Some(75.0)),
            ("Volume from 0 to 100 (default: 25)", 0.0, 100.0, Some(25.0)),
        ];
        
        for (pattern, expected_min, expected_max, expected_default) in patterns {
            let dsl = format!("Test:\n  {}", pattern);
            let result = parse_popup_dsl(&dsl);
            assert!(result.is_ok(), "Failed to parse pattern: {}", pattern);
            
            let popup = result.unwrap();
            match &popup.elements[0] {
                Element::Slider { label, min, max, default } => {
                    assert_eq!(label, "Volume");
                    assert_eq!(*min, expected_min);
                    assert_eq!(*max, expected_max);
                    assert_eq!(*default, expected_default.unwrap_or((*min + *max) / 2.0));
                }
                _ => panic!("Expected slider for pattern: {}", pattern),
            }
        }
    }
    
    #[test]
    fn test_inferred_widget_types() {
        let tests = vec![
            ("Volume: 0-100", "slider"),
            ("Enabled: yes", "checkbox"),
            ("Theme: Light | Dark", "choice"),
            ("Tags: [Work, Personal, Urgent]", "multiselect"),
            ("Name: @Enter your name", "textbox"),
        ];
        
        for (pattern, expected_type) in tests {
            let dsl = format!("Test:\n  {}", pattern);
            let result = parse_popup_dsl(&dsl);
            assert!(result.is_ok(), "Failed to parse pattern: {}", pattern);
            
            let popup = result.unwrap();
            let element_type = match &popup.elements[0] {
                Element::Slider { .. } => "slider",
                Element::Checkbox { .. } => "checkbox",
                Element::Choice { .. } => "choice",
                Element::Multiselect { .. } => "multiselect",
                Element::Textbox { .. } => "textbox",
                _ => "other",
            };
            
            assert_eq!(element_type, expected_type, "Wrong type for pattern: {}", pattern);
        }
    }
    
    #[test]
    fn test_multiple_popup_formats() {
        let formats = vec![
            // Structured format
            "Settings:\n  Volume: 0-100\n  [Save | Cancel]",
            // Bracket format
            "[Settings: Volume: 0-100, Save or Cancel]",
            // Natural language format
            "confirm \"Save changes?\" with Yes or No",
        ];
        
        for format in formats {
            let result = parse_popup_dsl(format);
            assert!(result.is_ok(), "Failed to parse format: {}", format);
        }
    }
    
    #[test]
    fn test_symbolic_widgets() {
        // Test checkbox patterns
        let checkbox_tests = vec![
            ("✓ Notifications", true),
            ("☐ Updates", false),
        ];
        
        for (pattern, expected_default) in checkbox_tests {
            let dsl = format!("Test:\n  {}", pattern);
            let result = parse_popup_dsl(&dsl);
            assert!(result.is_ok(), "Failed to parse pattern: {}", pattern);
            
            let popup = result.unwrap();
            match &popup.elements[0] {
                Element::Checkbox { default, .. } => {
                    assert_eq!(*default, expected_default);
                }
                _ => panic!("Expected checkbox for pattern: {}", pattern),
            }
        }
        
        // Test slider patterns
        let slider_tests = vec![
            ("★★★☆☆", 3.0),
            ("[•••••     ]", 5.0),
        ];
        
        for (pattern, expected_default) in slider_tests {
            let dsl = format!("Test:\n  {}", pattern);
            let result = parse_popup_dsl(&dsl);
            assert!(result.is_ok(), "Failed to parse pattern: {}", pattern);
            
            let popup = result.unwrap();
            match &popup.elements[0] {
                Element::Slider { default, .. } => {
                    assert_eq!(*default, expected_default);
                }
                _ => panic!("Expected slider for pattern: {}", pattern),
            }
        }
    }
    
    #[test]
    fn test_conditional_syntax_variations() {
        let conditions = vec![
            "when notifications:",
            "if notifications:",
            "unless not notifications:",
            "show when notifications:",
            "visible if notifications:",
            "when notifications =>",
            "if notifications then",
        ];
        
        for condition in conditions {
            let dsl = format!("Test:\n  Enabled: yes\n  {}\n    Sound: on\n  [OK]", condition);
            let result = parse_popup_dsl(&dsl);
            assert!(result.is_ok(), "Failed to parse condition: {}", condition);
        }
    }
    
    #[test]
    fn test_button_syntax_variations() {
        let button_patterns = vec![
            "[OK | Cancel]",
            "→ Continue",
            "buttons: [Save, Cancel]",
            "actions: Save or Cancel",
            "OK or Cancel",
            "---\nSave | Cancel",
        ];
        
        for pattern in button_patterns {
            let dsl = format!("Test:\n  {}", pattern);
            let result = parse_popup_dsl(&dsl);
            assert!(result.is_ok(), "Failed to parse button pattern: {}", pattern);
            
            let popup = result.unwrap();
            let has_buttons = popup.elements.iter().any(|e| matches!(e, Element::Buttons(_)));
            assert!(has_buttons, "No buttons found for pattern: {}", pattern);
        }
    }
    
    #[test]
    fn test_error_recovery() {
        let error_cases = vec![
            // Missing quotes
            ("Test:\n  checkbox enabled", true),
            ("Test:\n  slider volume 0..100", true),
            // Common typos
            ("Test:\n  chekbox enabled", true),
            ("Test:\n  choise theme [Light, Dark]", true),
            // Missing colon after title
            ("Test\n  Volume: 0-100\n  [OK]", true),
            // Missing buttons (should add default)
            ("Test:\n  Volume: 0-100", true),
        ];
        
        for (dsl, should_succeed) in error_cases {
            let result = parse_popup_dsl(dsl);
            assert_eq!(
                result.is_ok(), 
                should_succeed, 
                "Unexpected result for: {}", 
                dsl
            );
        }
    }
    
    // Property-based tests
    proptest! {
        #[test]
        fn prop_all_checkbox_aliases_parse(
            alias in prop::sample::select(vec![
                "checkbox", "check", "tick", "toggle", "switch", 
                "bool", "boolean", "yes/no", "y/n", "enabled"
            ]),
            label in "[a-zA-Z][a-zA-Z0-9 ]{0,20}"
        ) {
            let dsl = format!("Test:\n  {} \"{}\"", alias, label);
            let result = parse_popup_dsl(&dsl);
            prop_assert!(result.is_ok());
        }
        
        #[test]
        fn prop_range_patterns_parse(
            min in 0.0..100.0,
            max in 100.0..1000.0,
            sep in prop::sample::select(vec!["-", "..", "to"]),
            label in "[a-zA-Z][a-zA-Z0-9 ]{0,20}"
        ) {
            let dsl = format!("Test:\n  {}: {} {} {}", label, min, sep, max);
            let result = parse_popup_dsl(&dsl);
            prop_assert!(result.is_ok());
            
            if let Ok(popup) = result {
                if let Some(Element::Slider { min: parsed_min, max: parsed_max, .. }) = popup.elements.first() {
                    prop_assert_eq!(*parsed_min, min as f32);
                    prop_assert_eq!(*parsed_max, max as f32);
                }
            }
        }
        
        #[test]
        fn prop_boolean_values_parse(
            value in prop::sample::select(vec![
                "yes", "no", "on", "off", "true", "false",
                "enabled", "disabled", "active", "inactive"
            ]),
            label in "[a-zA-Z][a-zA-Z0-9 ]{0,20}"
        ) {
            let dsl = format!("Test:\n  {}: {}", label, value);
            let result = parse_popup_dsl(&dsl);
            prop_assert!(result.is_ok());
        }
        
        #[test]
        fn prop_choice_patterns_parse(
            options in prop::collection::vec("[a-zA-Z]+", 2..5),
            label in "[a-zA-Z][a-zA-Z0-9 ]{0,20}"
        ) {
            let choice_str = options.join(" | ");
            let dsl = format!("Test:\n  {}: {}", label, choice_str);
            let result = parse_popup_dsl(&dsl);
            prop_assert!(result.is_ok());
        }
    }
    
    #[test]
    fn test_force_yield_safety() {
        let dsl = "Test:\n  Volume: 0-100\n  [Save]";
        let result = parse_popup_dsl(dsl).unwrap();
        
        // Check that Force Yield was added
        let buttons = result.elements.iter()
            .find_map(|e| match e {
                Element::Buttons(labels) => Some(labels),
                _ => None,
            })
            .unwrap();
        
        assert!(buttons.contains(&"Force Yield".to_string()));
    }
    
    #[test]
    fn test_comment_support() {
        let dsl = r#"Settings:
  # Audio settings
  Volume: 0-100
  // Visual settings
  Theme: Light | Dark
  /* Advanced options */
  Debug: no
  [Save | Cancel]"#;
        
        let result = parse_popup_dsl(dsl);
        assert!(result.is_ok());
        
        let popup = result.unwrap();
        assert_eq!(popup.elements.len(), 5); // 3 widgets + buttons + force yield
    }
    
    #[test]
    fn test_value_reference_syntax() {
        let references = vec![
            "volume > 50",
            "$volume > 50",
            "@volume > 50",
            "{volume} > 50",
            "volume.count > 3",
            "#tasks > 3",
        ];
        
        for reference in references {
            let dsl = format!("Test:\n  Volume: 0-100\n  when {}:\n    Loud: yes\n  [OK]", reference);
            let result = parse_popup_dsl(&dsl);
            assert!(result.is_ok(), "Failed to parse reference: {}", reference);
        }
    }
    
    #[test]
    fn test_serialization_roundtrip() {
        let original = PopupDefinition {
            title: "Test".to_string(),
            elements: vec![
                Element::Slider {
                    label: "Volume".to_string(),
                    min: 0.0,
                    max: 100.0,
                    default: 50.0,
                },
                Element::Checkbox {
                    label: "Mute".to_string(),
                    default: false,
                },
                Element::Choice {
                    label: "Theme".to_string(),
                    options: vec!["Light".to_string(), "Dark".to_string()],
                },
                Element::Buttons(vec!["Save".to_string(), "Cancel".to_string()]),
            ],
        };
        
        let serialized = serialize_popup_dsl(&original);
        let parsed = parse_popup_dsl(&serialized).unwrap();
        
        // Compare elements (ignoring Force Yield)
        assert_eq!(original.title, parsed.title);
        assert_eq!(original.elements.len(), parsed.elements.len() - 1); // -1 for Force Yield
    }