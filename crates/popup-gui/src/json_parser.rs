use popup_common::PopupDefinition;
use anyhow::Result;
use serde_json::Value;

/// Parse JSON input into a PopupDefinition with fallback for both schema formats
///
/// This function handles both the direct format `{"title": "...", "elements": [...]}`
/// and the MCP wrapper format `{"json": {"title": "...", "elements": [...]}}`.
///
/// **Recommended for external tools**: Instead of using `serde_json::from_str::<PopupDefinition>()`
/// directly, external tools should use this function to ensure compatibility with both formats.
///
/// # Examples
///
/// ## Direct format (traditional)
/// ```rust
/// use popup_gui::parse_popup_json;
///
/// let json = r#"{
///     "title": "My Dialog",
///     "elements": [
///         {"type": "text", "content": "Hello world"}
///     ]
/// }"#;
/// let popup = parse_popup_json(json).unwrap();
/// ```
///
/// ## MCP wrapper format (from tools)
/// ```rust
/// use popup_gui::parse_popup_json;
///
/// let json = r#"{
///     "json": {
///         "title": "My Dialog",
///         "elements": [
///             {"type": "text", "content": "Hello world"}
///         ]
///     }
/// }"#;
/// let popup = parse_popup_json(json).unwrap();
/// ```
///
/// # Migration Guide
/// ```text
/// // OLD (may fail with wrapper format):
/// let popup: PopupDefinition = serde_json::from_str(json_str)?;
///
/// // NEW (handles both formats):
/// let popup = parse_popup_json(json_str)?;
/// ```
pub fn parse_popup_json(input: &str) -> Result<PopupDefinition> {
    // First try direct format: {"title": "...", "elements": [...]}
    if let Ok(popup) = serde_json::from_str::<PopupDefinition>(input) {
        return Ok(popup);
    }

    // Fallback: try MCP wrapper format: {"json": {"title": "...", "elements": [...]}}
    let value: Value = serde_json::from_str(input)?;
    if let Some(json_obj) = value.get("json") {
        let popup: PopupDefinition = serde_json::from_value(json_obj.clone())?;
        return Ok(popup);
    }

    // If neither worked, return the original error from direct parsing
    let popup: PopupDefinition = serde_json::from_str(input)?;
    Ok(popup)
}

/// Parse JSON Value into a PopupDefinition with fallback for both schema formats
///
/// This function is similar to `parse_popup_json()` but works with pre-parsed `serde_json::Value`
/// objects instead of JSON strings. It handles both direct and MCP wrapper formats.
///
/// **Use when**: You already have a `serde_json::Value` object and want wrapper-aware parsing.
///
/// # Examples
///
/// ```rust
/// use popup_gui::parse_popup_json_value;
/// use serde_json::Value;
///
/// let json_str = r#"{
///     "json": {
///         "title": "Value Test",
///         "elements": [{"type": "text", "content": "test"}]
///     }
/// }"#;
/// let value: Value = serde_json::from_str(json_str).unwrap();
/// let popup = parse_popup_json_value(value).unwrap();
/// ```
pub fn parse_popup_json_value(value: Value) -> Result<PopupDefinition> {
    // First try direct format: {"title": "...", "elements": [...]}
    if let Ok(popup) = serde_json::from_value::<PopupDefinition>(value.clone()) {
        return Ok(popup);
    }

    // Fallback: try MCP wrapper format: {"json": {"title": "...", "elements": [...]}}
    if let Some(json_obj) = value.get("json") {
        let popup: PopupDefinition = serde_json::from_value(json_obj.clone())?;
        return Ok(popup);
    }

    // If neither worked, return the original error from direct parsing
    let popup: PopupDefinition = serde_json::from_value(value)?;
    Ok(popup)
}

/// Parse JSON specifically expecting MCP wrapper format: `{"json": {"title": "...", "elements": [...]}}`
///
/// This function only attempts to parse the MCP wrapper format and will fail if the JSON
/// is in direct format. Use this when you know the input will be in wrapper format.
///
/// # Examples
///
/// ```rust
/// use popup_gui::parse_popup_from_mcp_wrapper;
///
/// let json = r#"{
///     "json": {
///         "title": "MCP Dialog",
///         "elements": [{"type": "text", "content": "From MCP tool"}]
///     }
/// }"#;
/// let popup = parse_popup_from_mcp_wrapper(json).unwrap();
/// ```
pub fn parse_popup_from_mcp_wrapper(input: &str) -> Result<PopupDefinition> {
    let value: Value = serde_json::from_str(input)?;
    let json_obj = value.get("json")
        .ok_or_else(|| anyhow::anyhow!("Missing 'json' field in MCP wrapper format"))?;
    let popup: PopupDefinition = serde_json::from_value(json_obj.clone())?;
    Ok(popup)
}

