use crate::json_parser::parse_popup_json;
use popup_common::{Condition, Element};

#[test]
fn test_simple_confirmation() {
    let json = r#"{
        "title": "Confirm",
        "elements": [
            {"type": "text", "content": "Are you sure?"}
        ]
    }"#;

    let popup = parse_popup_json(json).unwrap();
    assert_eq!(popup.title, Some("Confirm".to_string()));
    assert_eq!(popup.elements.len(), 1);

    match &popup.elements[0] {
        Element::Text { content } => assert_eq!(content, "Are you sure?"),
        _ => panic!("Expected text element"),
    }
}

#[test]
fn test_all_widget_types() {
    let json = r#"{
        "title": "All Widgets",
        "elements": [
            {"type": "text", "content": "Test all widget types"},
            {"type": "slider", "label": "Volume", "min": 0, "max": 100, "default": 50},
            {"type": "checkbox", "label": "Enable", "default": true},
            {"type": "textbox", "label": "Name", "placeholder": "Enter name"},
            {"type": "multiselect", "label": "Features", "options": ["A", "B", "C"]},
            {"type": "choice", "label": "Mode", "options": ["X", "Y", "Z"]}
        ]
    }"#;

    let popup = parse_popup_json(json).unwrap();
    assert_eq!(popup.elements.len(), 6);

    // Verify slider
    match &popup.elements[1] {
        Element::Slider {
            label,
            min,
            max,
            default,
        } => {
            assert_eq!(label, "Volume");
            assert_eq!(*min, 0.0);
            assert_eq!(*max, 100.0);
            assert_eq!(*default, Some(50.0));
        }
        _ => panic!("Expected slider"),
    }

    // Verify checkbox
    match &popup.elements[2] {
        Element::Checkbox { label, default, .. } => {
            assert_eq!(label, "Enable");
            assert_eq!(*default, true);
        }
        _ => panic!("Expected checkbox"),
    }

    // Verify textbox
    match &popup.elements[3] {
        Element::Textbox {
            label,
            placeholder,
            rows,
        } => {
            assert_eq!(label, "Name");
            assert_eq!(placeholder.as_deref(), Some("Enter name"));
            assert_eq!(*rows, None);
        }
        _ => panic!("Expected textbox"),
    }

    // Verify multiselect
    match &popup.elements[4] {
        Element::Multiselect { label, options } => {
            assert_eq!(label, "Features");
            assert_eq!(options.len(), 3);
        }
        _ => panic!("Expected multiselect"),
    }

    // Verify choice
    match &popup.elements[5] {
        Element::Choice {
            label,
            options,
            default,
        } => {
            assert_eq!(label, "Mode");
            assert_eq!(options.len(), 3);
            assert_eq!(*default, None); // No default
        }
        _ => panic!("Expected choice"),
    }
}

#[test]
fn test_simple_conditional() {
    let json = r#"{
        "title": "Conditional Test",
        "elements": [
            {"type": "checkbox", "label": "Advanced", "default": false},
            {
                "type": "conditional",
                "condition": "Advanced",
                "elements": [
                    {"type": "slider", "label": "Level", "min": 0, "max": 10}
                ]
            }
        ]
    }"#;

    let popup = parse_popup_json(json).unwrap();
    assert_eq!(popup.elements.len(), 2);

    match &popup.elements[1] {
        Element::Conditional {
            condition,
            elements,
        } => {
            match condition {
                Condition::Simple(label) => assert_eq!(label, "Advanced"),
                _ => panic!("Expected Simple condition"),
            }
            assert_eq!(elements.len(), 1);
        }
        _ => panic!("Expected conditional"),
    }
}

