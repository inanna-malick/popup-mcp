use crate::{Element, ElementValue, OptionValue, PopupDefinition, PopupResult, PopupState};
use std::collections::HashMap;

fn slider(label: &str, id: &str, min: f32, max: f32, default: Option<f32>) -> Element {
    Element::Slider {
        slider: label.to_string(),
        id: id.to_string(),
        min,
        max,
        default,
        step: None,
        when: None,
    }
}

fn check(label: &str, id: &str, default: bool, reveals: Vec<Element>) -> Element {
    Element::Check {
        check: label.to_string(),
        id: id.to_string(),
        default,
        reveals,
        when: None,
    }
}

fn select(
    label: &str,
    id: &str,
    options: &[&str],
    default: Option<&str>,
    option_children: HashMap<String, Vec<Element>>,
) -> Element {
    Element::Select {
        select: label.to_string(),
        id: id.to_string(),
        options: options
            .iter()
            .map(|o| OptionValue::Simple(o.to_string()))
            .collect(),
        default: default.map(|s| s.to_string()),
        style: None,
        option_children,
        reveals: vec![],
        when: None,
    }
}

#[test]
fn test_popupstate_init_slider() {
    let def = PopupDefinition::new(
        "Test",
        vec![slider("Volume", "vol", 0.0, 100.0, Some(75.0))],
    );

    let state = PopupState::new(&def);
    assert_eq!(state.values.get("vol"), Some(&ElementValue::Number(75.0)));
}

#[test]
fn test_popupstate_init_slider_default_midpoint() {
    let def = PopupDefinition::new("Test", vec![slider("Level", "level", 1.0, 10.0, None)]);

    let state = PopupState::new(&def);
    assert_eq!(state.values.get("level"), Some(&ElementValue::Number(5.5)));
}

#[test]
fn test_popupstate_init_checkbox() {
    let def = PopupDefinition::new("Test", vec![check("Enable", "enable", true, vec![])]);

    let state = PopupState::new(&def);
    assert_eq!(
        state.values.get("enable"),
        Some(&ElementValue::Boolean(true))
    );
}

#[test]
fn test_popupstate_init_with_reveals() {
    let def = PopupDefinition::new(
        "Test",
        vec![check(
            "Enable advanced",
            "enable",
            false,
            vec![slider("Level", "level", 1.0, 10.0, None)],
        )],
    );

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
        vec![slider("Brightness", "brightness", 0.0, 100.0, Some(50.0))],
    );

    let def = PopupDefinition::new(
        "Test",
        vec![select(
            "Theme",
            "theme",
            &["Light", "Dark"],
            None,
            option_children,
        )],
    );

    let state = PopupState::new(&def);
    assert_eq!(state.values.get("theme"), Some(&ElementValue::Choice(None)));
    assert_eq!(
        state.values.get("brightness"),
        Some(&ElementValue::Number(50.0))
    );
}

#[test]
fn test_popupstate_init_multiselect() {
    let def = PopupDefinition::new(
        "Test",
        vec![Element::Multi {
            multi: "Features".to_string(),
            id: "features".to_string(),
            options: vec![
                OptionValue::Simple("A".to_string()),
                OptionValue::Simple("B".to_string()),
                OptionValue::Simple("C".to_string()),
            ],
            default: vec![],
            option_children: HashMap::new(),
            reveals: vec![],
            when: None,
        }],
    );

    let state = PopupState::new(&def);
    assert_eq!(
        state.values.get("features"),
        Some(&ElementValue::MultiChoice(vec![false, false, false]))
    );
}

#[test]
fn test_popupstate_init_multiselect_with_defaults() {
    let def = PopupDefinition::new(
        "Test",
        vec![Element::Multi {
            multi: "Features".to_string(),
            id: "features".to_string(),
            options: vec![
                OptionValue::Simple("A".to_string()),
                OptionValue::Simple("B".to_string()),
                OptionValue::Simple("C".to_string()),
            ],
            default: vec!["A".to_string(), "C".to_string()],
            option_children: HashMap::new(),
            reveals: vec![],
            when: None,
        }],
    );

    let state = PopupState::new(&def);
    assert_eq!(
        state.values.get("features"),
        Some(&ElementValue::MultiChoice(vec![true, false, true]))
    );
}

#[test]
fn test_popupstate_init_textbox() {
    let def = PopupDefinition::new(
        "Test",
        vec![Element::Input {
            input: "Name".to_string(),
            id: "name".to_string(),
            placeholder: None,
            default: None,
            rows: None,
            when: None,
        }],
    );

    let state = PopupState::new(&def);
    assert_eq!(
        state.values.get("name"),
        Some(&ElementValue::Text(String::new()))
    );
}

