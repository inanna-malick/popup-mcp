use crate::{Element, ElementValue, OptionValue, PopupDefinition, PopupState};
use std::collections::HashMap;

#[test]
fn test_popupstate_init_slider() {
    let def = PopupDefinition {
        title: "Test".to_string(),
        elements: vec![Element::Slider {
            slider: "Volume".to_string(),
            id: "vol".to_string(),
            min: 0.0,
            max: 100.0,
            default: Some(75.0),
            when: None,
        }],
    };

    let state = PopupState::new(&def);
    assert_eq!(state.values.get("vol"), Some(&ElementValue::Number(75.0)));
}

#[test]
fn test_popupstate_init_slider_default_midpoint() {
    let def = PopupDefinition {
        title: "Test".to_string(),
        elements: vec![Element::Slider {
            slider: "Level".to_string(),
            id: "level".to_string(),
            min: 1.0,
            max: 10.0,
            default: None, // Should default to midpoint
            when: None,
        }],
    };

    let state = PopupState::new(&def);
    assert_eq!(state.values.get("level"), Some(&ElementValue::Number(5.5)));
}

#[test]
fn test_popupstate_init_checkbox() {
    let def = PopupDefinition {
        title: "Test".to_string(),
        elements: vec![Element::Check {
            check: "Enable".to_string(),
            id: "enable".to_string(),
            default: true,
            reveals: vec![],
            when: None,
        }],
    };

    let state = PopupState::new(&def);
    assert_eq!(
        state.values.get("enable"),
        Some(&ElementValue::Boolean(true))
    );
}

#[test]
fn test_popupstate_init_with_reveals() {
    let def = PopupDefinition {
        title: "Test".to_string(),
        elements: vec![Element::Check {
            check: "Enable advanced".to_string(),
            id: "enable".to_string(),
            default: false,
            reveals: vec![Element::Slider {
                slider: "Level".to_string(),
                id: "level".to_string(),
                min: 1.0,
                max: 10.0,
                default: None,
                when: None,
            }],
            when: None,
        }],
    };

    let state = PopupState::new(&def);
    assert_eq!(
        state.values.get("enable"),
        Some(&ElementValue::Boolean(false))
    );
    assert_eq!(state.values.get("level"), Some(&ElementValue::Number(5.5))); // midpoint
}

#[test]
fn test_popupstate_init_with_option_children() {
    let mut option_children = HashMap::new();
    option_children.insert(
        "Dark".to_string(),
        vec![Element::Slider {
            slider: "Brightness".to_string(),
            id: "brightness".to_string(),
            min: 0.0,
            max: 100.0,
            default: Some(50.0),
            when: None,
        }],
    );

    let def = PopupDefinition {
        title: "Test".to_string(),
        elements: vec![Element::Select {
            select: "Theme".to_string(),
            id: "theme".to_string(),
            options: vec![
                OptionValue::Simple("Light".to_string()),
                OptionValue::Simple("Dark".to_string()),
            ],
            default: None,
            option_children,
            reveals: vec![],
            when: None,
        }],
    };

    let state = PopupState::new(&def);
    assert_eq!(state.values.get("theme"), Some(&ElementValue::Choice(None)));
    assert_eq!(
        state.values.get("brightness"),
        Some(&ElementValue::Number(50.0))
    );
}

#[test]
fn test_popupstate_init_multiselect() {
    let def = PopupDefinition {
        title: "Test".to_string(),
        elements: vec![Element::Multi {
            multi: "Features".to_string(),
            id: "features".to_string(),
            options: vec![
                OptionValue::Simple("A".to_string()),
                OptionValue::Simple("B".to_string()),
                OptionValue::Simple("C".to_string()),
            ],
            option_children: HashMap::new(),
            reveals: vec![],
            when: None,
        }],
    };

    let state = PopupState::new(&def);
    assert_eq!(
        state.values.get("features"),
        Some(&ElementValue::MultiChoice(vec![false, false, false]))
    );
}

