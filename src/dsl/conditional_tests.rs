#[cfg(test)]
mod tests {
    use crate::dsl::simple_parser::parse_popup_dsl;
    use crate::models::{Element, Condition, ComparisonOp};

    #[test]
    fn test_simple_checkbox_conditional() {
        let input = r#"Settings
Advanced: no

[if Advanced] {
  Debug level: 0-10
  Log file: @/tmp/debug.log
}

[Save]"#;

        let result = parse_popup_dsl(input).unwrap();
        assert_eq!(result.title, "Settings");
        assert_eq!(result.elements.len(), 3);
        
        // Check checkbox
        match &result.elements[0] {
            Element::Checkbox { label, default } => {
                assert_eq!(label, "Advanced");
                assert_eq!(*default, false);
            }
            _ => panic!("Expected checkbox"),
        }
        
        // Check conditional
        match &result.elements[1] {
            Element::Conditional { condition, elements } => {
                match condition {
                    Condition::Checked(label) => assert_eq!(label, "Advanced"),
                    _ => panic!("Expected Checked condition"),
                }
                assert_eq!(elements.len(), 2);
                
                // Check nested slider
                match &elements[0] {
                    Element::Slider { label, min, max, .. } => {
                        assert_eq!(label, "Debug level");
                        assert_eq!(min, &0.0);
                        assert_eq!(max, &10.0);
                    }
                    _ => panic!("Expected slider"),
                }
                
                // Check nested textbox
                match &elements[1] {
                    Element::Textbox { label, placeholder, .. } => {
                        assert_eq!(label, "Log file");
                        assert_eq!(*placeholder, Some("/tmp/debug.log".to_string()));
                    }
                    _ => panic!("Expected textbox"),
                }
            }
            _ => panic!("Expected conditional"),
        }
    }

    #[test]
    fn test_choice_equals_conditional() {
        let input = r#"Theme Settings
Theme: Light | Dark | Auto

[if Theme = Dark] {
  Contrast: High | Normal | Low
  Glow effect: yes
}

[OK]"#;

        let result = parse_popup_dsl(input).unwrap();
        
        // Check choice widget
        match &result.elements[0] {
            Element::Choice { label, options } => {
                assert_eq!(label, "Theme");
                assert_eq!(options, &vec!["Light", "Dark", "Auto"]);
            }
            _ => panic!("Expected choice"),
        }
        
        // Check conditional with selection
        match &result.elements[1] {
            Element::Conditional { condition, elements } => {
                match condition {
                    Condition::Selected(label, value) => {
                        assert_eq!(label, "Theme");
                        assert_eq!(value, "Dark");
                    }
                    _ => panic!("Expected Selected condition"),
                }
                assert_eq!(elements.len(), 2);
            }
            _ => panic!("Expected conditional"),
        }
    }

    #[test]
    fn test_count_conditional() {
        let input = r#"Tags
Categories: [Work, Personal, Archive]

[if Categories > 1] {
  Priority: High | Medium | Low
}

[if Categories = 0] {
  ! Please select at least one category
}

[Done]"#;

        let result = parse_popup_dsl(input).unwrap();
        
        // Check multiselect
        match &result.elements[0] {
            Element::Multiselect { label, options } => {
                assert_eq!(label, "Categories");
                assert_eq!(options.len(), 3);
            }
            _ => panic!("Expected multiselect"),
        }
        
        // Check first conditional (> 1)
        match &result.elements[1] {
            Element::Conditional { condition, elements } => {
                match condition {
                    Condition::Count(label, op, count) => {
                        assert_eq!(label, "Categories");
                        assert_eq!(op, &ComparisonOp::Greater);
                        assert_eq!(*count, 1);
                    }
                    _ => panic!("Expected Count condition"),
                }
                assert_eq!(elements.len(), 1);
            }
            _ => panic!("Expected conditional"),
        }
        
        // Check second conditional (= 0)
        match &result.elements[2] {
            Element::Conditional { condition, elements } => {
                match condition {
                    Condition::Count(label, op, count) => {
                        assert_eq!(label, "Categories");
                        assert_eq!(op, &ComparisonOp::Equal);
                        assert_eq!(*count, 0);
                    }
                    _ => panic!("Expected Count condition"),
                }
                assert_eq!(elements.len(), 1);
                
                // Check warning message
                match &elements[0] {
                    Element::Text(text) => {
                        assert!(text.starts_with("⚠️"));
                    }
                    _ => panic!("Expected text"),
                }
            }
            _ => panic!("Expected conditional"),
        }
    }

    #[test]
    fn test_not_conditional() {
        let input = r#"Options
Auto-save: yes

[if not Auto-save] {
  Save interval: 5-60 = 30
  Reminder: yes
}

[OK]"#;

        let result = parse_popup_dsl(input).unwrap();
        
        // Check conditional with NOT
        match &result.elements[1] {
            Element::Conditional { condition, elements } => {
                match condition {
                    // We represent "not X" as Selected(X, "false")
                    Condition::Selected(label, value) => {
                        assert_eq!(label, "Auto-save");
                        assert_eq!(value, "false");
                    }
                    _ => panic!("Expected Selected condition for 'not'"),
                }
                assert_eq!(elements.len(), 2);
            }
            _ => panic!("Expected conditional"),
        }
    }