#[test]
fn test_complex_conditional() {
    let json = r#"{
        "title": "Complex Conditional",
        "elements": [
            {
                "type": "conditional",
                "condition": "Debug Mode",
                "elements": [
                    {"type": "text", "content": "Debug mode active"}
                ]
            },
            {
                "type": "conditional",
                "condition": {"field": "Items", "count": ">5"},
                "elements": [
                    {"type": "text", "content": "More than 5 items"}
                ]
            }
        ]
    }"#;

    let popup = parse_popup_json(json).unwrap();
    assert_eq!(popup.elements.len(), 2);

    // Check first conditional
    match &popup.elements[0] {
        Element::Conditional { condition, .. } => match condition {
            Condition::Simple(label) => {
                assert_eq!(label, "Debug Mode");
            }
            _ => panic!("Expected Simple condition"),
        },
        _ => panic!("Expected conditional"),
    }

    // Check second conditional
    match &popup.elements[1] {
        Element::Conditional { condition, .. } => match condition {
            Condition::Count { field, count } => {
                assert_eq!(field, "Items");
                assert_eq!(count, ">5");
            }
            _ => panic!("Expected Count condition"),
        },
        _ => panic!("Expected conditional"),
    }
}

#[test]
fn test_nested_groups() {
    let json = r#"{
        "title": "Groups",
        "elements": [
            {
                "type": "group",
                "label": "Settings",
                "elements": [
                    {"type": "checkbox", "label": "Option1", "default": true},
                    {"type": "checkbox", "label": "Option2", "default": false}
                ]
            }
        ]
    }"#;

    let popup = parse_popup_json(json).unwrap();
    assert_eq!(popup.elements.len(), 1);

    match &popup.elements[0] {
        Element::Group { label, elements } => {
            assert_eq!(label, "Settings");
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
            {"type": "textbox", "label": "Comments", "placeholder": "Enter comments", "rows": 5}
        ]
    }"#;

    let popup = parse_popup_json(json).unwrap();

    match &popup.elements[0] {
        Element::Textbox {
            label,
            placeholder,
            rows,
        } => {
            assert_eq!(label, "Comments");
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
            {"type": "slider", "label": "Progress", "min": 0, "max": 100}
        ]
    }"#;

    let popup = parse_popup_json(json).unwrap();

    match &popup.elements[0] {
        Element::Slider {
            label,
            min,
            max,
            default,
        } => {
            assert_eq!(label, "Progress");
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
            {"type": "unknown", "label": "Test"}
        ]
    }"#;

    // Should fail because "unknown" is not a valid element type
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
            {"type": "choice", "label": "Theme", "options": ["Light", "Dark", "Auto"]}
        ]
    }"#;

    let popup = parse_popup_json(json).unwrap();
    assert_eq!(popup.elements.len(), 1);

    match &popup.elements[0] {
        Element::Choice {
            label,
            options,
            default,
        } => {
            assert_eq!(label, "Theme");
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
            {"type": "choice", "label": "Mode", "options": ["Easy", "Medium", "Hard"], "default": 1}
        ]
    }"#;

    let popup = parse_popup_json(json).unwrap();
    assert_eq!(popup.elements.len(), 1);

    match &popup.elements[0] {
        Element::Choice {
            label,
            options,
            default,
        } => {
            assert_eq!(label, "Mode");
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
            {"type": "choice", "label": "NoDefault", "options": ["A", "B"]},
            {"type": "choice", "label": "WithDefault", "options": ["X", "Y", "Z"], "default": 2}
        ]
    }"#;

    let popup = parse_popup_json(json).unwrap();
    let state = PopupState::new(&popup);

    // Check NoDefault initializes to None
    let key = state.find_key_by_label("NoDefault").unwrap();
    match state.values.get(&key) {
        Some(ElementValue::Choice(None)) => {}
        _ => panic!("Expected Choice(None) for NoDefault"),
    }

    // Check WithDefault initializes to Some(2)
    let key = state.find_key_by_label("WithDefault").unwrap();
    match state.values.get(&key) {
        Some(ElementValue::Choice(Some(2))) => {}
        _ => panic!("Expected Choice(Some(2)) for WithDefault"),
    }
}