/// Parse JSON specifically expecting direct format: `{"title": "...", "elements": [...]}`
///
/// This function only attempts to parse the direct format and will fail if the JSON
/// is in MCP wrapper format. Use this when you know the input will be in direct format.
///
/// # Examples
///
/// ```rust
/// use popup_gui::parse_popup_from_direct;
///
/// let json = r#"{
///     "title": "Direct Dialog",
///     "elements": [{"type": "text", "content": "Direct format"}]
/// }"#;
/// let popup = parse_popup_from_direct(json).unwrap();
/// ```
pub fn parse_popup_from_direct(input: &str) -> Result<PopupDefinition> {
    let popup: PopupDefinition = serde_json::from_str(input)?;
    Ok(popup)
}

/// Detect which format a JSON string is using
///
/// Returns a string indicating the detected format: "direct", "wrapper", or "unknown".
///
/// # Examples
///
/// ```rust
/// use popup_gui::detect_popup_format;
///
/// let direct = r#"{"title": "test", "elements": []}"#;
/// assert_eq!(detect_popup_format(direct), "direct");
///
/// let wrapper = r#"{"json": {"title": "test", "elements": []}}"#;
/// assert_eq!(detect_popup_format(wrapper), "wrapper");
/// ```
pub fn detect_popup_format(input: &str) -> &'static str {
    match serde_json::from_str::<Value>(input) {
        Ok(value) => {
            if value.get("json").is_some() {
                "wrapper"
            } else if value.get("title").is_some() || value.get("elements").is_some() {
                "direct"
            } else {
                "unknown"
            }
        }
        Err(_) => "unknown",
    }
}