#[test]
fn test_popupstate_init_textbox_with_default() {
    let def = PopupDefinition::new(
        "Test",
        vec![Element::Input {
            input: "Name".to_string(),
            id: "name".to_string(),
            placeholder: None,
            default: Some("proposed".to_string()),
            rows: None,
            when: None,
        }],
    );

    let state = PopupState::new(&def);
    assert_eq!(
        state.values.get("name"),
        Some(&ElementValue::Text("proposed".to_string()))
    );
}

#[test]
fn test_popupstate_init_group() {
    let def = PopupDefinition::new(
        "Test",
        vec![Element::Group {
            group: "Settings".to_string(),
            id: None,
            elements: vec![
                slider("Volume", "vol", 0.0, 100.0, Some(50.0)),
                check("Mute", "mute", false, vec![]),
            ],
            when: None,
        }],
    );

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
    let def = PopupDefinition::new(
        "Test",
        vec![
            slider("Volume", "vol", 0.0, 100.0, Some(50.0)),
            check("Enabled", "enabled", false, vec![]),
        ],
    );

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
        vec![slider("Complexity", "complexity", 1.0, 10.0, Some(5.0))],
    );

    let def = PopupDefinition::new(
        "Test",
        vec![check(
            "Show options",
            "show_opts",
            false,
            vec![select(
                "Mode",
                "mode",
                &["Basic", "Advanced"],
                Some("Basic"),
                option_children,
            )],
        )],
    );

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
fn test_from_state_finds_element_in_option_children() {
    let mut option_children = HashMap::new();
    option_children.insert(
        "Pro".to_string(),
        vec![slider(
            "Feature level",
            "feature_level",
            1.0,
            100.0,
            Some(50.0),
        )],
    );

    let def = PopupDefinition::new(
        "Test",
        vec![select(
            "Plan",
            "plan",
            &["Basic", "Pro"],
            Some("Pro"),
            option_children,
        )],
    );

    let mut state = PopupState::new(&def);
    state.button_clicked = Some("submit".to_string());

    // Modify the nested slider value
    if let Some(level) = state.get_number_mut("feature_level") {
        *level = 75.0;
    }

    let active = vec!["plan".to_string(), "feature_level".to_string()];
    let result = PopupResult::from_state(&state, &def, &active);

    match result {
        PopupResult::Completed { values, .. } => {
            // Should find the element and return the integer value + option text
            assert_eq!(values.get("feature_level").unwrap().as_i64(), Some(75));
            assert_eq!(values.get("plan").unwrap().as_str(), Some("Pro"));
        }
        _ => panic!("Expected Completed result"),
    }
}

#[test]
fn test_from_state_deep_nesting() {
    // Deep nesting: choice with option_children containing elements with reveals
    let mut option_children = HashMap::new();
    option_children.insert(
        "Advanced".to_string(),
        vec![check(
            "Debug mode",
            "debug",
            false,
            vec![slider("Debug level", "debug_level", 1.0, 10.0, Some(5.0))],
        )],
    );

    let def = PopupDefinition::new(
        "Test",
        vec![select(
            "Mode",
            "mode",
            &["Simple", "Advanced"],
            Some("Advanced"),
            option_children,
        )],
    );

    let mut state = PopupState::new(&def);
    state.button_clicked = Some("submit".to_string());

    // Modify the deeply nested slider value
    if let Some(level) = state.get_number_mut("debug_level") {
        *level = 8.0;
    }

    let active = vec![
        "mode".to_string(),
        "debug".to_string(),
        "debug_level".to_string(),
    ];
    let result = PopupResult::from_state(&state, &def, &active);

    match result {
        PopupResult::Completed { values, .. } => {
            assert_eq!(values.get("debug_level").unwrap().as_i64(), Some(8));
        }
        _ => panic!("Expected Completed result"),
    }
}