#[test]
fn test_popupstate_init_textbox() {
    let def = PopupDefinition {
        title: "Test".to_string(),
        elements: vec![Element::Input {
            input: "Name".to_string(),
            id: "name".to_string(),
            placeholder: None,
            rows: None,
            when: None,
        }],
    };

    let state = PopupState::new(&def);
    assert_eq!(
        state.values.get("name"),
        Some(&ElementValue::Text(String::new()))
    );
}

#[test]
fn test_popupstate_init_group() {
    let def = PopupDefinition {
        title: "Test".to_string(),
        elements: vec![Element::Group {
            group: "Settings".to_string(),
            id: None,
            elements: vec![
                Element::Slider {
                    slider: "Volume".to_string(),
                    id: "vol".to_string(),
                    min: 0.0,
                    max: 100.0,
                    default: Some(50.0),
                    when: None,
                },
                Element::Check {
                    check: "Mute".to_string(),
                    id: "mute".to_string(),
                    default: false,
                    reveals: vec![],
                    when: None,
                },
            ],
            when: None,
        }],
    };

    let state = PopupState::new(&def);
    assert_eq!(state.values.get("vol"), Some(&ElementValue::Number(50.0)));
    assert_eq!(
        state.values.get("mute"),
        Some(&ElementValue::Boolean(false))
    );
}

#[test]
fn test_to_value_map() {
    let mut state = PopupState::default();
    state
        .values
        .insert("cpu".to_string(), ElementValue::Number(85.0));
    state
        .values
        .insert("enabled".to_string(), ElementValue::Boolean(true));
    state
        .values
        .insert("name".to_string(), ElementValue::Text("Alice".to_string()));

    // Empty elements array for this simple test
    let value_map = state.to_value_map(&[]);
    assert_eq!(value_map.get("cpu").unwrap().as_f64(), Some(85.0));
    assert_eq!(value_map.get("enabled").unwrap().as_bool(), Some(true));
    assert_eq!(value_map.get("name").unwrap().as_str(), Some("Alice"));
}

#[test]
fn test_get_mut_methods() {
    let def = PopupDefinition {
        title: "Test".to_string(),
        elements: vec![
            Element::Slider {
                slider: "Volume".to_string(),
                id: "vol".to_string(),
                min: 0.0,
                max: 100.0,
                default: Some(50.0),
                when: None,
            },
            Element::Check {
                check: "Enabled".to_string(),
                id: "enabled".to_string(),
                default: false,
                reveals: vec![],
                when: None,
            },
        ],
    };

    let mut state = PopupState::new(&def);

    // Test get_number_mut
    if let Some(vol) = state.get_number_mut("vol") {
        *vol = 75.0;
    }
    assert_eq!(state.values.get("vol"), Some(&ElementValue::Number(75.0)));

    // Test get_boolean_mut
    if let Some(enabled) = state.get_boolean_mut("enabled") {
        *enabled = true;
    }
    assert_eq!(
        state.values.get("enabled"),
        Some(&ElementValue::Boolean(true))
    );
}

#[test]
fn test_nested_reveals_and_option_children() {
    // Complex nested structure: checkbox with reveals, choice with option_children
    let mut option_children = HashMap::new();
    option_children.insert(
        "Advanced".to_string(),
        vec![Element::Slider {
            slider: "Complexity".to_string(),
            id: "complexity".to_string(),
            min: 1.0,
            max: 10.0,
            default: Some(5.0),
            when: None,
        }],
    );

    let def = PopupDefinition {
        title: "Test".to_string(),
        elements: vec![Element::Check {
            check: "Show options".to_string(),
            id: "show_opts".to_string(),
            default: false,
            reveals: vec![Element::Select {
                select: "Mode".to_string(),
                id: "mode".to_string(),
                options: vec![
                    OptionValue::Simple("Basic".to_string()),
                    OptionValue::Simple("Advanced".to_string()),
                ],
                default: Some("Basic".to_string()),
                option_children,
                reveals: vec![],
                when: None,
            }],
            when: None,
        }],
    };

    let state = PopupState::new(&def);
    assert_eq!(
        state.values.get("show_opts"),
        Some(&ElementValue::Boolean(false))
    );
    assert_eq!(
        state.values.get("mode"),
        Some(&ElementValue::Choice(Some(0)))
    );
    assert_eq!(
        state.values.get("complexity"),
        Some(&ElementValue::Number(5.0))
    );
}

