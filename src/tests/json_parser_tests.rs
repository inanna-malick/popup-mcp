use crate::json_parser::parse_popup_json;
use crate::models::{Condition, Element};

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
            {"type": "multiselect", "label": "Features", "options": ["A", "B", "C"]}
        ]
    }"#;

    let popup = parse_popup_json(json).unwrap();
    assert_eq!(popup.elements.len(), 5);

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
        Element::Checkbox { label, default } => {
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
                "condition": {"checked": "Debug Mode"},
                "elements": [
                    {"type": "text", "content": "Debug mode active"}
                ]
            },
            {
                "type": "conditional",
                "condition": {"count": "Items", "op": ">", "value": 5},
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
            Condition::Checked { checked } => {
                assert_eq!(checked, "Debug Mode");
            }
            _ => panic!("Expected Checked condition"),
        },
        _ => panic!("Expected conditional"),
    }

    // Check second conditional
    match &popup.elements[1] {
        Element::Conditional { condition, .. } => match condition {
            Condition::Count { count, op, value } => {
                assert_eq!(count, "Items");
                assert_eq!(*op, crate::models::ComparisonOp::Greater);
                assert_eq!(*value, 5);
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
