use super::*;
use crate::models::{Element, Condition, ComparisonOp};

#[test]
fn test_parse_natural_language_ast() {
    let dsl = r#"confirm "Delete important.txt?" with Yes or No"#;
    let result = parse_popup_dsl(dsl).unwrap();
    
    // Verify AST structure
    assert_eq!(result.title, "Delete important.txt?");
    assert_eq!(result.elements.len(), 1, "Should have 1 element (buttons)");
    
    match &result.elements[0] {
        Element::Buttons(labels) => {
            assert_eq!(labels.len(), 3, "Should have 3 buttons (Yes, No, Force Yield)");
            assert_eq!(labels[0], "Yes");
            assert_eq!(labels[1], "No");
            assert_eq!(labels[2], "Force Yield");
        }
        _ => panic!("Expected Buttons element"),
    }
}

#[test]
fn test_parse_settings_panel_ast() {
    let dsl = r#"Settings:
  Volume: 0-100 = 75
  Theme: Light | Dark | Auto
  Notifications: ✓
  Auto-save: yes
  Language: [English, Spanish, French, German]
  
  → Save Settings"#;
    
    let result = parse_popup_dsl(dsl).unwrap();
    
    // Debug print
    println!("Parsed {} elements:", result.elements.len());
    for (i, elem) in result.elements.iter().enumerate() {
        match elem {
            Element::Slider { label, min, max, default } => 
                println!("  {}: Slider('{}', {}-{}, default={})", i, label, min, max, default),
            Element::Choice { label, options } => 
                println!("  {}: Choice('{}', {:?})", i, label, options),
            Element::Checkbox { label, default } => 
                println!("  {}: Checkbox('{}', {})", i, label, default),
            Element::Multiselect { label, options } => 
                println!("  {}: Multiselect('{}', {:?})", i, label, options),
            Element::Buttons(labels) => 
                println!("  {}: Buttons({:?})", i, labels),
            Element::Text(text) => 
                println!("  {}: Text('{}')", i, text),
            _ => println!("  {}: Other element", i),
        }
    }
    
    assert_eq!(result.title, "Settings");
    
    // Verify each element
    let mut found_volume = false;
    let mut found_theme = false;
    let mut found_notifications = false;
    let mut found_autosave = false;
    let mut found_language = false;
    let mut found_buttons = false;
    
    for elem in &result.elements {
        match elem {
            Element::Slider { label, min, max, default } if label == "Volume" => {
                assert_eq!(*min, 0.0);
                assert_eq!(*max, 100.0);
                assert_eq!(*default, 75.0);
                found_volume = true;
            }
            Element::Choice { label, options } if label == "Theme" => {
                assert_eq!(options, &vec!["Light", "Dark", "Auto"]);
                found_theme = true;
            }
            Element::Checkbox { label, default } if label == "Notifications" => {
                assert_eq!(*default, true);
                found_notifications = true;
            }
            Element::Checkbox { label, default } if label == "Auto-save" => {
                assert_eq!(*default, true);
                found_autosave = true;
            }
            Element::Multiselect { label, options } if label == "Language" => {
                assert_eq!(options.len(), 4);
                found_language = true;
            }
            Element::Buttons(labels) => {
                assert!(labels.contains(&"Save Settings".to_string()), 
                    "Should have 'Save Settings' button, got: {:?}", labels);
                found_buttons = true;
            }
            _ => {}
        }
    }
    
    assert!(found_volume, "Should have Volume slider");
    assert!(found_theme, "Should have Theme choice");
    assert!(found_notifications, "Should have Notifications checkbox");
    assert!(found_autosave, "Should have Auto-save checkbox");
    assert!(found_language, "Should have Language multiselect");
    assert!(found_buttons, "Should have buttons");
}

#[test]
fn test_parse_warning_with_text_elements_ast() {
    let dsl = r#"System Warning:
  ! Critical updates are available
  > Your system will restart after installation
  ? This may take 10-15 minutes
  
  Install now: ☐
  Schedule for later: ☐
  
  [Install | Remind Me Later | Skip]"#;
    
    let result = parse_popup_dsl(dsl).unwrap();
    
    assert_eq!(result.title, "System Warning");
    
    // Count different element types
    let text_count = result.elements.iter()
        .filter(|e| matches!(e, Element::Text(_)))
        .count();
    let checkbox_count = result.elements.iter()
        .filter(|e| matches!(e, Element::Checkbox { .. }))
        .count();
    let button_count = result.elements.iter()
        .filter(|e| matches!(e, Element::Buttons(_)))
        .count();
    
    println!("Found {} text elements, {} checkboxes, {} button groups", 
        text_count, checkbox_count, button_count);
    
    assert!(text_count >= 3, "Should have at least 3 text elements");
    assert_eq!(checkbox_count, 2, "Should have 2 checkboxes");
    assert_eq!(button_count, 1, "Should have 1 button group");
    
    // Verify specific elements
    for elem in &result.elements {
        match elem {
            Element::Text(text) => {
                assert!(
                    text.contains("Critical updates") || 
                    text.contains("restart") || 
                    text.contains("10-15 minutes"),
                    "Text should be one of the expected messages"
                );
            }
            Element::Checkbox { label, default } => {
                assert!(
                    label == "Install now" || label == "Schedule for later",
                    "Checkbox label should be recognized"
                );
                assert_eq!(*default, false, "Checkboxes should default to false");
            }
            Element::Buttons(labels) => {
                assert_eq!(labels.len(), 4, "Should have 4 buttons including Force Yield");
                assert!(labels.contains(&"Install".to_string()));
                assert!(labels.contains(&"Remind Me Later".to_string()));
                assert!(labels.contains(&"Skip".to_string()));
            }
            _ => {}
        }
    }
}

