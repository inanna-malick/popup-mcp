use crate::json_parser::parse_popup_json;
use popup_common::Element;

#[test]
fn test_simple_confirmation() {
    let json = r#"{
        "title": "Confirm",
        "elements": [
            {"text": "Are you sure?", "id": "confirm_text"}
        ]
    }"#;

    let popup = parse_popup_json(json).unwrap();
    assert_eq!(popup.title, Some("Confirm".to_string()));
    assert_eq!(popup.elements.len(), 1);

    match &popup.elements[0] {
        Element::Text { text, .. } => assert_eq!(text, "Are you sure?"),
        _ => panic!("Expected text element"),
    }
}

#[test]
fn test_all_widget_types() {
    let json = r#"{
        "title": "All Widgets",
        "elements": [
            {"text": "Test all widget types", "id": "intro"},
            {"slider": "Volume", "id": "volume", "min": 0, "max": 100, "default": 50},
            {"checkbox": "Enable", "id": "enable", "default": true},
            {"textbox": "Name", "id": "name", "placeholder": "Enter name"},
            {"multiselect": "Features", "id": "features", "options": ["A", "B", "C"]},
            {"choice": "Mode", "id": "mode", "options": ["X", "Y", "Z"]}
        ]
    }"#;

    let popup = parse_popup_json(json).unwrap();
    assert_eq!(popup.elements.len(), 6);

    // Verify slider
    match &popup.elements[1] {
        Element::Slider {
            slider,
            id,
            min,
            max,
            default,
            ..
        } => {
            assert_eq!(slider, "Volume");
            assert_eq!(id, "volume");
            assert_eq!(*min, 0.0);
            assert_eq!(*max, 100.0);
            assert_eq!(*default, Some(50.0));
        }
        _ => panic!("Expected slider"),
    }

    // Verify checkbox
    match &popup.elements[2] {
        Element::Checkbox {
            checkbox,
            id,
            default,
            ..
        } => {
            assert_eq!(checkbox, "Enable");
            assert_eq!(id, "enable");
            assert_eq!(*default, true);
        }
        _ => panic!("Expected checkbox"),
    }

    // Verify textbox
    match &popup.elements[3] {
        Element::Textbox {
            textbox,
            id,
            placeholder,
            rows,
            ..
        } => {
            assert_eq!(textbox, "Name");
            assert_eq!(id, "name");
            assert_eq!(placeholder.as_deref(), Some("Enter name"));
            assert_eq!(*rows, None);
        }
        _ => panic!("Expected textbox"),
    }

    // Verify multiselect
    match &popup.elements[4] {
        Element::Multiselect {
            multiselect,
            id,
            options,
            ..
        } => {
            assert_eq!(multiselect, "Features");
            assert_eq!(id, "features");
            assert_eq!(options.len(), 3);
        }
        _ => panic!("Expected multiselect"),
    }

    // Verify choice
    match &popup.elements[5] {
        Element::Choice {
            choice,
            id,
            options,
            default,
            ..
        } => {
            assert_eq!(choice, "Mode");
            assert_eq!(id, "mode");
            assert_eq!(options.len(), 3);
            assert_eq!(*default, None); // No default
        }
        _ => panic!("Expected choice"),
    }
}

#[test]
fn test_simple_when_clause() {
    let json = r#"{
        "title": "When Clause Test",
        "elements": [
            {"checkbox": "Advanced", "id": "advanced", "default": false},
            {"slider": "Level", "id": "level", "min": 0, "max": 10, "when": "@advanced"}
        ]
    }"#;

    let popup = parse_popup_json(json).unwrap();
    assert_eq!(popup.elements.len(), 2);

    // Verify checkbox
    match &popup.elements[0] {
        Element::Checkbox { checkbox, id, .. } => {
            assert_eq!(checkbox, "Advanced");
            assert_eq!(id, "advanced");
        }
        _ => panic!("Expected checkbox"),
    }

    // Verify slider with when clause
    match &popup.elements[1] {
        Element::Slider {
            slider, id, when, ..
        } => {
            assert_eq!(slider, "Level");
            assert_eq!(id, "level");
            assert_eq!(when.as_deref(), Some("@advanced"));
        }
        _ => panic!("Expected slider"),
    }
}

#[test]
fn test_complex_when_clauses() {
    let json = r#"{
        "title": "Complex When Clauses",
        "elements": [
            {"multiselect": "Items", "id": "items", "options": ["A", "B", "C", "D", "E", "F"]},
            {"text": "More than 5 items", "id": "many_items", "when": "count(@items) > 5"}
        ]
    }"#;

    let popup = parse_popup_json(json).unwrap();
    assert_eq!(popup.elements.len(), 2);

    // Verify text with count-based when clause
    match &popup.elements[1] {
        Element::Text { text, id, when, .. } => {
            assert_eq!(text, "More than 5 items");
            assert_eq!(id.as_deref(), Some("many_items"));
            assert_eq!(when.as_deref(), Some("count(@items) > 5"));
        }
        _ => panic!("Expected text element"),
    }
}

#[test]
fn test_nested_groups() {
    let json = r#"{
        "title": "Groups",
        "elements": [
            {
                "group": "Settings",
                "id": "settings_group",
                "elements": [
                    {"checkbox": "Option1", "id": "opt1", "default": true},
                    {"checkbox": "Option2", "id": "opt2", "default": false}
                ]
            }
        ]
    }"#;

    let popup = parse_popup_json(json).unwrap();
    assert_eq!(popup.elements.len(), 1);

    match &popup.elements[0] {
        Element::Group {
            group, elements, ..
        } => {
            assert_eq!(group, "Settings");
            assert_eq!(elements.len(), 2);
        }
        _ => panic!("Expected group"),
    }
}

