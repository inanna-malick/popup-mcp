use crate::models::PopupDefinition;
use anyhow::Result;
use serde_json::Value;

/// Parse JSON input into a PopupDefinition
pub fn parse_popup_json(input: &str) -> Result<PopupDefinition> {
    let popup: PopupDefinition = serde_json::from_str(input)?;
    Ok(popup)
}

/// Parse JSON Value into a PopupDefinition
pub fn parse_popup_json_value(value: Value) -> Result<PopupDefinition> {
    let popup: PopupDefinition = serde_json::from_value(value)?;
    Ok(popup)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Condition, Element};

    #[test]
    fn test_simple_popup() {
        let json = r#"{
            "title": "Confirm Delete",
            "elements": [
                {"type": "text", "content": "Are you sure?"}
            ]
        }"#;

        let popup = parse_popup_json(json).unwrap();
        assert_eq!(popup.title, "Confirm Delete");
        assert_eq!(popup.elements.len(), 1);

        match &popup.elements[0] {
            Element::Text { content } => assert_eq!(content, "Are you sure?"),
            _ => panic!("Expected text element"),
        }
    }

    #[test]
    fn test_widgets_popup() {
        let json = r#"{
            "title": "Settings",
            "elements": [
                {"type": "slider", "label": "Volume", "min": 0, "max": 100, "default": 75},
                {"type": "checkbox", "label": "Notifications", "default": true},
                {"type": "choice", "label": "Theme", "options": ["Light", "Dark", "Auto"]},
                {"type": "textbox", "label": "Name", "placeholder": "Enter your name"}
            ]
        }"#;

        let popup = parse_popup_json(json).unwrap();
        assert_eq!(popup.elements.len(), 4);

        match &popup.elements[0] {
            Element::Slider {
                label,
                min,
                max,
                default,
            } => {
                assert_eq!(label, "Volume");
                assert_eq!(*min, 0.0);
                assert_eq!(*max, 100.0);
                assert_eq!(*default, Some(75.0));
            }
            _ => panic!("Expected slider element"),
        }
    }

    #[test]
    fn test_conditional_popup() {
        let json = r#"{
            "title": "Advanced",
            "elements": [
                {"type": "checkbox", "label": "Show advanced", "default": false},
                {
                    "type": "conditional",
                    "condition": "Show advanced",
                    "elements": [
                        {"type": "slider", "label": "Debug", "min": 0, "max": 10}
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
                    Condition::Simple(label) => assert_eq!(label, "Show advanced"),
                    _ => panic!("Expected Simple condition"),
                }
                assert_eq!(elements.len(), 1);
            }
            _ => panic!("Expected conditional element"),
        }
    }
}
