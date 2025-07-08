use super::*;
use crate::models::{Element, Condition, ComparisonOp};

// Tests for the current grammar implementation

#[test]
fn test_natural_language_buttons() {
    // The main fix: natural language buttons with "or" separator
    let test_cases = vec![
        (r#"confirm "Delete file?" with Yes or No"#, vec!["Yes", "No"]),
        (r#"confirm "Save changes?" with Save or Discard"#, vec!["Save", "Discard"]),
        (r#"confirm "Continue?" using OK or Cancel"#, vec!["OK", "Cancel"]),
        (r#"confirm "Choose one" with Option1 or Option2 or Option3"#, vec!["Option1", "Option2", "Option3"]),
    ];
    
    for (dsl, expected) in test_cases {
        let result = parse_popup_dsl(dsl);
        assert!(result.is_ok(), "Failed to parse: {}", dsl);
        
        let popup = result.unwrap();
        let buttons = popup.elements.iter()
            .find_map(|e| match e {
                Element::Buttons(labels) => Some(labels),
                _ => None,
            })
            .unwrap();
        
        for expected_button in expected {
            assert!(buttons.contains(&expected_button.to_string()), 
                "Missing button '{}' in {:?}", expected_button, buttons);
        }
        assert!(buttons.contains(&"Force Yield".to_string()));
    }
}

#[test]
fn test_structured_format() {
    // Based on new_dsl_examples.popup
    let dsl = r#"Delete File?:
  Are you sure you want to delete config.json?
  ! This action cannot be undone
  [Yes | No]"#;
    
    let result = parse_popup_dsl(dsl);
    assert!(result.is_ok(), "Failed to parse structured format");
    
    let popup = result.unwrap();
    assert_eq!(popup.title, "Delete File?");
    
    // Debug: print what elements we got
    println!("Elements found:");
    for (i, elem) in popup.elements.iter().enumerate() {
        match elem {
            Element::Text(t) => println!("  {}: Text('{}')", i, t),
            Element::Buttons(b) => println!("  {}: Buttons({:?})", i, b),
            _ => println!("  {}: Other element", i),
        }
    }
    
    // Should have text elements and buttons
    let has_text = popup.elements.iter().any(|e| matches!(e, Element::Text(_)));
    let has_buttons = popup.elements.iter().any(|e| matches!(e, Element::Buttons(_)));
    
    assert!(has_text, "Should have text elements");
    assert!(has_buttons, "Should have buttons");
}

#[test]
fn test_widget_inference() {
    let dsl = r#"Settings:
  Volume: 0-100 = 75
  Notifications: ✓
  Theme: Light | Dark
  Language: [English, Spanish, French, German]
  [Save]"#;
    
    let result = parse_popup_dsl(dsl);
    assert!(result.is_ok(), "Failed to parse settings");
    
    let popup = result.unwrap();
    
    // Check for specific widget types
    let has_slider = popup.elements.iter().any(|e| matches!(e, Element::Slider { label, .. } if label == "Volume"));
    let has_checkbox = popup.elements.iter().any(|e| matches!(e, Element::Checkbox { label, .. } if label == "Notifications"));
    let has_choice = popup.elements.iter().any(|e| matches!(e, Element::Choice { label, .. } if label == "Theme"));
    let has_multiselect = popup.elements.iter().any(|e| matches!(e, Element::Multiselect { label, .. } if label == "Language"));
    
    assert!(has_slider, "Should have Volume slider");
    assert!(has_checkbox, "Should have Notifications checkbox");
    assert!(has_choice, "Should have Theme choice");
    assert!(has_multiselect, "Should have Language multiselect");
}

#[test]
fn test_textbox_with_placeholder() {
    let dsl = r#"User Profile:
  Name: @Enter your full name
  Email: @user@example.com
  [Submit]"#;
    
    let result = parse_popup_dsl(dsl);
    assert!(result.is_ok(), "Failed to parse textboxes");
    
    let popup = result.unwrap();
    
    // Check textboxes
    for element in &popup.elements {
        if let Element::Textbox { label, placeholder, .. } = element {
            match label.as_str() {
                "Name" => assert_eq!(placeholder.as_deref(), Some("Enter your full name")),
                "Email" => assert_eq!(placeholder.as_deref(), Some("user@example.com")),
                _ => {}
            }
        }
    }
}

#[test]
fn test_conditional_elements() {
    let dsl = r#"Account Setup:
  Account Type: Free | Pro
  
  when Account Type = Pro:
    Payment Method: Credit Card | PayPal
    Auto-renew: yes
  
  [Continue]"#;
    
    let result = parse_popup_dsl(dsl);
    assert!(result.is_ok(), "Failed to parse conditionals");
    
    let popup = result.unwrap();
    
    // Find conditional
    let has_conditional = popup.elements.iter().any(|e| {
        matches!(e, Element::Conditional { condition: Condition::Selected(field, value), .. } 
            if field == "Account Type" && value == "Pro")
    });
    
    assert!(has_conditional, "Should have conditional for Account Type = Pro");
}

#[test]
fn test_button_formats() {
    let test_cases = vec![
        // Bracket format
        (r#"Test:
  [OK | Cancel]"#, vec!["OK", "Cancel"]),
        
        // Arrow format
        (r#"Test:
  → Continue"#, vec!["Continue"]),
        
        // Separator format
        (r#"Test:
  ---
  Save | Exit"#, vec!["Save", "Exit"]),
        
        // Natural format in structured popup
        (r#"Test:
  Done or Cancel"#, vec!["Done", "Cancel"]),
    ];
    
    for (dsl, expected) in test_cases {
        let result = parse_popup_dsl(dsl);
        assert!(result.is_ok(), "Failed to parse: {}", dsl);
        
        let popup = result.unwrap();
        let buttons = popup.elements.iter()
            .find_map(|e| match e {
                Element::Buttons(labels) => Some(labels),
                _ => None,
            })
            .unwrap();
        
        for expected_button in expected {
            assert!(buttons.contains(&expected_button.to_string()), 
                "Missing button '{}' in {:?} for DSL: {}", expected_button, buttons, dsl);
        }
    }
}

#[test]
fn test_sections() {
    let dsl = r#"System Monitor:
  --- Performance ---
  CPU: 0-100
  RAM: 0-16
  
  --- Network ---
  Upload: 0-1000
  Download: 0-1000
  
  [Refresh]"#;
    
    let result = parse_popup_dsl(dsl);
    assert!(result.is_ok(), "Failed to parse sections");
    
    let popup = result.unwrap();
    
    // Should have groups
    let group_count = popup.elements.iter().filter(|e| matches!(e, Element::Group { .. })).count();
    assert_eq!(group_count, 2, "Should have 2 groups");
}

#[test]
fn test_inline_bracket_format() {
    let dsl = "[Quick Settings: Volume: 0-100, Theme: Light | Dark, Save or Cancel]";
    
    let result = parse_popup_dsl(dsl);
    assert!(result.is_ok(), "Failed to parse inline bracket format");
    
    let popup = result.unwrap();
    assert_eq!(popup.title, "Quick Settings");
    
    // Should have slider, choice, and buttons
    let has_slider = popup.elements.iter().any(|e| matches!(e, Element::Slider { .. }));
    let has_choice = popup.elements.iter().any(|e| matches!(e, Element::Choice { .. }));
    let has_buttons = popup.elements.iter().any(|e| matches!(e, Element::Buttons(_)));
    
    assert!(has_slider, "Should have slider");
    assert!(has_choice, "Should have choice");
    assert!(has_buttons, "Should have buttons");
}

#[test]
fn test_emoji_support() {
    let dsl = r#"Mood Tracker:
  Energy: 😴──────😃 = 5
  Mood: 😢 | 😐 | 😊 | 😄
  [📝 Save Entry]"#;
    
    let result = parse_popup_dsl(dsl);
    assert!(result.is_ok(), "Failed to parse emoji content");
    
    let popup = result.unwrap();
    
    // Check that emojis are preserved
    for element in &popup.elements {
        match element {
            Element::Slider { label, .. } if label.contains("Energy") => {
                assert!(label.contains("😴") && label.contains("😃"), "Emojis should be preserved in labels");
            }
            Element::Choice { options, .. } => {
                assert!(options.iter().any(|o| o.contains("😊")), "Emojis should be preserved in options");
            }
            Element::Buttons(labels) => {
                assert!(labels.iter().any(|l| l.contains("📝")), "Emojis should be preserved in buttons");
            }
            _ => {}
        }
    }
}

#[test]
fn test_range_formats() {
    let test_cases = vec![
        ("Volume: 0-100", 0.0, 100.0),
        ("Volume: 0..100", 0.0, 100.0),
        ("Volume: 0 to 100", 0.0, 100.0),
        ("Volume: 0…100", 0.0, 100.0),
    ];
    
    for (widget_def, expected_min, expected_max) in test_cases {
        let dsl = format!("Test:\n  {}\n  [OK]", widget_def);
        let result = parse_popup_dsl(&dsl);
        
        assert!(result.is_ok(), "Failed to parse range format: {}", widget_def);
        
        let popup = result.unwrap();
        let found_slider = popup.elements.iter().any(|e| {
            matches!(e, Element::Slider { min, max, .. } if *min == expected_min && *max == expected_max)
        });
        
        assert!(found_slider, "Failed to parse range: {}", widget_def);
    }
}

#[test]
fn test_checkbox_formats() {
    let test_cases = vec![
        ("Enabled: ✓", true),
        ("Disabled: ✗", false),
        ("Active: yes", true),
        ("Inactive: no", false),
        ("On: true", true),
        ("Off: false", false),
        ("Checked: [x]", true),
        ("Unchecked: [ ]", false),
    ];
    
    for (widget_def, expected_default) in test_cases {
        let dsl = format!("Test:\n  {}\n  [OK]", widget_def);
        let result = parse_popup_dsl(&dsl);
        
        assert!(result.is_ok(), "Failed to parse checkbox format: {}", widget_def);
        
        let popup = result.unwrap();
        let found_checkbox = popup.elements.iter().any(|e| {
            matches!(e, Element::Checkbox { default, .. } if *default == expected_default)
        });
        
        assert!(found_checkbox, "Failed to parse checkbox: {}", widget_def);
    }
}

#[test]
fn test_text_prefixes() {
    let dsl = r#"Messages:
  > Information message
  ! Warning message
  ? Help message
  • Bullet point
  Plain text
  [OK]"#;
    
    let result = parse_popup_dsl(dsl);
    assert!(result.is_ok(), "Failed to parse text prefixes");
    
    let popup = result.unwrap();
    
    // Check that text elements are created with appropriate formatting
    let text_count = popup.elements.iter().filter(|e| matches!(e, Element::Text(_))).count();
    assert!(text_count >= 5, "Should have at least 5 text elements");
}

#[test]
fn test_comparison_conditions() {
    let test_cases = vec![
        ("when volume > 50:", ComparisonOp::Greater, 50),
        ("when count >= 10:", ComparisonOp::GreaterEqual, 10),
        ("when items < 5:", ComparisonOp::Less, 5),
        ("when value <= 100:", ComparisonOp::LessEqual, 100),
        ("when score = 42:", ComparisonOp::Equal, 42),
    ];
    
    for (condition_str, expected_op, expected_val) in test_cases {
        let dsl = format!("Test:\n  Volume: 0-100\n  {}\n    Alert: Active\n  [OK]", condition_str);
        let result = parse_popup_dsl(&dsl);
        
        assert!(result.is_ok(), "Failed to parse condition: {}", condition_str);
        
        let popup = result.unwrap();
        let found_condition = popup.elements.iter().any(|e| {
            if let Element::Conditional { condition, .. } = e {
                if let Condition::Count(_, op, val) = condition {
                    matches!((op, val), (op, v) if 
                        std::mem::discriminant(op) == std::mem::discriminant(&expected_op) && *v == expected_val)
                } else {
                    false
                }
            } else {
                false
            }
        });
        
        assert!(found_condition, "Failed to parse condition: {}", condition_str);
    }
}

#[test]
fn test_force_yield_automatic() {
    // Force Yield should be added automatically to all button lists
    let test_cases = vec![
        r#"confirm "Test" with OK"#,
        r#"Test:
  [Save]"#,
        r#"Test:
  → Continue"#,
    ];
    
    for dsl in test_cases {
        let result = parse_popup_dsl(dsl).unwrap();
        let buttons = result.elements.iter()
            .find_map(|e| match e {
                Element::Buttons(labels) => Some(labels),
                _ => None,
            })
            .unwrap();
        
        assert!(buttons.contains(&"Force Yield".to_string()), 
            "Force Yield should be automatically added for: {}", dsl);
    }
}