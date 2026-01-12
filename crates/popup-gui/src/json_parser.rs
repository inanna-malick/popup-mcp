use anyhow::Result;
use popup_common::{Element, PopupDefinition};
use serde_json::Value;

/// Element type discriminator keys used for single-element root detection
const ELEMENT_KEYS: &[&str] = &[
    "text", "markdown", "slider", "check", "input", "select", "multi", "group",
];

/// Check if a JSON object looks like a single element (has an element type key)
fn is_element_object(obj: &serde_json::Map<String, Value>) -> bool {
    ELEMENT_KEYS.iter().any(|key| obj.contains_key(*key))
}

/// Parse JSON input into a PopupDefinition with support for multiple formats
///
/// This function handles:
/// 1. **Direct format**: `{"title": "...", "elements": [...]}`
/// 2. **MCP wrapper format**: `{"json": {"title": "...", "elements": [...]}}`
/// 3. **Single-element root**: `{"text": "Hello"}` → wraps in PopupDefinition
/// 4. **Array root**: `[{"text": "Hello"}, {"check": "OK"}]` → wraps in PopupDefinition
///
/// **Recommended for external tools**: Instead of using `serde_json::from_str::<PopupDefinition>()`
/// directly, external tools should use this function to ensure compatibility with all formats.
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
///         {"text": "Hello world", "id": "greeting"}
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
///             {"text": "Hello world", "id": "greeting"}
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
/// // NEW (handles all formats):
/// let popup = parse_popup_json(json_str)?;
/// ```
pub fn parse_popup_json(input: &str) -> Result<PopupDefinition> {
    let value: Value = serde_json::from_str(input)?;
    parse_popup_json_value(value)
}

