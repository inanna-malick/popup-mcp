use anyhow::Result;
use popup_common::PopupDefinition;

/// Parse JSON input into a PopupDefinition
///
/// This function uses standard Serde deserialization to ensure the input matches
/// the canonical PopupDefinition format: `{"title": "...", "elements": [...]}`
pub fn parse_popup_json(input: &str) -> Result<PopupDefinition> {
    let popup: PopupDefinition = serde_json::from_str(input)?;
    Ok(popup)
}

/// Validate popup JSON without parsing into PopupDefinition
pub fn validate_popup_json(input: &str) -> Result<()> {
    parse_popup_json(input).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    use popup_common::Element;

    #[test]
    fn test_simple_popup() {
        let json = r#"{
            "title": "Confirm Delete",
            "elements": [
                {"text": "Are you sure?", "id": "confirm_text"}
            ]
        }"#;

        let popup = parse_popup_json(json).unwrap();
        assert_eq!(popup.title, "Confirm Delete");
        assert_eq!(popup.elements.len(), 1);

        match &popup.elements[0] {
            Element::Text { text, .. } => assert_eq!(text, "Are you sure?"),
            _ => panic!("Expected text element"),
        }
    }

    #[test]
    fn test_missing_title() {
        let json = r#"{
            "elements": [
                {"text": "No title provided"}
            ]
        }"#;

        let result = parse_popup_json(json);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("missing field `title`"));
    }

    #[test]
    fn test_missing_elements() {
        let json = r#"{
            "title": "Missing Elements"
        }"#;

        let result = parse_popup_json(json);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("missing field `elements`"));
    }
}