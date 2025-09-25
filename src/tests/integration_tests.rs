use crate::json_parser::parse_popup_json;
use crate::models::{PopupResult, PopupState};
use std::fs;

#[test]
fn test_parse_example_files() {
    let examples_dir = "examples";

    // Test all JSON example files
    for entry in fs::read_dir(examples_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let content = fs::read_to_string(&path).unwrap();
            let result = parse_popup_json(&content);

            assert!(
                result.is_ok(),
                "Failed to parse {:?}: {:?}",
                path,
                result.err()
            );

            let popup = result.unwrap();
            assert!(
                popup.title.as_ref().map_or(false, |t| !t.is_empty()),
                "Title should not be empty in {:?}",
                path
            );

            // Create state and verify it initializes correctly
            let state = PopupState::new(&popup);
            let result = PopupResult::from_state(&state);

            // Verify JSON serialization works
            let json = serde_json::to_string(&result).unwrap();
            assert!(!json.is_empty());
        }
    }
}

#[test]
fn test_popup_state_initialization() {
    let json = r#"{
        "title": "State Test",
        "elements": [
            {"type": "slider", "label": "Volume", "min": 0, "max": 100, "default": 75},
            {"type": "checkbox", "label": "Mute", "default": false},
            {"type": "textbox", "label": "Name", "placeholder": "Enter name"},
            {"type": "choice", "label": "Quality", "options": ["Low", "Medium", "High"]},
            {"type": "multiselect", "label": "Features", "options": ["A", "B", "C"]}
        ]
    }"#;

    let popup = parse_popup_json(json).unwrap();
    let mut state = PopupState::new(&popup);

    // Check slider initialization
    assert_eq!(*state.get_number_mut("Volume").unwrap(), 75.0);

    // Check checkbox initialization
    assert_eq!(*state.get_boolean_mut("Mute").unwrap(), false);

    // Check textbox initialization
    assert_eq!(state.get_text_mut("Name").unwrap(), "");

    // Check choice initialization
    assert_eq!(*state.get_choice_mut("Quality").unwrap(), 0);

    // Check multiselect initialization
    assert_eq!(state.get_multichoice_mut("Features").unwrap().len(), 3);
    assert!(state
        .get_multichoice_mut("Features")
        .unwrap()
        .iter()
        .all(|&x| !x));
}

#[test]
fn test_popup_result_serialization() {
    let json = r#"{
        "title": "Result Test",
        "elements": [
            {"type": "slider", "label": "Value", "min": 0, "max": 10, "default": 5},
            {"type": "checkbox", "label": "Enabled", "default": true},
            {"type": "textbox", "label": "Text", "placeholder": "..."}
        ]
    }"#;

    let popup = parse_popup_json(json).unwrap();
    let mut state = PopupState::new(&popup);

    // Modify state
    *state.get_number_mut("Value").unwrap() = 7.0;
    *state.get_text_mut("Text").unwrap() = "Hello".to_string();
    state.button_clicked = Some("submit".to_string());

    // Create result
    let result = PopupResult::from_state(&state);

    // Serialize to JSON
    let json_str = serde_json::to_string(&result).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    // Verify structure
    assert_eq!(parsed["Value"], 7);
    assert_eq!(parsed["Enabled"], true);
    assert_eq!(parsed["Text"], "Hello");
    assert_eq!(parsed["button"], "submit");
}

#[test]
fn test_conditional_in_json() {
    let json = r#"{
        "title": "Conditional",
        "elements": [
            {"type": "checkbox", "label": "Show", "default": true},
            {
                "type": "conditional",
                "condition": "Show",
                "elements": [
                    {"type": "text", "content": "This is shown when Show is checked"},
                    {"type": "slider", "label": "Value", "min": 0, "max": 100}
                ]
            }
        ]
    }"#;

    let popup = parse_popup_json(json).unwrap();
    let state = PopupState::new(&popup);

    // Verify conditional has proper structure
    assert_eq!(popup.elements.len(), 2);

    // State should still initialize nested elements
    assert!(state.values.contains_key("Value"));
}

#[test]
fn test_group_in_json() {
    let json = r#"{
        "title": "Grouped",
        "elements": [
            {
                "type": "group",
                "label": "Audio Settings",
                "elements": [
                    {"type": "slider", "label": "Volume", "min": 0, "max": 100},
                    {"type": "slider", "label": "Bass", "min": -10, "max": 10, "default": 0},
                    {"type": "checkbox", "label": "Surround", "default": false}
                ]
            }
        ]
    }"#;

    let popup = parse_popup_json(json).unwrap();
    let state = PopupState::new(&popup);

    // Verify group structure
    assert_eq!(popup.elements.len(), 1);

    // State should initialize nested elements
    assert!(state.values.contains_key("Volume"));
    assert!(state.values.contains_key("Bass"));
    assert!(state.values.contains_key("Surround"));
}