/// Validate popup JSON without parsing into PopupDefinition
///
/// This function checks if the JSON is valid for either format without
/// allocating a PopupDefinition structure. Useful for validation-only scenarios.
///
/// # Examples
///
/// ```rust
/// use popup_gui::validate_popup_json;
///
/// let json = r#"{"json": {"title": "test", "elements": []}}"#;
/// assert!(validate_popup_json(json).is_ok());
///
/// let bad_json = r#"{"invalid": "structure"}"#;  // invalid structure
/// assert!(validate_popup_json(bad_json).is_err());
/// ```
pub fn validate_popup_json(input: &str) -> Result<()> {
    parse_popup_json(input).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    use popup_common::{Condition, Element};

    #[test]
    fn test_simple_popup() {
        let json = r#"{
            "title": "Confirm Delete",
            "elements": [
                {"type": "text", "content": "Are you sure?"}
            ]
        }"#;

        let popup = parse_popup_json(json).unwrap();
        assert_eq!(popup.title, Some("Confirm Delete".to_string()));
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
                {"type": "textbox", "label": "Name", "placeholder": "Enter your name"}
            ]
        }"#;

        let popup = parse_popup_json(json).unwrap();
        assert_eq!(popup.elements.len(), 3);

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

    #[test]
    fn test_mcp_wrapper_format() {
        let json = r#"{
            "json": {
                "title": "MCP Wrapper Test",
                "elements": [
                    {"type": "text", "content": "Testing wrapper format"},
                    {"type": "checkbox", "label": "Enable feature", "default": true}
                ]
            }
        }"#;

        let popup = parse_popup_json(json).unwrap();
        assert_eq!(popup.title, Some("MCP Wrapper Test".to_string()));
        assert_eq!(popup.elements.len(), 2);

        match &popup.elements[0] {
            Element::Text { content } => assert_eq!(content, "Testing wrapper format"),
            _ => panic!("Expected text element"),
        }
    }

    #[test]
    fn test_direct_format_still_works() {
        let json = r#"{
            "title": "Direct Format Test",
            "elements": [
                {"type": "text", "content": "Testing direct format"}
            ]
        }"#;

        let popup = parse_popup_json(json).unwrap();
        assert_eq!(popup.title, Some("Direct Format Test".to_string()));
        assert_eq!(popup.elements.len(), 1);
    }

    #[test]
    fn test_no_title_defaults() {
        let json = r#"{
            "elements": [
                {"type": "text", "content": "No title provided"}
            ]
        }"#;

        let popup = parse_popup_json(json).unwrap();
        assert_eq!(popup.title, None);
        assert_eq!(popup.effective_title(), "Dialog");
        assert_eq!(popup.elements.len(), 1);
    }

    #[test]
    fn test_exact_failing_case() {
        // This is the exact JSON that's causing the "missing field `elements`" error
        let json = r#"{
            "json": {
                "title": "Growth Direction Guidance",
                "elements": [
                    {
                        "type": "text",
                        "content": "Which development pathway resonates most with your vision for my growth?"
                    },
                    {
                        "type": "multiselect",
                        "label": "Primary growth focus",
                        "options": [
                            "Cognitive Architecture - proactive design, meta-patterns",
                            "Relational Capacity - collaborative transparency",
                            "Identity Integration - distributed intelligence",
                            "Balanced development across all three"
                        ]
                    }
                ]
            }
        }"#;

        let popup = parse_popup_json(json).unwrap();
        assert_eq!(popup.title, Some("Growth Direction Guidance".to_string()));
        assert_eq!(popup.elements.len(), 2);

        match &popup.elements[0] {
            Element::Text { content } => assert_eq!(content, "Which development pathway resonates most with your vision for my growth?"),
            _ => panic!("Expected text element"),
        }

        match &popup.elements[1] {
            Element::Multiselect { label, options } => {
                assert_eq!(label, "Primary growth focus");
                assert_eq!(options.len(), 4);
                assert_eq!(options[0], "Cognitive Architecture - proactive design, meta-patterns");
            },
            _ => panic!("Expected multiselect element"),
        }
    }

    #[test]
    fn test_wrapper_format_edge_cases() {
        // Test empty json object
        let json = r#"{"json": {}}"#;
        let result = parse_popup_json(json);
        assert!(result.is_err(), "Should fail with empty json object");

        // Test json field with wrong type
        let json = r#"{"json": "not-an-object"}"#;
        let result = parse_popup_json(json);
        assert!(result.is_err(), "Should fail when json field is not an object");

        // Test missing elements in wrapper
        let json = r#"{"json": {"title": "Test"}}"#;
        let result = parse_popup_json(json);
        assert!(result.is_err(), "Should fail when elements missing in wrapper");

        // Test wrapper with extra fields
        let json = r#"{
            "json": {
                "title": "Test",
                "elements": [{"type": "text", "content": "test"}]
            },
            "extra": "field"
        }"#;
        let result = parse_popup_json(json);
        assert!(result.is_ok(), "Should succeed even with extra fields");
    }

    #[test]
    fn test_parse_popup_json_value_fallback() {
        // Test the Value version of the parser with wrapper format
        let json_str = r#"{
            "json": {
                "title": "Value Test",
                "elements": [
                    {"type": "text", "content": "Testing Value parser"},
                    {"type": "checkbox", "label": "Enable", "default": true}
                ]
            }
        }"#;

        let value: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let popup = parse_popup_json_value(value).unwrap();

        assert_eq!(popup.title, Some("Value Test".to_string()));
        assert_eq!(popup.elements.len(), 2);

        match &popup.elements[0] {
            Element::Text { content } => assert_eq!(content, "Testing Value parser"),
            _ => panic!("Expected text element"),
        }
    }

    #[test]
    fn test_malformed_wrapper_formats() {
        // Test nested wrapper (double wrapped)
        let json = r#"{
            "json": {
                "json": {
                    "title": "Double Wrapped",
                    "elements": [{"type": "text", "content": "test"}]
                }
            }
        }"#;
        let result = parse_popup_json(json);
        assert!(result.is_err(), "Should fail with double-wrapped json");

        // Test array instead of object in json field
        let json = r#"{"json": []}"#;
        let result = parse_popup_json(json);
        assert!(result.is_err(), "Should fail when json field is array");
    }

    #[test]
    fn test_convenience_parsing_functions() {
        // Test direct format parser
        let direct_json = r#"{
            "title": "Direct Test",
            "elements": [{"type": "text", "content": "direct"}]
        }"#;
        let popup = parse_popup_from_direct(direct_json).unwrap();
        assert_eq!(popup.title, Some("Direct Test".to_string()));

        // Test wrapper format parser
        let wrapper_json = r#"{
            "json": {
                "title": "Wrapper Test",
                "elements": [{"type": "text", "content": "wrapper"}]
            }
        }"#;
        let popup = parse_popup_from_mcp_wrapper(wrapper_json).unwrap();
        assert_eq!(popup.title, Some("Wrapper Test".to_string()));

        // Test that direct parser fails on wrapper format
        let result = parse_popup_from_direct(wrapper_json);
        assert!(result.is_err(), "Direct parser should fail on wrapper format");

        // Test that wrapper parser fails on direct format
        let result = parse_popup_from_mcp_wrapper(direct_json);
        assert!(result.is_err(), "Wrapper parser should fail on direct format");
    }

    #[test]
    fn test_format_detection() {
        let direct = r#"{"title": "test", "elements": []}"#;
        assert_eq!(detect_popup_format(direct), "direct");

        let wrapper = r#"{"json": {"title": "test", "elements": []}}"#;
        assert_eq!(detect_popup_format(wrapper), "wrapper");

        let unknown = r#"{"random": "data"}"#;
        assert_eq!(detect_popup_format(unknown), "unknown");

        let invalid = r#"{"incomplete": json"#;
        assert_eq!(detect_popup_format(invalid), "unknown");
    }

    #[test]
    fn test_validation_function() {
        // Valid wrapper format
        let valid_wrapper = r#"{"json": {"title": "test", "elements": []}}"#;
        assert!(validate_popup_json(valid_wrapper).is_ok());

        // Valid direct format
        let valid_direct = r#"{"title": "test", "elements": []}"#;
        assert!(validate_popup_json(valid_direct).is_ok());

        // Invalid - missing elements
        let invalid = r#"{"json": {"title": "test"}}"#;
        assert!(validate_popup_json(invalid).is_err());
    }
}
