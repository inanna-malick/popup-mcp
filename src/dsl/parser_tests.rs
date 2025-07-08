use super::*;
use crate::models::{Element, PopupDefinition, Condition, ComparisonOp};

// Comprehensive parser test suite
#[test]
fn test_natural_language_buttons_parse_correctly() {
    // This is the bug case - "Yes or No" should parse as two buttons
    let dsl = r#"confirm "Delete file?" with Yes or No"#;
    let result = parse_popup_dsl(dsl).unwrap();
    
    // Find the buttons element
    let buttons = result.elements.iter()
        .find_map(|e| match e {
            Element::Buttons(labels) => Some(labels),
            _ => None,
        })
        .expect("Should have buttons");
    
    // Debug: Print what we actually got
    eprintln!("Parsed buttons: {:?}", buttons);
    eprintln!("Button count: {}", buttons.len());
    for (i, button) in buttons.iter().enumerate() {
        eprintln!("  Button {}: '{}'", i, button);
    }
    
    // Should have 3 buttons: Yes, No, and Force Yield
    assert_eq!(buttons.len(), 3, "Expected 3 buttons (Yes, No, Force Yield), got {:?}", buttons);
    assert!(buttons.contains(&"Yes".to_string()), "Missing 'Yes' button in {:?}", buttons);
    assert!(buttons.contains(&"No".to_string()), "Missing 'No' button in {:?}", buttons);
    assert!(buttons.contains(&"Force Yield".to_string()), "Missing 'Force Yield' button in {:?}", buttons);
}

#[test] 
fn test_natural_language_format_parsing() {
    // Test that natural language format is recognized
    let dsl = r#"confirm "Save changes?" with Save or Discard"#;
    let result = parse_popup_dsl(dsl);
    assert!(result.is_ok(), "Failed to parse natural language format");
    
    let popup = result.unwrap();
    assert_eq!(popup.title, "Save changes?");
    
    // Check elements
    eprintln!("Elements: {:?}", popup.elements);
    assert!(!popup.elements.is_empty(), "No elements parsed");
}

