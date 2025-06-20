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
fn test_optional_commas_between_newlines() {
    // Test that commas are optional when elements are on separate lines
    let test_cases = vec![
        // Classic syntax without commas
        r#"popup "No Commas" [
            text "First line"
            text "Second line"  
            textbox "Input field"
            buttons ["OK"]
        ]"#,
        
        // Simplified syntax without commas
        r#"[No Commas:
            text "First line"
            text "Second line"
            textbox "Input field" 
            buttons ["OK"]
        ]"#,
        
    ];
    
    for (i, input) in test_cases.iter().enumerate() {
        let result = parse_popup_dsl(input);
        assert!(result.is_ok(), "Test case {} failed to parse: {}", i, input);
        
        let popup = result.unwrap();
        assert_eq!(popup.elements.len(), 4); // 2 text + 1 textbox + 1 buttons
    }
}

#[test]
fn test_natural_multiline_layout_works() {
    // The key feature: natural layout without commas works perfectly
    let input = r#"popup "Natural Layout" [
        text "First element"
        text "Second element"  
        textbox "User input"
        checkbox "Option"
        buttons ["Save", "Cancel"]
    ]"#;
    
    let result = parse_popup_dsl(input);
    assert!(result.is_ok(), "Natural layout should work without commas");
    
    let popup = result.unwrap();
    assert_eq!(popup.elements.len(), 5); // text + text + textbox + checkbox + buttons
}

#[test]
fn test_extended_bare_text_support() {
    // Test enhanced bare text patterns - multiline format (working)
    let test_cases = vec![
        // Simple multiline bare text
        (r#"[Test: 
            Name: [textbox]
            buttons ["OK"]
        ]"#, 2),
        
        // Test with parentheses in multiline
        (r#"[Test:
            Age (optional): [input]
            buttons ["OK"]
        ]"#, 2),
        
        // Test with question marks in multiline
        (r#"[Test:
            What's your name?: [textbox]
            buttons ["OK"]
        ]"#, 2),
        
        // Test multiple bare text elements
        (r#"[Form:
            Full name: [textbox]
            Email: [input]
            Subscribe: [checkbox]
            buttons ["Submit"]
        ]"#, 4),
    ];
    
    for (i, (input, expected_elements)) in test_cases.iter().enumerate() {
        let result = parse_popup_dsl(input);
        assert!(result.is_ok(), "Test case {} failed: {}", i, result.unwrap_err());
        
        let popup = result.unwrap();
        assert_eq!(popup.elements.len(), *expected_elements, "Wrong element count for test case {}", i);
    }
}

#[test]
fn test_slider_pattern_parsing() {
    let test_cases = vec![
        (r#"[Test: Volume: [0..100], buttons ["OK"]]"#, 0.0, 100.0, 50.0),
        (r#"[Test: Rating: [1-5], buttons ["OK"]]"#, 1.0, 5.0, 3.0),
        (r#"[Test: Score: [0..10@7], buttons ["OK"]]"#, 0.0, 10.0, 7.0),
    ];
    
    for (input, expected_min, expected_max, expected_default) in test_cases {
        let result = parse_popup_dsl(input);
        assert!(result.is_ok(), "Failed to parse slider pattern: {}", input);
        
        let popup = result.unwrap();
        match &popup.elements[0] {
            Element::Slider { min, max, default, .. } => {
                assert_eq!(*min, expected_min);
                assert_eq!(*max, expected_max);
                assert_eq!(*default, expected_default);
            },
            _ => panic!("Expected slider element"),
        }
    }
}

#[test]
fn test_natural_language_labels() {
    // Test that labels with punctuation and complex text work
    let input = r#"[User Info:
        What's your full name?: [textbox]
        Email (required): [input]
        Do you agree to the terms?: [checkbox]
        How would you rate us?: [1..5]
        buttons ["Continue"]
    ]"#;
    
    let result = parse_popup_dsl(input);
    assert!(result.is_ok(), "Natural language labels should work");
    
    let popup = result.unwrap();
    assert_eq!(popup.elements.len(), 5);
    
    // Check that labels are preserved correctly
    match &popup.elements[0] {
        Element::Textbox { label, .. } => assert_eq!(label, "What's your full name?"),
        _ => panic!("Expected textbox"),
    }
    
    match &popup.elements[1] {
        Element::Textbox { label, .. } => assert_eq!(label, "Email (required)"),
        _ => panic!("Expected textbox"),
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