/// Parse JSON Value into a PopupDefinition with support for multiple formats
///
/// This function is similar to `parse_popup_json()` but works with pre-parsed `serde_json::Value`
/// objects instead of JSON strings. It handles all supported formats:
/// 1. Direct format: `{"title": "...", "elements": [...]}`
/// 2. MCP wrapper format: `{"json": {...}}`
/// 3. Single-element root: `{"text": "Hello"}` → wraps in PopupDefinition
/// 4. Array root: `[{...}, {...}]` → wraps in PopupDefinition
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
///         "elements": [{"text": "test", "id": "test_text"}]
///     }
/// }"#;
/// let value: Value = serde_json::from_str(json_str).unwrap();
/// let popup = parse_popup_json_value(value).unwrap();
/// ```
pub fn parse_popup_json_value(value: Value) -> Result<PopupDefinition> {
    // Handle array root: [...] → wrap elements in PopupDefinition
    if let Value::Array(arr) = &value {
        let elements: Vec<Element> = arr
            .iter()
            .map(|v| serde_json::from_value(v.clone()))
            .collect::<Result<Vec<_>, _>>()?;
        return Ok(PopupDefinition {
            title: None,
            elements,
        });
    }

    // Handle object types
    if let Value::Object(obj) = &value {
        // Check for MCP wrapper format first: {"json": {...}}
        if let Some(json_obj) = obj.get("json") {
            // Recursively parse the inner json object
            return parse_popup_json_value(json_obj.clone());
        }

        // Check for single-element root: {"text": "..."} or {"slider": "...", ...}
        if is_element_object(obj) {
            let element: Element = serde_json::from_value(value)?;
            return Ok(PopupDefinition {
                title: None,
                elements: vec![element],
            });
        }

        // Otherwise, try standard PopupDefinition format: {"title": "...", "elements": [...]}
        let popup: PopupDefinition = serde_json::from_value(value)?;
        return Ok(popup);
    }

    // Neither object nor array - return error
    Err(anyhow::anyhow!(
        "Invalid popup JSON: expected object or array, got {:?}",
        value
    ))
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
///         "elements": [{"text": "From MCP tool", "id": "mcp_text"}]
///     }
/// }"#;
/// let popup = parse_popup_from_mcp_wrapper(json).unwrap();
/// ```
pub fn parse_popup_from_mcp_wrapper(input: &str) -> Result<PopupDefinition> {
    let value: Value = serde_json::from_str(input)?;
    let json_obj = value
        .get("json")
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
///     "elements": [{"text": "Direct format", "id": "direct_text"}]
/// }"#;
/// let popup = parse_popup_from_direct(json).unwrap();
/// ```
pub fn parse_popup_from_direct(input: &str) -> Result<PopupDefinition> {
    let popup: PopupDefinition = serde_json::from_str(input)?;
    Ok(popup)
}

/// Detect which format a JSON string is using
///
/// Returns a string indicating the detected format:
/// - "direct" - standard PopupDefinition format
/// - "wrapper" - MCP wrapper format with "json" key
/// - "element" - single element at root
/// - "array" - array of elements at root
/// - "unknown" - unrecognized format
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
        Ok(Value::Array(_)) => "array",
        Ok(Value::Object(obj)) => {
            if obj.get("json").is_some() {
                "wrapper"
            } else if obj.get("title").is_some() || obj.get("elements").is_some() {
                "direct"
            } else if is_element_object(&obj) {
                "element"
            } else {
                "unknown"
            }
        }
        _ => "unknown",
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
        assert_eq!(popup.title, Some("Confirm Delete".to_string()));
        assert_eq!(popup.elements.len(), 1);

        match &popup.elements[0] {
            Element::Text { text, .. } => assert_eq!(text, "Are you sure?"),
            _ => panic!("Expected text element"),
        }
    }

    #[test]
    fn test_widgets_popup() {
        let json = r#"{
            "title": "Settings",
            "elements": [
                {"slider": "Volume", "id": "volume", "min": 0, "max": 100, "default": 75},
                {"check": "Notifications", "id": "notifications", "default": true},
                {"input": "Name", "id": "name", "placeholder": "Enter your name"}
            ]
        }"#;

        let popup = parse_popup_json(json).unwrap();
        assert_eq!(popup.elements.len(), 3);

        match &popup.elements[0] {
            Element::Slider {
                slider,
                min,
                max,
                default,
                ..
            } => {
                assert_eq!(slider, "Volume");
                assert_eq!(*min, 0.0);
                assert_eq!(*max, 100.0);
                assert_eq!(*default, Some(75.0));
            }
            _ => panic!("Expected slider element"),
        }
    }

    #[test]
    fn test_when_clause_popup() {
        let json = r#"{
            "title": "Advanced",
            "elements": [
                {"check": "Show advanced", "id": "show_advanced", "default": false},
                {"slider": "Debug", "id": "debug", "min": 0, "max": 10, "when": "@show_advanced"}
            ]
        }"#;

        let popup = parse_popup_json(json).unwrap();
        assert_eq!(popup.elements.len(), 2);

        match &popup.elements[1] {
            Element::Slider { slider, when, .. } => {
                assert_eq!(slider, "Debug");
                assert_eq!(when.as_deref(), Some("@show_advanced"));
            }
            _ => panic!("Expected slider element"),
        }
    }

    #[test]
    fn test_mcp_wrapper_format() {
        let json = r#"{
            "json": {
                "title": "MCP Wrapper Test",
                "elements": [
                    {"text": "Testing wrapper format", "id": "wrapper_text"},
                    {"check": "Enable feature", "id": "enable_feature", "default": true}
                ]
            }
        }"#;

        let popup = parse_popup_json(json).unwrap();
        assert_eq!(popup.title, Some("MCP Wrapper Test".to_string()));
        assert_eq!(popup.elements.len(), 2);

        match &popup.elements[0] {
            Element::Text { text, .. } => assert_eq!(text, "Testing wrapper format"),
            _ => panic!("Expected text element"),
        }
    }

    #[test]
    fn test_direct_format_still_works() {
        let json = r#"{
            "title": "Direct Format Test",
            "elements": [
                {"text": "Testing direct format", "id": "direct_text"}
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
                {"text": "No title provided", "id": "no_title_text"}
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
                        "text": "Which development pathway resonates most with your vision for my growth?",
                        "id": "guidance_text"
                    },
                    {
                        "multi": "Primary growth focus",
                        "id": "growth_focus",
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
            Element::Text { text, .. } => assert_eq!(
                text,
                "Which development pathway resonates most with your vision for my growth?"
            ),
            _ => panic!("Expected text element"),
        }

        match &popup.elements[1] {
            Element::Multi { multi, options, .. } => {
                assert_eq!(multi, "Primary growth focus");
                assert_eq!(options.len(), 4);
                assert_eq!(
                    options[0].value(),
                    "Cognitive Architecture - proactive design, meta-patterns"
                );
            }
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
        assert!(
            result.is_err(),
            "Should fail when json field is not an object"
        );

        // Test missing elements in wrapper
        let json = r#"{"json": {"title": "Test"}}"#;
        let result = parse_popup_json(json);
        assert!(
            result.is_err(),
            "Should fail when elements missing in wrapper"
        );

        // Test wrapper with extra fields
        let json = r#"{
            "json": {
                "title": "Test",
                "elements": [{"text": "test", "id": "test_text"}]
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
                    {"text": "Testing Value parser", "id": "test_text"},
                    {"check": "Enable", "id": "enable", "default": true}
                ]
            }
        }"#;

        let value: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let popup = parse_popup_json_value(value).unwrap();

        assert_eq!(popup.title, Some("Value Test".to_string()));
        assert_eq!(popup.elements.len(), 2);

        match &popup.elements[0] {
            Element::Text { text, .. } => assert_eq!(text, "Testing Value parser"),
            _ => panic!("Expected text element"),
        }
    }

    #[test]
    fn test_malformed_wrapper_formats() {
        // Test nested wrapper (double wrapped) - now works due to recursive parsing
        let json = r#"{
            "json": {
                "json": {
                    "title": "Double Wrapped",
                    "elements": [{"text": "test", "id": "double_wrap_text"}]
                }
            }
        }"#;
        let result = parse_popup_json(json);
        assert!(
            result.is_ok(),
            "Double-wrapped json now works due to recursive parsing"
        );
        let popup = result.unwrap();
        assert_eq!(popup.title, Some("Double Wrapped".to_string()));

        // Test empty array in json field - now valid (empty popup)
        let json = r#"{"json": []}"#;
        let result = parse_popup_json(json);
        assert!(result.is_ok(), "Empty array in json field is now valid");
        let popup = result.unwrap();
        assert_eq!(popup.elements.len(), 0);
    }

    #[test]
    fn test_convenience_parsing_functions() {
        // Test direct format parser
        let direct_json = r#"{
            "title": "Direct Test",
            "elements": [{"text": "direct", "id": "direct_text"}]
        }"#;
        let popup = parse_popup_from_direct(direct_json).unwrap();
        assert_eq!(popup.title, Some("Direct Test".to_string()));

        // Test wrapper format parser
        let wrapper_json = r#"{
            "json": {
                "title": "Wrapper Test",
                "elements": [{"text": "wrapper", "id": "wrapper_text"}]
            }
        }"#;
        let popup = parse_popup_from_mcp_wrapper(wrapper_json).unwrap();
        assert_eq!(popup.title, Some("Wrapper Test".to_string()));

        // Test that direct parser fails on wrapper format
        let result = parse_popup_from_direct(wrapper_json);
        assert!(
            result.is_err(),
            "Direct parser should fail on wrapper format"
        );

        // Test that wrapper parser fails on direct format
        let result = parse_popup_from_mcp_wrapper(direct_json);
        assert!(
            result.is_err(),
            "Wrapper parser should fail on direct format"
        );
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

        // New formats
        let element = r#"{"text": "Hello"}"#;
        assert_eq!(detect_popup_format(element), "element");

        let array = r#"[{"text": "Hello"}, {"check": "OK", "id": "ok"}]"#;
        assert_eq!(detect_popup_format(array), "array");
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

    // Phase 6: Single-element root tests

    #[test]
    fn test_single_element_root_text() {
        // Simplest case: just text
        let json = r#"{"text": "Delete this file? This cannot be undone."}"#;
        let popup = parse_popup_json(json).unwrap();

        assert_eq!(popup.title, None);
        assert_eq!(popup.elements.len(), 1);
        match &popup.elements[0] {
            Element::Text { text, .. } => {
                assert_eq!(text, "Delete this file? This cannot be undone.")
            }
            _ => panic!("Expected text element"),
        }
    }

    #[test]
    fn test_single_element_root_slider() {
        let json = r#"{"slider": "Volume", "id": "volume", "min": 0, "max": 100}"#;
        let popup = parse_popup_json(json).unwrap();

        assert_eq!(popup.title, None);
        assert_eq!(popup.elements.len(), 1);
        match &popup.elements[0] {
            Element::Slider {
                slider, min, max, ..
            } => {
                assert_eq!(slider, "Volume");
                assert_eq!(*min, 0.0);
                assert_eq!(*max, 100.0);
            }
            _ => panic!("Expected slider element"),
        }
    }

    #[test]
    fn test_single_element_root_choice_with_option_children() {
        // Complex single element with nested structure
        let json = r#"{
            "select": "Mode",
            "id": "mode",
            "options": ["Simple", "Advanced"],
            "Advanced": [
                {"slider": "Level", "id": "level", "min": 1, "max": 10}
            ]
        }"#;
        let popup = parse_popup_json(json).unwrap();

        assert_eq!(popup.title, None);
        assert_eq!(popup.elements.len(), 1);
        match &popup.elements[0] {
            Element::Select {
                select,
                options,
                option_children,
                ..
            } => {
                assert_eq!(select, "Mode");
                assert_eq!(options.len(), 2);
                assert!(option_children.contains_key("Advanced"));
            }
            _ => panic!("Expected choice element"),
        }
    }

    // Phase 6: Array root tests

    #[test]
    fn test_array_root_simple() {
        let json = r#"[
            {"text": "Configure audio settings"},
            {"slider": "Volume", "id": "volume", "min": 0, "max": 100},
            {"check": "Mute", "id": "mute"}
        ]"#;
        let popup = parse_popup_json(json).unwrap();

        assert_eq!(popup.title, None);
        assert_eq!(popup.elements.len(), 3);

        match &popup.elements[0] {
            Element::Text { text, .. } => assert_eq!(text, "Configure audio settings"),
            _ => panic!("Expected text element"),
        }
        match &popup.elements[1] {
            Element::Slider { slider, .. } => assert_eq!(slider, "Volume"),
            _ => panic!("Expected slider element"),
        }
        match &popup.elements[2] {
            Element::Check { check, .. } => assert_eq!(check, "Mute"),
            _ => panic!("Expected checkbox element"),
        }
    }

    #[test]
    fn test_array_root_with_when_clauses() {
        let json = r#"[
            {"check": "Enable advanced", "id": "enable_advanced"},
            {"slider": "Level", "id": "level", "min": 1, "max": 10, "when": "@enable_advanced"}
        ]"#;
        let popup = parse_popup_json(json).unwrap();

        assert_eq!(popup.elements.len(), 2);
        match &popup.elements[1] {
            Element::Slider { when, .. } => assert_eq!(when.as_deref(), Some("@enable_advanced")),
            _ => panic!("Expected slider element"),
        }
    }

    #[test]
    fn test_wrapper_with_single_element() {
        // MCP wrapper around single element
        let json = r#"{"json": {"text": "Hello from MCP"}}"#;
        let popup = parse_popup_json(json).unwrap();

        assert_eq!(popup.title, None);
        assert_eq!(popup.elements.len(), 1);
        match &popup.elements[0] {
            Element::Text { text, .. } => assert_eq!(text, "Hello from MCP"),
            _ => panic!("Expected text element"),
        }
    }

    #[test]
    fn test_wrapper_with_array() {
        // MCP wrapper around array
        let json = r#"{"json": [
            {"text": "First"},
            {"text": "Second"}
        ]}"#;
        let popup = parse_popup_json(json).unwrap();

        assert_eq!(popup.title, None);
        assert_eq!(popup.elements.len(), 2);
    }

    #[test]
    fn test_validation_of_new_formats() {
        // Single element
        assert!(validate_popup_json(r#"{"text": "Hello"}"#).is_ok());
        assert!(
            validate_popup_json(r#"{"slider": "Vol", "id": "vol", "min": 0, "max": 100}"#).is_ok()
        );

        // Array
        assert!(validate_popup_json(r#"[{"text": "A"}, {"text": "B"}]"#).is_ok());

        // Invalid single element (missing required fields)
        assert!(validate_popup_json(r#"{"slider": "Vol"}"#).is_err()); // missing id, min, max
    }
}