#[test]
fn test_multiline_textbox() {
    let json = r#"{
        "title": "Multiline",
        "elements": [
            {"textbox": "Comments", "id": "comments", "placeholder": "Enter comments", "rows": 5}
        ]
    }"#;

    let popup = parse_popup_json(json).unwrap();

    match &popup.elements[0] {
        Element::Textbox {
            textbox,
            id,
            placeholder,
            rows,
            ..
        } => {
            assert_eq!(textbox, "Comments");
            assert_eq!(id, "comments");
            assert_eq!(placeholder.as_deref(), Some("Enter comments"));
            assert_eq!(*rows, Some(5));
        }
        _ => panic!("Expected textbox"),
    }
}

#[test]
fn test_slider_without_default() {
    let json = r#"{
        "title": "Slider Test",
        "elements": [
            {"slider": "Progress", "id": "progress", "min": 0, "max": 100}
        ]
    }"#;

    let popup = parse_popup_json(json).unwrap();

    match &popup.elements[0] {
        Element::Slider {
            slider,
            id,
            min,
            max,
            default,
            ..
        } => {
            assert_eq!(slider, "Progress");
            assert_eq!(id, "progress");
            assert_eq!(*min, 0.0);
            assert_eq!(*max, 100.0);
            assert_eq!(*default, None); // No default provided
        }
        _ => panic!("Expected slider"),
    }
}

#[test]
fn test_empty_elements() {
    let json = r#"{
        "title": "Empty",
        "elements": []
    }"#;

    let popup = parse_popup_json(json).unwrap();
    assert_eq!(popup.title, Some("Empty".to_string()));
    assert_eq!(popup.elements.len(), 0);
}

#[test]
fn test_invalid_json() {
    let json = r#"{
        "title": "Invalid",
        "elements": [
            {"unknown_element": "Test", "id": "test"}
        ]
    }"#;

    // Should fail because no valid element key is present
    assert!(parse_popup_json(json).is_err());
}

#[test]
fn test_missing_required_fields() {
    let json = r#"{
        "elements": []
    }"#;

    // Should succeed because title is now optional
    let popup = parse_popup_json(json).unwrap();
    assert_eq!(popup.title, None);
    assert_eq!(popup.effective_title(), "Dialog");

    let json = r#"{
        "title": "No Elements"
    }"#;

    // Should fail because elements is still required
    assert!(parse_popup_json(json).is_err());
}

#[test]
fn test_choice_no_default() {
    let json = r#"{
        "title": "Choice Test",
        "elements": [
            {"choice": "Theme", "id": "theme", "options": ["Light", "Dark", "Auto"]}
        ]
    }"#;

    let popup = parse_popup_json(json).unwrap();
    assert_eq!(popup.elements.len(), 1);

    match &popup.elements[0] {
        Element::Choice {
            choice,
            id,
            options,
            default,
            ..
        } => {
            assert_eq!(choice, "Theme");
            assert_eq!(id, "theme");
            assert_eq!(options.len(), 3);
            assert_eq!(options[0].value(), "Light");
            assert_eq!(options[1].value(), "Dark");
            assert_eq!(options[2].value(), "Auto");
            assert_eq!(*default, None); // No default, should be None
        }
        _ => panic!("Expected choice element"),
    }
}

#[test]
fn test_choice_with_default() {
    let json = r#"{
        "title": "Choice Test",
        "elements": [
            {"choice": "Mode", "id": "mode", "options": ["Easy", "Medium", "Hard"], "default": 1}
        ]
    }"#;

    let popup = parse_popup_json(json).unwrap();
    assert_eq!(popup.elements.len(), 1);

    match &popup.elements[0] {
        Element::Choice {
            choice,
            id,
            options,
            default,
            ..
        } => {
            assert_eq!(choice, "Mode");
            assert_eq!(id, "mode");
            assert_eq!(options.len(), 3);
            assert_eq!(options[0].value(), "Easy");
            assert_eq!(options[1].value(), "Medium");
            assert_eq!(options[2].value(), "Hard");
            assert_eq!(*default, Some(1)); // Default to index 1 (Medium)
        }
        _ => panic!("Expected choice element"),
    }
}

#[test]
fn test_choice_state_initialization() {
    use popup_common::{ElementValue, PopupState};

    let json = r#"{
        "title": "Choice State Test",
        "elements": [
            {"choice": "NoDefault", "id": "no_default", "options": ["A", "B"]},
            {"choice": "WithDefault", "id": "with_default", "options": ["X", "Y", "Z"], "default": 2}
        ]
    }"#;

    let popup = parse_popup_json(json).unwrap();
    let state = PopupState::new(&popup);

    // Check no_default initializes to None
    match state.values.get("no_default") {
        Some(ElementValue::Choice(None)) => {}
        _ => panic!("Expected Choice(None) for no_default"),
    }

    // Check with_default initializes to Some(2)
    match state.values.get("with_default") {
        Some(ElementValue::Choice(Some(2))) => {}
        _ => panic!("Expected Choice(Some(2)) for with_default"),
    }
}
