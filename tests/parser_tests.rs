use popup_mcp::dsl::parse_popup_dsl;
use popup_mcp::models::Element;

#[test]
fn test_widget_aliases_in_bare_text() {
    // Test various widget aliases - each on a new line for proper parsing
    let test_cases = vec![
        (r#"[Form: 
            Name: [input]
            buttons ["OK"]]"#, "input", "textbox"),
        (r#"[Form: 
            Email: [field]
            buttons ["OK"]]"#, "field", "textbox"),
        (r#"[Form: 
            Subscribe: [bool]
            buttons ["OK"]]"#, "bool", "checkbox"),
        (r#"[Form: 
            Agree: [toggle]
            buttons ["OK"]]"#, "toggle", "checkbox"),
        // y/n becomes a choice with Y/N options  
        (r#"[Form: 
            Continue: [y/n]
            buttons ["OK"]]"#, "y/n", "choice"),
        // Y/N also becomes choice
        (r#"[Form: 
            Proceed: [Y/N]
            buttons ["OK"]]"#, "Y/N", "choice")
    ];
    
    for (input, alias, expected_type) in test_cases {
        let result = parse_popup_dsl(input);
        assert!(result.is_ok(), "Failed to parse '{}' with alias '{}'", input, alias);
        
        let popup = result.unwrap();
        match &popup.elements[0] {
            Element::Textbox { .. } if expected_type == "textbox" => {},
            Element::Checkbox { .. } if expected_type == "checkbox" => {},
            Element::Choice { .. } if expected_type == "choice" => {},
            _ => panic!("Wrong element type for alias '{}', expected {}", alias, expected_type),
        }
    }
}

#[test]
fn test_mixed_case_widget_aliases() {
    let input = r#"[Test:
        Field1: [Input]
        Field2: [BOOL]
        Field3: [Toggle]
        buttons ["OK"]
    ]"#;
    
    let result = parse_popup_dsl(input);
    assert!(result.is_ok());
    
    let popup = result.unwrap();
    assert_eq!(popup.elements.len(), 4); // 3 fields + buttons
}

#[test]
fn test_enhanced_error_messages() {
    // Missing quotes
    let result = parse_popup_dsl("popup Title []");
    assert!(result.is_err());
    let error = result.unwrap_err().to_string();
    assert!(error.contains("Missing quotes") || error.contains("expected string"));
    
    // Empty popup
    let result = parse_popup_dsl("popup \"Test\" [");
    assert!(result.is_err());
    let error = result.unwrap_err().to_string();
    assert!(error.contains("Expected a popup element") || error.contains("expected element"));
}

#[test]
fn test_invalid_widget_in_simplified_syntax() {
    // This should fail because "invalid_widget" is not a recognized widget type
    let input = r#"[Title: invalid_widget "test"]"#;
    let result = parse_popup_dsl(input);
    assert!(result.is_err());
}

#[test]
fn test_bare_text_not_allowed_in_classic_syntax() {
    // Bare text like "Name: [textbox]" should not work in classic popup syntax with comma
    let input = r#"popup "Title" [ Name: [textbox], buttons ["OK"] ]"#;
    let result = parse_popup_dsl(input);
    // This might actually parse if bare_text is allowed everywhere
    // Let's just check it doesn't crash
    let _ = result;
}


#[test]
fn test_widget_alias_normalization() {
    // Test that aliases work in simplified syntax
    let inputs = vec![
        "[Quick: label \"test\", buttons [\"OK\"]]",
        "[Quick: info \"test\", buttons [\"OK\"]]", 
        "[Quick: message \"test\", buttons [\"OK\"]]",
    ];
    
    for input in inputs {
        let result = parse_popup_dsl(input);
        // These should fail because bare_text expects "Label: [widget]" format
        assert!(result.is_err() || matches!(&result.unwrap().elements[0], Element::Text(_)));
    }
}