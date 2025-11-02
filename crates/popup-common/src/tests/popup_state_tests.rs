use crate::{Element, ElementValue, PopupDefinition, PopupState};
use std::collections::HashMap;

#[test]
fn test_popupstate_init_slider() {
    let def = PopupDefinition {
        title: None,
        elements: vec![Element::Slider {
            slider: "Volume".to_string(),
            id: "vol".to_string(),
            min: 0.0,
            max: 100.0,
            default: Some(75.0),
            reveals: vec![],
            when: None,
        }],
    };

    let state = PopupState::new(&def);
    assert_eq!(state.values.get("vol"), Some(&ElementValue::Number(75.0)));
}

#[test]
fn test_popupstate_init_slider_default_midpoint() {
    let def = PopupDefinition {
        title: None,
        elements: vec![Element::Slider {
            slider: "Level".to_string(),
            id: "level".to_string(),
            min: 1.0,
            max: 10.0,
            default: None, // Should default to midpoint
            reveals: vec![],
            when: None,
        }],
    };

    let state = PopupState::new(&def);
    assert_eq!(state.values.get("level"), Some(&ElementValue::Number(5.5)));
}

#[test]
fn test_popupstate_init_checkbox() {
    let def = PopupDefinition {
        title: None,
        elements: vec![Element::Checkbox {
            checkbox: "Enable".to_string(),
            id: "enable".to_string(),
            default: true,
            reveals: vec![],
            when: None,
        }],
    };

    let state = PopupState::new(&def);
    assert_eq!(state.values.get("enable"), Some(&ElementValue::Boolean(true)));
}

#[test]
fn test_popupstate_init_with_reveals() {
    let def = PopupDefinition {
        title: None,
        elements: vec![Element::Checkbox {
            checkbox: "Enable advanced".to_string(),
            id: "enable".to_string(),
            default: false,
            reveals: vec![Element::Slider {
                slider: "Level".to_string(),
                id: "level".to_string(),
                min: 1.0,
                max: 10.0,
                default: None,
                reveals: vec![],
                when: None,
            }],
            when: None,
        }],
    };

    let state = PopupState::new(&def);
    assert_eq!(state.values.get("enable"), Some(&ElementValue::Boolean(false)));
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
            reveals: vec![],
            when: None,
        }],
    );

    let def = PopupDefinition {
        title: None,
        elements: vec![Element::Choice {
            choice: "Theme".to_string(),
            id: "theme".to_string(),
            options: vec!["Light".to_string(), "Dark".to_string()],
            default: None,
            option_children,
            reveals: vec![],
            when: None,
        }],
    };

    let state = PopupState::new(&def);
    assert_eq!(state.values.get("theme"), Some(&ElementValue::Choice(None)));
    assert_eq!(state.values.get("brightness"), Some(&ElementValue::Number(50.0)));
}

#[test]
fn test_popupstate_init_multiselect() {
    let def = PopupDefinition {
        title: None,
        elements: vec![Element::Multiselect {
            multiselect: "Features".to_string(),
            id: "features".to_string(),
            options: vec!["A".to_string(), "B".to_string(), "C".to_string()],
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
        title: None,
        elements: vec![Element::Textbox {
            textbox: "Name".to_string(),
            id: "name".to_string(),
            placeholder: None,
            rows: None,
            reveals: vec![],
            when: None,
        }],
    };

    let state = PopupState::new(&def);
    assert_eq!(state.values.get("name"), Some(&ElementValue::Text(String::new())));
}

#[test]
fn test_popupstate_init_group() {
    let def = PopupDefinition {
        title: None,
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
                    reveals: vec![],
                    when: None,
                },
                Element::Checkbox {
                    checkbox: "Mute".to_string(),
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
    assert_eq!(state.values.get("mute"), Some(&ElementValue::Boolean(false)));
}

#[test]
fn test_to_value_map() {
    let mut state = PopupState::default();
    state.values.insert("cpu".to_string(), ElementValue::Number(85.0));
    state.values.insert("enabled".to_string(), ElementValue::Boolean(true));
    state.values.insert("name".to_string(), ElementValue::Text("Alice".to_string()));

    let value_map = state.to_value_map();
    assert_eq!(value_map.get("cpu").unwrap().as_f64(), Some(85.0));
    assert_eq!(value_map.get("enabled").unwrap().as_bool(), Some(true));
    assert_eq!(value_map.get("name").unwrap().as_str(), Some("Alice"));
}

#[test]
fn test_get_mut_methods() {
    let def = PopupDefinition {
        title: None,
        elements: vec![
            Element::Slider {
                slider: "Volume".to_string(),
                id: "vol".to_string(),
                min: 0.0,
                max: 100.0,
                default: Some(50.0),
                reveals: vec![],
                when: None,
            },
            Element::Checkbox {
                checkbox: "Enabled".to_string(),
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
    assert_eq!(state.values.get("enabled"), Some(&ElementValue::Boolean(true)));
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
            reveals: vec![],
            when: None,
        }],
    );

    let def = PopupDefinition {
        title: None,
        elements: vec![Element::Checkbox {
            checkbox: "Show options".to_string(),
            id: "show_opts".to_string(),
            default: false,
            reveals: vec![Element::Choice {
                choice: "Mode".to_string(),
                id: "mode".to_string(),
                options: vec!["Basic".to_string(), "Advanced".to_string()],
                default: Some(0),
                option_children,
                reveals: vec![],
                when: None,
            }],
            when: None,
        }],
    };

    let state = PopupState::new(&def);
    assert_eq!(state.values.get("show_opts"), Some(&ElementValue::Boolean(false)));
    assert_eq!(state.values.get("mode"), Some(&ElementValue::Choice(Some(0))));
    assert_eq!(state.values.get("complexity"), Some(&ElementValue::Number(5.0)));
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
            reveals: vec![],
            when: None,
        }],
    );

    let def = PopupDefinition {
        title: Some("Test".to_string()),
        elements: vec![Element::Choice {
            choice: "Plan".to_string(),
            id: "plan".to_string(),
            options: vec!["Basic".to_string(), "Pro".to_string()],
            default: Some(1), // Select "Pro"
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
            // Should find the element and format it as "75/100"
            assert_eq!(values.get("feature_level").unwrap().as_str(), Some("75/100"));
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
        vec![Element::Checkbox {
            checkbox: "Debug mode".to_string(),
            id: "debug".to_string(),
            default: false,
            reveals: vec![Element::Slider {
                slider: "Debug level".to_string(),
                id: "debug_level".to_string(),
                min: 1.0,
                max: 10.0,
                default: Some(5.0),
                reveals: vec![],
                when: None,
            }],
            when: None,
        }],
    );

    let def = PopupDefinition {
        title: Some("Test".to_string()),
        elements: vec![Element::Choice {
            choice: "Mode".to_string(),
            id: "mode".to_string(),
            options: vec!["Simple".to_string(), "Advanced".to_string()],
            default: Some(1),
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
            // Should find the deeply nested element
            assert_eq!(values.get("debug_level").unwrap().as_str(), Some("8/10"));
        }
        _ => panic!("Expected Completed result"),
    }
}
