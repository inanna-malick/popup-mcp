use crate::json_parser::parse_popup_json;
use popup_common::{PopupResult, PopupState};
use std::fs;

#[test]
fn test_parse_example_files() {
    // Tests run from workspace root - use CARGO_MANIFEST_DIR to find workspace root
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let workspace_root = std::path::Path::new(&manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let examples_dir = workspace_root.join("examples");

    // Test all JSON example files
    for entry in fs::read_dir(&examples_dir).unwrap() {
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
            {"type": "multiselect", "label": "Features", "options": ["A", "B", "C"]}
        ]
    }"#;

    let popup = parse_popup_json(json).unwrap();
    let mut state = PopupState::new(&popup);

    // Check slider initialization
    let volume_key = state.find_key_by_label("Volume").unwrap();
    assert_eq!(*state.get_number_mut(&volume_key).unwrap(), 75.0);

    // Check checkbox initialization
    let mute_key = state.find_key_by_label("Mute").unwrap();
    assert_eq!(*state.get_boolean_mut(&mute_key).unwrap(), false);

    // Check textbox initialization
    let name_key = state.find_key_by_label("Name").unwrap();
    assert_eq!(state.get_text_mut(&name_key).unwrap(), "");

    // Check multiselect initialization
    let features_key = state.find_key_by_label("Features").unwrap();
    assert_eq!(state.get_multichoice_mut(&features_key).unwrap().len(), 3);
    let features_key2 = state.find_key_by_label("Features").unwrap();
    assert!(state
        .get_multichoice_mut(&features_key2)
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
    let value_key = state.find_key_by_label("Value").unwrap();
    *state.get_number_mut(&value_key).unwrap() = 7.0;
    let text_key = state.find_key_by_label("Text").unwrap();
    *state.get_text_mut(&text_key).unwrap() = "Hello".to_string();
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
    assert!(state.find_key_by_label("Value").is_some());
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
    assert!(state.find_key_by_label("Volume").is_some());
    assert!(state.find_key_by_label("Bass").is_some());
    assert!(state.find_key_by_label("Surround").is_some());
}
