use anyhow::{anyhow, Result};
use popup_common::{validate_definition, PopupDefinition};

/// Parse JSON input into a PopupDefinition
///
/// This function uses standard Serde deserialization to ensure the input matches
/// the canonical PopupDefinition format: `{"title": "...", "elements": [...]}`
pub fn parse_popup_json(input: &str) -> Result<PopupDefinition> {
    let popup: PopupDefinition = serde_json::from_str(input)?;
    Ok(popup)
}

/// Parse, apply automatic transforms ("Other" injection), and structurally
/// validate popup JSON: duplicate/reserved ids, broken when clauses,
/// out-of-range defaults, etc.
///
/// Validation runs on the post-injection definition - the one that will
/// actually render - so collisions with injected "<id>_other_text" inputs
/// are caught too. Returns the render-ready definition.
pub fn parse_and_validate_popup_json(input: &str) -> Result<PopupDefinition> {
    let popup = crate::transform::inject_other_options(parse_popup_json(input)?);
    let issues = validate_definition(&popup);
    if issues.is_empty() {
        Ok(popup)
    } else {
        let summary = issues
            .iter()
            .map(|i| format!("- {}", i))
            .collect::<Vec<_>>()
            .join("\n");
        Err(anyhow!(
            "Popup definition has {} problem(s):\n{}",
            issues.len(),
            summary
        ))
    }
}

/// Validate popup JSON without using the parsed PopupDefinition
pub fn validate_popup_json(input: &str) -> Result<()> {
    parse_and_validate_popup_json(input).map(|_| ())
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
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("missing field `title`"));
    }

    #[test]
    fn test_missing_elements() {
        let json = r#"{
            "title": "Missing Elements"
        }"#;

        let result = parse_popup_json(json);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("missing field `elements`"));
    }
}