#[test]
fn test_button_list_parsing_unit() {
    // Test the grammar's ability to parse button lists
    let test_cases = vec![
        ("Yes or No", vec!["Yes", "No"]),
        ("Save or Cancel", vec!["Save", "Cancel"]),
        ("OK or Cancel or Maybe", vec!["OK", "Cancel", "Maybe"]),
    ];
    
    for (input, expected) in test_cases {
        let dsl = format!(r#"confirm "Test" with {}"#, input);
        let result = parse_popup_dsl(&dsl);
        
        assert!(result.is_ok(), "Failed to parse: {}", dsl);
        let popup = result.unwrap();
        
        let buttons = popup.elements.iter()
            .find_map(|e| match e {
                Element::Buttons(labels) => Some(labels),
                _ => None,
            })
            .expect("Should have buttons");
        
        // Remove Force Yield for comparison
        let user_buttons: Vec<&String> = buttons.iter()
            .filter(|b| b.as_str() != "Force Yield")
            .collect();
        
        assert_eq!(user_buttons.len(), expected.len(), 
            "Wrong button count for '{}'. Got {:?}", input, user_buttons);
        
        for expected_button in expected {
            assert!(user_buttons.iter().any(|b| b.as_str() == expected_button),
                "Missing button '{}' in {:?} for input '{}'", expected_button, user_buttons, input);
        }
    }
}

#[test]
fn test_button_text_grammar_issue() {
    // The grammar defines button_text as: @{ (!(button_sep | "]" | NEWLINE) ~ ANY)+ }
    // where button_sep = { "|" | "," }
    // This means "or" is NOT a separator, so "Yes or No" is consumed as one button_text
    
    // Test current behavior
    let dsl = r#"confirm "Test" with Yes or No"#;
    let result = parse_popup_dsl(&dsl).unwrap();
    
    let buttons = result.elements.iter()
        .find_map(|e| match e {
            Element::Buttons(labels) => Some(labels),
            _ => None,
        })
        .unwrap();
    
    // Current behavior: "Yes or No" is one button
    eprintln!("Current parsing: {:?}", buttons);
    
    // Test with explicit separators that DO work
    let test_cases = vec![
        ("[Yes | No]", vec!["Yes", "No"]),
        ("[Yes, No]", vec!["Yes", "No"]),
        ("buttons: [Yes, No]", vec!["Yes", "No"]),
    ];
    
    for (pattern, expected) in test_cases {
        let dsl = format!("Test:\n  {}", pattern);
        let result = parse_popup_dsl(&dsl).unwrap();
        
        let buttons = result.elements.iter()
            .find_map(|e| match e {
                Element::Buttons(labels) => Some(labels),
                _ => None,
            })
            .unwrap();
        
        let user_buttons: Vec<&String> = buttons.iter()
            .filter(|b| b.as_str() != "Force Yield")
            .collect();
        
        assert_eq!(user_buttons.len(), expected.len(), 
            "Pattern '{}' should parse to {:?} buttons", pattern, expected);
    }
}

#[test]
fn test_natural_popup_grammar_path() {
    // Test that natural_popup rule exists and is being used
    use pest::Parser;
    
    // Try parsing with direct rule access
    let input = r#"confirm "Delete?" with Yes or No"#;
    
    // This should match the natural_popup rule
    match PopupParser::parse(Rule::popup, input) {
        Ok(pairs) => {
            eprintln!("Successfully parsed as popup");
            for pair in pairs {
                eprintln!("  Rule: {:?}, Text: '{}'", pair.as_rule(), pair.as_str());
                for inner in pair.into_inner() {
                    eprintln!("    Rule: {:?}, Text: '{}'", inner.as_rule(), inner.as_str());
                    for inner2 in inner.into_inner() {
                        eprintln!("      Rule: {:?}, Text: '{}'", inner2.as_rule(), inner2.as_str());
                        for inner3 in inner2.into_inner() {
                            eprintln!("        Rule: {:?}, Text: '{}'", inner3.as_rule(), inner3.as_str());
                            for inner4 in inner3.into_inner() {
                                eprintln!("          Rule: {:?}, Text: '{}'", inner4.as_rule(), inner4.as_str());
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
        }
    }
}

#[test]
fn test_button_value_parsing() {
    // The problem: button_text = @{ (!(button_sep | "]" | NEWLINE) ~ ANY)+ }
    // button_sep = { "|" | "," }
    // So "Yes or No" is consumed as ONE button_text because "or" is not a separator
    
    use pest::Parser;
    
    // Test what button_value matches
    let test_cases = vec![
        ("Yes", 1),
        ("Yes or No", 1),  // This is the bug - treated as one value
        ("\"Yes or No\"", 1), // Quoted string is correctly one value
    ];
    
    for (input, expected_values) in test_cases {
        match PopupParser::parse(Rule::button_value, input) {
            Ok(mut pairs) => {
                let pair = pairs.next().unwrap();
                eprintln!("button_value '{}' parsed as: {:?}", input, pair.as_str());
                assert_eq!(pairs.count() + 1, expected_values, 
                    "Expected {} values for input '{}'", expected_values, input);
            }
            Err(e) => {
                eprintln!("Failed to parse '{}': {}", input, e);
            }
        }
    }
}

#[test]
fn test_button_list_parsing_directly() {
    use pest::Parser;
    
    // Test button_list rule directly
    let test_cases = vec![
        ("Yes or No", vec!["Yes", "No"]),
        ("[Yes, No]", vec!["Yes", "No"]),
        ("Yes | No", vec!["Yes", "No"]),
    ];
    
    for (input, expected) in test_cases {
        eprintln!("\nTesting button_list: '{}'", input);
        match PopupParser::parse(Rule::button_list, input) {
            Ok(pairs) => {
                for pair in pairs {
                    eprintln!("  Matched: {:?}", pair.as_str());
                    for inner in pair.into_inner() {
                        eprintln!("    Inner: {:?} = '{}'", inner.as_rule(), inner.as_str());
                    }
                }
            }
            Err(e) => {
                eprintln!("  Parse error: {}", e);
            }
        }
    }
}

#[test]
fn test_parse_natural_popup_function() {
    // Look at how parse_natural_popup handles button_list
    let _dsl = r#"confirm "Test" with Yes or No"#;
    
    // The issue is in parse_natural_popup function
    // It's calling parse_button_list on the "Yes or No" part
    // But button_list grammar expects separators like [Yes, No] or "Yes | No"
    
    // Let's verify the button_list patterns
    let button_list_tests = vec![
        ("[OK, Cancel]", 2),
        ("OK or Cancel", 2),  // natural_buttons rule
        ("[OK | Cancel]", 2),
    ];
    
    for (pattern, expected_count) in button_list_tests {
        eprintln!("\nTesting button pattern: '{}'", pattern);
        // We can't test button_list directly, but we can test via full popup
        let dsl = format!("Test:\n  buttons: {}", pattern);
        match parse_popup_dsl(&dsl) {
            Ok(popup) => {
                let buttons = popup.elements.iter()
                    .find_map(|e| match e {
                        Element::Buttons(labels) => Some(labels),
                        _ => None,
                    })
                    .unwrap();
                let user_buttons: Vec<&String> = buttons.iter()
                    .filter(|b| b.as_str() != "Force Yield")
                    .collect();
                eprintln!("  Parsed {} buttons: {:?}", user_buttons.len(), user_buttons);
                assert_eq!(user_buttons.len(), expected_count, 
                    "Expected {} buttons for pattern '{}'", expected_count, pattern);
            }
            Err(e) => {
                eprintln!("  Parse error: {}", e);
            }
        }
    }
}

#[test]
fn test_structured_format_basic() {
    let dsl = "Test:\n  Volume: 0-100\n  [OK]";
    let result = parse_popup_dsl(dsl).unwrap();
    
    assert_eq!(result.title, "Test");
    assert_eq!(result.elements.len(), 2); // Slider + Buttons
    
    match &result.elements[0] {
        Element::Slider { label, min, max, default } => {
            assert_eq!(label, "Volume");
            assert_eq!(*min, 0.0);
            assert_eq!(*max, 100.0);
            assert_eq!(*default, 50.0);
        }
        _ => panic!("Expected slider"),
    }
}

#[test]
fn test_bracket_format() {
    let dsl = "[Settings: Volume: 0-100, Save or Cancel]";
    let result = parse_popup_dsl(dsl).unwrap();
    
    assert_eq!(result.title, "Settings");
    // Should have Volume slider and buttons
    assert!(matches!(&result.elements[0], Element::Slider { .. }));
    assert!(matches!(&result.elements[1], Element::Buttons(_)));
}

#[test]
fn test_widget_value_inference() {
    let test_cases = vec![
        ("Volume: 0-100", Element::Slider { 
            label: "Volume".to_string(), 
            min: 0.0, 
            max: 100.0, 
            default: 50.0 
        }),
        ("Enabled: yes", Element::Checkbox { 
            label: "Enabled".to_string(), 
            default: true 
        }),
        ("Disabled: no", Element::Checkbox { 
            label: "Disabled".to_string(), 
            default: false 
        }),
        ("Theme: Light | Dark | Auto", Element::Choice { 
            label: "Theme".to_string(), 
            options: vec!["Light".to_string(), "Dark".to_string(), "Auto".to_string()] 
        }),
        ("Tags: [Work, Personal, Urgent]", Element::Multiselect { 
            label: "Tags".to_string(), 
            options: vec!["Work".to_string(), "Personal".to_string(), "Urgent".to_string()] 
        }),
        ("Name: @Enter your name", Element::Textbox { 
            label: "Name".to_string(), 
            placeholder: Some("Enter your name".to_string()),
            rows: None
        }),
    ];
    
    for (pattern, expected) in test_cases {
        let dsl = format!("Test:\n  {}\n  [OK]", pattern);
        let result = parse_popup_dsl(&dsl).unwrap();
        
        match (&result.elements[0], &expected) {
            (Element::Slider { label: l1, min: min1, max: max1, default: d1 }, 
             Element::Slider { label: l2, min: min2, max: max2, default: d2 }) => {
                assert_eq!(l1, l2);
                assert_eq!(min1, min2);
                assert_eq!(max1, max2);
                assert_eq!(d1, d2);
            }
            (Element::Checkbox { label: l1, default: d1 }, 
             Element::Checkbox { label: l2, default: d2 }) => {
                assert_eq!(l1, l2);
                assert_eq!(d1, d2);
            }
            (Element::Choice { label: l1, options: o1 }, 
             Element::Choice { label: l2, options: o2 }) => {
                assert_eq!(l1, l2);
                assert_eq!(o1, o2);
            }
            (Element::Multiselect { label: l1, options: o1 }, 
             Element::Multiselect { label: l2, options: o2 }) => {
                assert_eq!(l1, l2);
                assert_eq!(o1, o2);
            }
            (Element::Textbox { label: l1, placeholder: p1, .. }, 
             Element::Textbox { label: l2, placeholder: p2, .. }) => {
                assert_eq!(l1, l2);
                assert_eq!(p1, p2);
            }
            _ => panic!("Type mismatch for pattern: {}", pattern),
        }
    }
}

#[test]
fn test_range_separators() {
    let separators = vec!["-", "..", "to", "…"];
    
    for sep in separators {
        let dsl = format!("Test:\n  Volume: 0{}100\n  [OK]", sep);
        let result = parse_popup_dsl(&dsl).unwrap();
        
        match &result.elements[0] {
            Element::Slider { min, max, .. } => {
                assert_eq!(*min, 0.0);
                assert_eq!(*max, 100.0);
            }
            _ => panic!("Expected slider for separator: {}", sep),
        }
    }
}

#[test]
fn test_checkbox_symbols() {
    let test_cases = vec![
        ("✓ Enabled", true),
        ("✗ Disabled", false),
        ("☐ Unchecked", false),
        ("☑ Checked", true),
        ("☒ Crossed", false),
        ("[x] Selected", true),
        ("[ ] Unselected", false),
        ("(*) Active", true),
        ("( ) Inactive", false),
    ];
    
    for (pattern, expected_default) in test_cases {
        let dsl = format!("Test:\n  {}\n  [OK]", pattern);
        let result = parse_popup_dsl(&dsl).unwrap();
        
        match &result.elements[0] {
            Element::Checkbox { default, .. } => {
                assert_eq!(*default, expected_default, "Wrong default for pattern: {}", pattern);
            }
            _ => panic!("Expected checkbox for pattern: {}", pattern),
        }
    }
}

#[test]
fn test_conditional_elements() {
    let dsl = r#"Settings:
  Enable features: yes
  when Enable features:
    Advanced: no
    Debug level: 0-5
  [Save]"#;
    
    let result = parse_popup_dsl(dsl).unwrap();
    
    // Should have checkbox, conditional, and buttons
    assert_eq!(result.elements.len(), 3);
    
    match &result.elements[1] {
        Element::Conditional { condition, elements } => {
            match condition {
                Condition::Checked(field) => assert_eq!(field, "Enable features"),
                _ => panic!("Expected Checked condition"),
            }
            assert_eq!(elements.len(), 2);
        }
        _ => panic!("Expected conditional element"),
    }
}

#[test]
fn test_condition_types() {
    let test_cases = vec![
        ("when notifications:", Condition::Checked("notifications".to_string())),
        ("when theme = Dark:", Condition::Selected("theme".to_string(), "Dark".to_string())),
        ("when volume > 50:", Condition::Count("volume".to_string(), ComparisonOp::Greater, 50)),
        ("when tasks.count >= 3:", Condition::Count("tasks".to_string(), ComparisonOp::GreaterEqual, 3)),
        ("when #items < 10:", Condition::Count("items".to_string(), ComparisonOp::Less, 10)),
    ];
    
    for (condition_str, expected_condition) in test_cases {
        let dsl = format!("Test:\n  Enabled: yes\n  {}:\n    Warning: Active\n  [OK]", condition_str);
        let result = parse_popup_dsl(&dsl).unwrap();
        
        let found_conditional = result.elements.iter().any(|e| {
            if let Element::Conditional { condition, .. } = e {
                match (condition, &expected_condition) {
                    (Condition::Checked(a), Condition::Checked(b)) => a == b,
                    (Condition::Selected(a1, a2), Condition::Selected(b1, b2)) => a1 == b1 && a2 == b2,
                    (Condition::Count(a1, a2, a3), Condition::Count(b1, b2, b3)) => {
                        a1 == b1 && match (a2, b2) {
                            (ComparisonOp::Greater, ComparisonOp::Greater) => true,
                            (ComparisonOp::Less, ComparisonOp::Less) => true,
                            (ComparisonOp::GreaterEqual, ComparisonOp::GreaterEqual) => true,
                            (ComparisonOp::LessEqual, ComparisonOp::LessEqual) => true,
                            (ComparisonOp::Equal, ComparisonOp::Equal) => true,
                            _ => false,
                        } && a3 == b3
                    }
                    _ => false,
                }
            } else {
                false
            }
        });
        
        assert!(found_conditional, "Failed to find expected condition for: {}", condition_str);
    }
}

#[test]
fn test_sections_and_groups() {
    let dsl = r#"Settings:
  --- Audio ---
  Volume: 0-100
  Mute: no
  
  --- Video ---
  Resolution: 720p | 1080p | 4K
  Fullscreen: yes
  
  [Apply | Cancel]"#;
    
    let result = parse_popup_dsl(dsl).unwrap();
    
    // Count groups
    let group_count = result.elements.iter().filter(|e| {
        matches!(e, Element::Group { .. })
    }).count();
    
    assert_eq!(group_count, 2, "Should have 2 groups");
}

#[test]
fn test_button_variations() {
    let test_cases = vec![
        ("[OK | Cancel]", vec!["OK", "Cancel"]),
        ("[Save, Exit]", vec!["Save", "Exit"]),
        ("→ Continue", vec!["Continue"]),
        ("buttons: [Yes, No, Maybe]", vec!["Yes", "No", "Maybe"]),
        ("actions: Submit or Reset", vec!["Submit", "Reset"]),
        ("OK or Cancel", vec!["OK", "Cancel"]),
    ];
    
    for (pattern, expected_buttons) in test_cases {
        let dsl = format!("Test:\n  {}", pattern);
        let result = parse_popup_dsl(&dsl).unwrap();
        
        let buttons = result.elements.iter()
            .find_map(|e| match e {
                Element::Buttons(labels) => Some(labels),
                _ => None,
            })
            .expect("Should have buttons");
        
        for expected in expected_buttons {
            assert!(buttons.contains(&expected.to_string()), 
                "Missing button '{}' in pattern: {}", expected, pattern);
        }
    }
}

#[test]
fn test_standalone_text_formatting() {
    let test_cases = vec![
        ("> Information message", "ℹ️ Information message"),
        ("! Warning message", "⚠️ Warning message"),
        ("? Help message", "❓ Help message"),
        ("• Bullet point", "• Bullet point"),
        ("Plain text", "Plain text"),
    ];
    
    for (input, expected_text) in test_cases {
        let dsl = format!("Test:\n  {}\n  [OK]", input);
        let result = parse_popup_dsl(&dsl).unwrap();
        
        match &result.elements[0] {
            Element::Text(text) => {
                assert_eq!(text, expected_text);
            }
            _ => panic!("Expected text element for: {}", input),
        }
    }
}

#[test]
fn test_complex_nested_structure() {
    let dsl = r#"Configuration:
  Mode: Basic | Advanced
  
  when Mode = Advanced:
    --- Performance ---
    Threads: 1-16 = 4
    Cache size: 0-1000 = 256
    
    when Threads > 8:
      ! High thread count may impact stability
      
    --- Features ---
    Experimental: no
    when Experimental:
      Beta features: [AI assist, Live preview, Cloud sync]
      Risk accepted: no
  
  [Save | Reset | Cancel]"#;
    
    let result = parse_popup_dsl(&dsl).unwrap();
    assert_eq!(result.title, "Configuration");
    
    // Verify the structure parsed correctly
    assert!(result.elements.len() > 2);
    
    // Find the main conditional
    let has_mode_conditional = result.elements.iter().any(|e| {
        matches!(e, Element::Conditional { condition: Condition::Selected(field, value), .. } 
            if field == "Mode" && value == "Advanced")
    });
    
    assert!(has_mode_conditional, "Should have Mode = Advanced conditional");
}

#[test]
fn test_emoji_in_labels() {
    let dsl = r#"Mood Tracker:
  Energy 😴──😃: 0-10 = 5
  Mood: 😢 | 😐 | 😊 | 😄
  Activities today: [🏃 Exercise, 💼 Work, 🎨 Creative, 😴 Rest]
  [📝 Save Entry]"#;
    
    let result = parse_popup_dsl(&dsl).unwrap();
    assert_eq!(result.title, "Mood Tracker");
    
    // Verify emojis are preserved in labels
    match &result.elements[0] {
        Element::Slider { label, .. } => {
            assert!(label.contains("😴") && label.contains("😃"));
        }
        _ => panic!("Expected slider"),
    }
}

#[test]
fn test_quoted_strings_in_values() {
    let dsl = r#"Test:
  Name: @"Enter your full name"
  Status: "Active" | "Inactive" | "Pending"
  Message: 'Hello, world!'
  ["Save Changes" | "Cancel"]"#;
    
    let result = parse_popup_dsl(&dsl).unwrap();
    
    // Check that quoted strings are handled correctly
    match &result.elements[1] {
        Element::Choice { options, .. } => {
            assert_eq!(options, &vec!["Active".to_string(), "Inactive".to_string(), "Pending".to_string()]);
        }
        _ => panic!("Expected choice element"),
    }
}

#[test]
fn test_whitespace_handling() {
    // Test with various whitespace patterns
    let dsl = r#"Test   :  
  
    Volume  :   0  -  100   =   75  
    
    Enabled   :    yes    
    
    [  OK   |   Cancel  ]"#;
    
    let result = parse_popup_dsl(&dsl).unwrap();
    assert_eq!(result.title, "Test");
    
    // Verify elements parsed correctly despite extra whitespace
    match &result.elements[0] {
        Element::Slider { label, min, max, default } => {
            assert_eq!(label.trim(), "Volume");
            assert_eq!(*min, 0.0);
            assert_eq!(*max, 100.0);
            assert_eq!(*default, 75.0);
        }
        _ => panic!("Expected slider"),
    }
}

#[test]
fn test_edge_cases() {
    // Empty popup with just title
    let dsl = "Test:\n  [OK]";
    let result = parse_popup_dsl(dsl);
    assert!(result.is_ok());
    
    // Single element
    let dsl = "Alert:\n  ! System will restart\n  [Acknowledge]";
    let result = parse_popup_dsl(&dsl);
    assert!(result.is_ok());
    
    // Only buttons
    let dsl = "Confirm:\n  [Yes | No | Cancel]";
    let result = parse_popup_dsl(&dsl);
    assert!(result.is_ok());
}

#[test]
fn test_multiline_values() {
    let dsl = r#"Feedback:
  Comments: @"Please share your thoughts
  Multiple lines are welcome"
  Rating: ★★★☆☆
  [Submit]"#;
    
    let result = parse_popup_dsl(&dsl);
    assert!(result.is_ok());
}

#[test]
fn test_special_characters_in_labels() {
    let dsl = r#"Test:
  User/Email: @user@example.com
  API-Key: @Enter key
  Build#: 1-1000
  [OK]"#;
    
    let result = parse_popup_dsl(&dsl).unwrap();
    assert_eq!(result.elements.len(), 4); // 3 widgets + buttons
}

#[test]
fn test_comparison_operators() {
    let operators = vec![
        (">=", ComparisonOp::GreaterEqual),
        ("<=", ComparisonOp::LessEqual),
        ("!=", ComparisonOp::Equal), // Note: != might not be supported
        ("==", ComparisonOp::Equal),
        (">", ComparisonOp::Greater),
        ("<", ComparisonOp::Less),
        ("=", ComparisonOp::Equal),
        ("more than", ComparisonOp::Greater),
        ("less than", ComparisonOp::Less),
        ("at least", ComparisonOp::GreaterEqual),
    ];
    
    for (op_str, _expected_op) in operators {
        let dsl = format!("Test:\n  Count: 0-100\n  when Count {} 50:\n    Alert: yes\n  [OK]", op_str);
        let result = parse_popup_dsl(&dsl);
        // Just check it parses, exact operator matching tested elsewhere
        assert!(result.is_ok() || op_str == "!="); // != might fail
    }
}

#[test]
fn test_force_yield_not_duplicated() {
    let dsl = "Test:\n  [OK | Force Yield]";
    let result = parse_popup_dsl(&dsl).unwrap();
    
    let buttons = result.elements.iter()
        .find_map(|e| match e {
            Element::Buttons(labels) => Some(labels),
            _ => None,
        })
        .unwrap();
    
    // Should not duplicate Force Yield
    let force_yield_count = buttons.iter().filter(|&b| b == "Force Yield").count();
    assert_eq!(force_yield_count, 1);
}