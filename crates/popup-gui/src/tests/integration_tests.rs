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
            {"slider": "Volume", "id": "volume", "min": 0, "max": 100, "default": 75},
            {"check": "Mute", "id": "mute", "default": false},
            {"input": "Name", "id": "name", "placeholder": "Enter name"},
            {"multi": "Features", "id": "features", "options": ["A", "B", "C"]}
        ]
    }"#;

    let popup = parse_popup_json(json).unwrap();
    let mut state = PopupState::new(&popup);

    // Check slider initialization
    assert_eq!(*state.get_number_mut("volume").unwrap(), 75.0);

    // Check checkbox initialization
    assert_eq!(*state.get_boolean_mut("mute").unwrap(), false);

    // Check textbox initialization
    assert_eq!(state.get_text_mut("name").unwrap(), "");

    // Check multiselect initialization
    assert_eq!(state.get_multichoice_mut("features").unwrap().len(), 3);
    assert!(state
        .get_multichoice_mut("features")
        .unwrap()
        .iter()
        .all(|&x| !x));
}

#[test]
fn test_popup_result_serialization() {
    let json = r#"{
        "title": "Result Test",
        "elements": [
            {"slider": "Value", "id": "value", "min": 0, "max": 10, "default": 5},
            {"check": "Enabled", "id": "enabled", "default": true},
            {"input": "Text", "id": "text", "placeholder": "..."}
        ]
    }"#;

    let popup = parse_popup_json(json).unwrap();
    let mut state = PopupState::new(&popup);

    // Modify state
    *state.get_number_mut("value").unwrap() = 7.0;
    *state.get_text_mut("text").unwrap() = "Hello".to_string();
    state.button_clicked = Some("submit".to_string());

    // Create result
    let result = PopupResult::from_state(&state);

    // Serialize to JSON
    let json_str = serde_json::to_string(&result).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    // Verify structure
    assert_eq!(parsed["value"], 7);
    assert_eq!(parsed["enabled"], true);
    assert_eq!(parsed["text"], "Hello");
    assert_eq!(parsed["button"], "submit");
}

#[test]
fn test_when_clause_in_json() {
    let json = r#"{
        "title": "When Clause",
        "elements": [
            {"check": "Show", "id": "show", "default": true},
            {"text": "This is shown when Show is checked", "id": "shown_text", "when": "@show"},
            {"slider": "Value", "id": "value", "min": 0, "max": 100, "when": "@show"}
        ]
    }"#;

    let popup = parse_popup_json(json).unwrap();
    let state = PopupState::new(&popup);

    // Verify structure
    assert_eq!(popup.elements.len(), 3);

    // State should initialize all elements (when clause doesn't affect initialization)
    assert!(state.values.get("value").is_some());
    assert!(state.values.get("show").is_some());
    assert!(state.values.get("shown_text").is_none()); // Text elements don't have state
}

#[test]
fn test_group_in_json() {
    let json = r#"{
        "title": "Grouped",
        "elements": [
            {
                "group": "Audio Settings",
                "id": "audio_group",
                "elements": [
                    {"slider": "Volume", "id": "volume", "min": 0, "max": 100},
                    {"slider": "Bass", "id": "bass", "min": -10, "max": 10, "default": 0},
                    {"check": "Surround", "id": "surround", "default": false}
                ]
            }
        ]
    }"#;

    let popup = parse_popup_json(json).unwrap();
    let state = PopupState::new(&popup);

    // Verify group structure
    assert_eq!(popup.elements.len(), 1);

    // State should initialize nested elements
    assert!(state.values.get("volume").is_some());
    assert!(state.values.get("bass").is_some());
    assert!(state.values.get("surround").is_some());
}