#[test]
fn test_parse_choice_panel_ast() {
    let dsl = r#"Choose Action:
  Priority: Low | Medium | High
  Notify team: yes
  Add to calendar: ☐
  
  [Proceed | Cancel]"#;
    
    let result = parse_popup_dsl(dsl).unwrap();
    
    println!("\nChoice panel elements:");
    for (i, elem) in result.elements.iter().enumerate() {
        match elem {
            Element::Choice { label, options } => 
                println!("  {}: Choice('{}', {:?})", i, label, options),
            Element::Checkbox { label, default } => 
                println!("  {}: Checkbox('{}', {})", i, label, default),
            Element::Buttons(labels) => 
                println!("  {}: Buttons({:?})", i, labels),
            _ => println!("  {}: Other element", i),
        }
    }
    
    assert_eq!(result.title, "Choose Action");
    
    // Verify Priority choice
    let has_priority = result.elements.iter().any(|e| {
        matches!(e, Element::Choice { label, options } 
            if label == "Priority" && options == &vec!["Low", "Medium", "High"])
    });
    assert!(has_priority, "Should have Priority choice element");
    
    // Verify checkboxes
    let has_notify = result.elements.iter().any(|e| {
        matches!(e, Element::Checkbox { label, default } 
            if label == "Notify team" && *default == true)
    });
    assert!(has_notify, "Should have 'Notify team' checkbox set to true");
    
    let has_calendar = result.elements.iter().any(|e| {
        matches!(e, Element::Checkbox { label, default } 
            if label == "Add to calendar" && *default == false)
    });
    assert!(has_calendar, "Should have 'Add to calendar' checkbox set to false");
    
    // Verify buttons
    let has_buttons = result.elements.iter().any(|e| {
        matches!(e, Element::Buttons(labels) 
            if labels.contains(&"Proceed".to_string()) && labels.contains(&"Cancel".to_string()))
    });
    assert!(has_buttons, "Should have Proceed and Cancel buttons");
}

#[test]
fn test_arrow_button_parsing() {
    let dsl = r#"Settings:
  Volume: 0-100
  → Save Settings"#;
    
    let result = parse_popup_dsl(dsl).unwrap();
    
    let has_arrow_button = result.elements.iter().any(|e| {
        matches!(e, Element::Buttons(labels) if labels.contains(&"Save Settings".to_string()))
    });
    
    assert!(has_arrow_button, "Arrow button '→ Save Settings' should be parsed");
}

#[test]
fn test_conditional_parsing_ast() {
    let dsl = r#"Form:
  Account Type: Free | Premium
  
  when Account Type = Premium:
    Payment: Credit Card | PayPal
    Auto-renew: yes
  
  [Submit]"#;
    
    let result = parse_popup_dsl(dsl).unwrap();
    
    // Should have choice, conditional, and buttons
    let has_choice = result.elements.iter().any(|e| {
        matches!(e, Element::Choice { label, .. } if label == "Account Type")
    });
    assert!(has_choice, "Should have Account Type choice");
    
    let has_conditional = result.elements.iter().any(|e| {
        matches!(e, Element::Conditional { condition: Condition::Selected(field, value), elements } 
            if field == "Account Type" && value == "Premium" && !elements.is_empty())
    });
    assert!(has_conditional, "Should have conditional for Premium account type");
    
    // Check elements inside conditional
    for elem in &result.elements {
        if let Element::Conditional { elements, .. } = elem {
            let has_payment = elements.iter().any(|e| {
                matches!(e, Element::Choice { label, .. } if label == "Payment")
            });
            let has_autorenew = elements.iter().any(|e| {
                matches!(e, Element::Checkbox { label, default } 
                    if label == "Auto-renew" && *default == true)
            });
            
            assert!(has_payment, "Conditional should contain Payment choice");
            assert!(has_autorenew, "Conditional should contain Auto-renew checkbox");
        }
    }
}