    #[test]
    fn test_has_conditional() {
        let input = r#"File Manager
Tags: [Important, Archive, Draft, Review]

[if Tags has Important] {
  ! This file is marked as important
  Backup: yes
}

[if Tags has Archive] {
  Compress: yes
}

[Done]"#;

        let result = parse_popup_dsl(input).unwrap();
        
        // Check first 'has' conditional
        match &result.elements[1] {
            Element::Conditional { condition, elements } => {
                match condition {
                    // We represent "X has Y" as Selected("X:has", Y)
                    Condition::Selected(label, value) => {
                        assert_eq!(label, "Tags:has");
                        assert_eq!(value, "Important");
                    }
                    _ => panic!("Expected Selected condition for 'has'"),
                }
                assert_eq!(elements.len(), 2);
            }
            _ => panic!("Expected conditional"),
        }
    }

    #[test]
    fn test_nested_conditionals() {
        let input = r#"Settings
Advanced: no

[if Advanced] {
  Debug: no
  
  [if Debug] {
    Level: 0-10 = 5
    Output: Console | File
  }
}

[Save]"#;

        let result = parse_popup_dsl(input).unwrap();
        
        // Check outer conditional
        match &result.elements[1] {
            Element::Conditional { condition: _, elements } => {
                assert_eq!(elements.len(), 2);
                
                // Check inner conditional
                match &elements[1] {
                    Element::Conditional { condition: inner_condition, elements: inner_elements } => {
                        match inner_condition {
                            Condition::Checked(label) => assert_eq!(label, "Debug"),
                            _ => panic!("Expected Checked condition"),
                        }
                        assert_eq!(inner_elements.len(), 2);
                    }
                    _ => panic!("Expected nested conditional"),
                }
            }
            _ => panic!("Expected conditional"),
        }
    }

    #[test]
    fn test_comparison_operators() {
        let input = r#"Test
Value: 0-100 = 50

[if Value > 75] {
  High: yes
}

[if Value < 25] {
  Low: yes
}

[if Value >= 50] {
  Mid or High: yes
}

[if Value <= 50] {
  Mid or Low: yes
}

[OK]"#;

        let result = parse_popup_dsl(input).unwrap();
        
        // Check > operator
        match &result.elements[1] {
            Element::Conditional { condition, .. } => {
                match condition {
                    Condition::Count(label, op, value) => {
                        assert_eq!(label, "Value");
                        assert_eq!(op, &ComparisonOp::Greater);
                        assert_eq!(*value, 75);
                    }
                    _ => panic!("Expected Count condition"),
                }
            }
            _ => panic!("Expected conditional"),
        }
        
        // Check < operator
        match &result.elements[2] {
            Element::Conditional { condition, .. } => {
                match condition {
                    Condition::Count(label, op, value) => {
                        assert_eq!(label, "Value");
                        assert_eq!(op, &ComparisonOp::Less);
                        assert_eq!(*value, 25);
                    }
                    _ => panic!("Expected Count condition"),
                }
            }
            _ => panic!("Expected conditional"),
        }
        
        // Check >= operator
        match &result.elements[3] {
            Element::Conditional { condition, .. } => {
                match condition {
                    Condition::Count(label, op, value) => {
                        assert_eq!(label, "Value");
                        assert_eq!(op, &ComparisonOp::GreaterEqual);
                        assert_eq!(*value, 50);
                    }
                    _ => panic!("Expected Count condition"),
                }
            }
            _ => panic!("Expected conditional"),
        }
        
        // Check <= operator
        match &result.elements[4] {
            Element::Conditional { condition, .. } => {
                match condition {
                    Condition::Count(label, op, value) => {
                        assert_eq!(label, "Value");
                        assert_eq!(op, &ComparisonOp::LessEqual);
                        assert_eq!(*value, 50);
                    }
                    _ => panic!("Expected Count condition"),
                }
            }
            _ => panic!("Expected conditional"),
        }
    }

    #[test]
    fn test_complex_conditional_form() {
        let input = r#"# Export Settings
Format: JSON | CSV | XML

[if Format = CSV] {
  Delimiter: Comma | Tab | Semicolon
  Headers: yes
}

[if Format = JSON] {
  Pretty: yes
  
  [if Pretty] {
    Indent: 2 | 4 | Tab
  }
}

Include metadata: no

[if Include metadata] {
  Fields: [Date, Author, Version]
  
  [if Fields > 0] {
    Location: Header | Footer
  }
}

[Export | Cancel]"#;

        let result = parse_popup_dsl(input).unwrap();
        assert_eq!(result.title, "Export Settings");
        
        // Verify we have all the expected elements
        let mut has_format_choice = false;
        let mut has_csv_conditional = false;
        let mut has_json_conditional = false;
        let mut has_metadata_checkbox = false;
        let mut has_metadata_conditional = false;
        let mut has_buttons = false;
        
        for element in &result.elements {
            match element {
                Element::Choice { label, .. } if label == "Format" => has_format_choice = true,
                Element::Conditional { condition, .. } => {
                    match condition {
                        Condition::Selected(label, value) if label == "Format" && value == "CSV" => {
                            has_csv_conditional = true;
                        }
                        Condition::Selected(label, value) if label == "Format" && value == "JSON" => {
                            has_json_conditional = true;
                        }
                        Condition::Checked(label) if label == "Include metadata" => {
                            has_metadata_conditional = true;
                        }
                        _ => {}
                    }
                }
                Element::Checkbox { label, .. } if label == "Include metadata" => {
                    has_metadata_checkbox = true;
                }
                Element::Buttons(_) => has_buttons = true,
                _ => {}
            }
        }
        
        assert!(has_format_choice, "Missing Format choice");
        assert!(has_csv_conditional, "Missing CSV conditional");
        assert!(has_json_conditional, "Missing JSON conditional");
        assert!(has_metadata_checkbox, "Missing metadata checkbox");
        assert!(has_metadata_conditional, "Missing metadata conditional");
        assert!(has_buttons, "Missing buttons");
    }
}