#[test]
fn test_from_state_result_semantics() {
    // absent = never shown; "" = shown but blank; null = shown but unselected
    let def = PopupDefinition::new(
        "Test",
        vec![
            Element::Input {
                input: "Blank".to_string(),
                id: "blank".to_string(),
                placeholder: None,
                default: None,
                rows: None,
                when: None,
            },
            Element::Input {
                input: "Hidden".to_string(),
                id: "hidden".to_string(),
                placeholder: None,
                default: None,
                rows: None,
                when: Some("false".to_string()),
            },
            select("Choice", "choice", &["A", "B"], None, HashMap::new()),
            slider("Fraction", "fraction", 0.0, 1.0, Some(0.5)),
        ],
    );

    let mut state = PopupState::new(&def);
    state.button_clicked = Some("submit".to_string());

    let active = vec![
        "blank".to_string(),
        "choice".to_string(),
        "fraction".to_string(),
    ];
    let result = PopupResult::from_state(&state, &def, &active);

    match result {
        PopupResult::Completed { values, .. } => {
            // shown-but-blank input: present as ""
            assert_eq!(values.get("blank").unwrap().as_str(), Some(""));
            // hidden input: absent
            assert!(!values.contains_key("hidden"));
            // unselected choice: null
            assert!(values.get("choice").unwrap().is_null());
            // fractional slider values survive (no integer truncation)
            assert_eq!(values.get("fraction").unwrap().as_f64(), Some(0.5));
            // no note was entered
            assert!(!values.contains_key("_notes"));
        }
        _ => panic!("Expected Completed result"),
    }
}

#[test]
fn test_rank_state_and_result() {
    let def = PopupDefinition::new(
        "Test",
        vec![Element::Rank {
            rank: "Priorities".to_string(),
            id: "priorities".to_string(),
            options: vec![
                OptionValue::Simple("Speed".to_string()),
                OptionValue::Simple("Cost".to_string()),
                OptionValue::Simple("Quality".to_string()),
            ],
            when: None,
        }],
    );

    let mut state = PopupState::new(&def);
    assert_eq!(
        state.values.get("priorities"),
        Some(&ElementValue::Order(vec![]))
    );

    // Rank Quality first, then Speed; Cost left unranked (partial)
    state.get_order_mut("priorities").unwrap().extend([2, 0]);

    // Value map: ranked texts in order, so count()/selected() work
    let values = state.to_value_map(&def.elements);
    assert_eq!(
        values.get("priorities").unwrap(),
        &serde_json::json!(["Quality", "Speed"])
    );

    // Result: same ranked array; unranked omitted
    state.button_clicked = Some("submit".to_string());
    let result = PopupResult::from_state(&state, &def, &["priorities".to_string()]);
    match result {
        PopupResult::Completed { values, .. } => {
            assert_eq!(
                values.get("priorities").unwrap(),
                &serde_json::json!(["Quality", "Speed"])
            );
        }
        _ => panic!("Expected Completed result"),
    }
}

#[test]
fn test_rank_works_with_conditions() {
    use crate::{evaluate_condition, parse_condition};

    let def = PopupDefinition::new(
        "Test",
        vec![Element::Rank {
            rank: "Priorities".to_string(),
            id: "priorities".to_string(),
            options: vec![
                OptionValue::Simple("Speed".to_string()),
                OptionValue::Simple("Cost".to_string()),
            ],
            when: None,
        }],
    );

    let mut state = PopupState::new(&def);
    state.get_order_mut("priorities").unwrap().push(1); // rank "Cost"

    let values = state.to_value_map(&def.elements);
    assert!(evaluate_condition(
        &parse_condition("selected(priorities, Cost)").unwrap(),
        &values
    ));
    assert!(!evaluate_condition(
        &parse_condition("selected(priorities, Speed)").unwrap(),
        &values
    ));
    assert!(evaluate_condition(
        &parse_condition("count(priorities) == 1").unwrap(),
        &values
    ));
}

#[test]
fn test_from_state_includes_labels() {
    let def = PopupDefinition::new(
        "Test",
        vec![
            check("Enable retries", "retries", true, vec![]),
            slider("Max attempts", "attempts", 1.0, 10.0, Some(3.0)),
        ],
    );

    let mut state = PopupState::new(&def);
    state.button_clicked = Some("submit".to_string());

    let active = vec!["retries".to_string(), "attempts".to_string()];
    let result = PopupResult::from_state(&state, &def, &active);
    match result {
        PopupResult::Completed { values, .. } => {
            let labels = values.get("_labels").unwrap().as_object().unwrap();
            assert_eq!(labels.get("retries").unwrap(), "Enable retries");
            assert_eq!(labels.get("attempts").unwrap(), "Max attempts");
        }
        _ => panic!("Expected Completed result"),
    }
}

#[test]
fn test_from_state_includes_notes() {
    let def = PopupDefinition::new("Test", vec![check("Ok", "ok", true, vec![])]);

    let mut state = PopupState::new(&def);
    state.button_clicked = Some("submit".to_string());
    state.notes = "  actually, please also check the logs  ".to_string();

    let result = PopupResult::from_state(&state, &def, &["ok".to_string()]);
    match result {
        PopupResult::Completed { values, .. } => {
            assert_eq!(
                values.get("_notes").unwrap().as_str(),
                Some("actually, please also check the logs")
            );
        }
        _ => panic!("Expected Completed result"),
    }
}
