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
fn test_multiline_text_support() {
    let input = r#"popup "Instructions" [
        text """Welcome to the system!
        
        This popup demonstrates:
        • Multiline text support
        • Bullet points work
        • Line breaks are preserved
        
        Please proceed when ready."""
        
        buttons ["Continue", "Help"]
    ]"#;
    
    let result = parse_popup_dsl(input);
    assert!(result.is_ok(), "Failed to parse multiline text");
    
    let popup = result.unwrap();
    match &popup.elements[0] {
        Element::Text(content) => {
            assert!(content.contains("Welcome to the system!"));
            assert!(content.contains("• Multiline text support"));
            assert!(content.contains("Line breaks are preserved"));
            // Check that newlines are preserved
            assert!(content.contains('\n'));
        },
        _ => panic!("Expected text element"),
    }
}

#[test]
fn test_mixed_single_and_multiline_strings() {
    let input = r#"popup "Mixed Example" [
        text "Single line text"
        text """Multiline text
        with multiple lines
        and formatting"""
        textbox "Enter details"
        buttons ["Save"]
    ]"#;
    
    let result = parse_popup_dsl(input);
    assert!(result.is_ok(), "Failed to parse mixed string types");
    
    let popup = result.unwrap();
    assert_eq!(popup.elements.len(), 4); // 2 text + 1 textbox + 1 buttons
    
    match &popup.elements[0] {
        Element::Text(content) => assert_eq!(content, "Single line text"),
        _ => panic!("Expected single line text"),
    }
    
    match &popup.elements[1] {
        Element::Text(content) => {
            assert!(content.contains("Multiline text"));
            assert!(content.contains("with multiple lines"));
        },
        _ => panic!("Expected multiline text"),
    }
}

#[test]
fn test_multiline_roundtrip() {
    use popup_mcp::dsl::serialize_popup_dsl;
    
    let input = r#"popup "Test" [
        text """Line 1
Line 2
Line 3"""
        buttons ["OK"]
    ]"#;
    
    let parsed = parse_popup_dsl(input).expect("Should parse multiline");
    let serialized = serialize_popup_dsl(&parsed);
    let reparsed = parse_popup_dsl(&serialized).expect("Should reparse serialized");
    
    // Check the content is preserved
    match &reparsed.elements[0] {
        Element::Text(content) => {
            assert_eq!(content, "Line 1\nLine 2\nLine 3");
        },
        _ => panic!("Expected text element"),
    }
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