#[test]
fn test_find_element_in_option_children() {
    // Test that from_state_with_context can find elements nested in option_children
    // This validates that the find_element_by_id function searches option_children correctly
    use crate::PopupResult;

    let mut option_children = HashMap::new();
    option_children.insert(
        "Pro".to_string(),
        vec![Element::Slider {
            slider: "Feature level".to_string(),
            id: "feature_level".to_string(),
            min: 1.0,
            max: 100.0,
            default: Some(50.0),
            when: None,
        }],
    );

    let def = PopupDefinition {
        title: "Test".to_string(),
        elements: vec![Element::Select {
            select: "Plan".to_string(),
            id: "plan".to_string(),
            options: vec![
                OptionValue::Simple("Basic".to_string()),
                OptionValue::Simple("Pro".to_string()),
            ],
            default: Some("Pro".to_string()),
            option_children,
            reveals: vec![],
            when: None,
        }],
    };

    let mut state = PopupState::new(&def);
    state.button_clicked = Some("submit".to_string());

    // Modify the nested slider value
    if let Some(level) = state.get_number_mut("feature_level") {
        *level = 75.0;
    }

    // Use from_state_with_context which calls find_element_by_id
    let result = PopupResult::from_state_with_context(&state, &def);

    match result {
        PopupResult::Completed { values, .. } => {
            // Should find the element and return the integer value
            assert_eq!(values.get("feature_level").unwrap().as_i64(), Some(75));
            assert_eq!(values.get("plan").unwrap().as_str(), Some("Pro"));
        }
        _ => panic!("Expected Completed result"),
    }
}

#[test]
fn test_find_element_in_nested_reveals_and_option_children() {
    // Test deep nesting: choice with option_children containing elements with reveals
    use crate::PopupResult;

    let mut option_children = HashMap::new();
    option_children.insert(
        "Advanced".to_string(),
        vec![Element::Check {
            check: "Debug mode".to_string(),
            id: "debug".to_string(),
            default: false,
            reveals: vec![Element::Slider {
                slider: "Debug level".to_string(),
                id: "debug_level".to_string(),
                min: 1.0,
                max: 10.0,
                default: Some(5.0),
                when: None,
            }],
            when: None,
        }],
    );

    let def = PopupDefinition {
        title: "Test".to_string(),
        elements: vec![Element::Select {
            select: "Mode".to_string(),
            id: "mode".to_string(),
            options: vec![
                OptionValue::Simple("Simple".to_string()),
                OptionValue::Simple("Advanced".to_string()),
            ],
            default: Some("Advanced".to_string()),
            option_children,
            reveals: vec![],
            when: None,
        }],
    };

    let mut state = PopupState::new(&def);
    state.button_clicked = Some("submit".to_string());

    // Modify the deeply nested slider value
    if let Some(level) = state.get_number_mut("debug_level") {
        *level = 8.0;
    }

    // Use from_state_with_context which needs to find element in option_children -> reveals
    let result = PopupResult::from_state_with_context(&state, &def);

    match result {
        PopupResult::Completed { values, .. } => {
            // Should find the deeply nested element and return integer value
            assert_eq!(values.get("debug_level").unwrap().as_i64(), Some(8));
        }
        _ => panic!("Expected Completed result"),
    }
}
