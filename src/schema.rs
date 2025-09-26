//! JSON Schema generation for popup definitions
//!
//! Provides functions to generate JSON schemas for MCP tool definitions
//! so library consumers can properly validate popup structures.

use serde_json::json;

/// Get the complete MCP tool schema for the popup tool
///
/// This includes name, description, and full inputSchema.
/// Library consumers can use this directly in their MCP tool definitions.
pub fn get_popup_tool_schema() -> serde_json::Value {
    json!({
        "name": "popup",
        "description": "Create a native GUI popup window. Returns user inputs when Submit is clicked, or cancellation status.",
        "inputSchema": get_input_schema()
    })
}

/// Get just the inputSchema portion of the tool definition
///
/// Use this if you want to customize the name or description
/// but need the proper schema for the popup structure.
pub fn get_input_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "json": {
                "type": "object",
                "properties": {
                    "title": {
                        "type": "string",
                        "description": "Title of the popup window"
                    },
                    "elements": {
                        "type": "array",
                        "description": "Array of UI elements to display",
                        "items": {
                            "oneOf": [
                                // Text element
                                {
                                    "type": "object",
                                    "properties": {
                                        "type": {"const": "text"},
                                        "content": {
                                            "type": "string",
                                            "description": "Text to display"
                                        }
                                    },
                                    "required": ["type", "content"],
                                    "additionalProperties": false
                                },
                                // Slider element
                                {
                                    "type": "object",
                                    "properties": {
                                        "type": {"const": "slider"},
                                        "label": {
                                            "type": "string",
                                            "description": "Label for the slider"
                                        },
                                        "min": {
                                            "type": "number",
                                            "description": "Minimum value"
                                        },
                                        "max": {
                                            "type": "number",
                                            "description": "Maximum value"
                                        },
                                        "default": {
                                            "type": "number",
                                            "description": "Default value (optional, defaults to midpoint)"
                                        }
                                    },
                                    "required": ["type", "label", "min", "max"],
                                    "additionalProperties": false
                                },
                                // Checkbox element
                                {
                                    "type": "object",
                                    "properties": {
                                        "type": {"const": "checkbox"},
                                        "label": {
                                            "type": "string",
                                            "description": "Label for the checkbox"
                                        },
                                        "default": {
                                            "type": "boolean",
                                            "description": "Default checked state",
                                            "default": false
                                        }
                                    },
                                    "required": ["type", "label"],
                                    "additionalProperties": false
                                },
                                // Textbox element
                                {
                                    "type": "object",
                                    "properties": {
                                        "type": {"const": "textbox"},
                                        "label": {
                                            "type": "string",
                                            "description": "Label for the text input"
                                        },
                                        "placeholder": {
                                            "type": "string",
                                            "description": "Placeholder text (optional)"
                                        },
                                        "rows": {
                                            "type": "integer",
                                            "minimum": 1,
                                            "description": "Number of rows for multiline input (optional)"
                                        }
                                    },
                                    "required": ["type", "label"],
                                    "additionalProperties": false
                                },
                                // Multiselect element
                                {
                                    "type": "object",
                                    "properties": {
                                        "type": {"const": "multiselect"},
                                        "label": {
                                            "type": "string",
                                            "description": "Label for the multiselect"
                                        },
                                        "options": {
                                            "type": "array",
                                            "items": {"type": "string"},
                                            "minItems": 1,
                                            "description": "Array of options for multiple selection"
                                        }
                                    },
                                    "required": ["type", "label", "options"],
                                    "additionalProperties": false
                                },
                                // Group element
                                {
                                    "type": "object",
                                    "properties": {
                                        "type": {"const": "group"},
                                        "label": {
                                            "type": "string",
                                            "description": "Label for the group"
                                        },
                                        "elements": {
                                            "$ref": "#/properties/json/properties/elements",
                                            "description": "Nested elements within the group"
                                        }
                                    },
                                    "required": ["type", "label", "elements"],
                                    "additionalProperties": false
                                },
                                // Conditional element
                                {
                                    "type": "object",
                                    "properties": {
                                        "type": {"const": "conditional"},
                                        "condition": {
                                            "oneOf": [
                                                {
                                                    "type": "string",
                                                    "description": "Pattern 1: Simple existence check - true if checkbox checked OR any multiselect option selected"
                                                },
                                                {
                                                    "type": "object",
                                                    "properties": {
                                                        "field": {
                                                            "type": "string",
                                                            "description": "Field name (checkbox or multiselect)"
                                                        },
                                                        "value": {
                                                            "type": "string",
                                                            "description": "Specific value - checkbox name must match OR multiselect option must be selected"
                                                        }
                                                    },
                                                    "required": ["field", "value"],
                                                    "additionalProperties": false
                                                },
                                                {
                                                    "type": "object",
                                                    "properties": {
                                                        "field": {
                                                            "type": "string",
                                                            "description": "Field name (checkbox or multiselect)"
                                                        },
                                                        "count": {
                                                            "type": "string",
                                                            "description": "Count condition like '>2', '=1', '<=3' - checkbox counts as 0 or 1"
                                                        }
                                                    },
                                                    "required": ["field", "count"],
                                                    "additionalProperties": false
                                                }
                                            ],
                                            "description": "Unified condition patterns that work for both checkbox and multiselect"
                                        },
                                        "elements": {
                                            "$ref": "#/properties/json/properties/elements",
                                            "description": "Elements shown when condition is true"
                                        }
                                    },
                                    "required": ["type", "condition", "elements"],
                                    "additionalProperties": false
                                }
                            ]
                        }
                    }
                },
                "required": ["elements"],
                "additionalProperties": false
            }
        },
        "required": ["json"],
        "additionalProperties": false
    })
}

/// Get a simplified MCP tool schema for basic popups
///
/// This radically simplified version supports only:
/// - text: Static text display
/// - textbox: Text input fields
/// - multiselect: Multiple selection lists
pub fn get_simple_popup_tool_schema() -> serde_json::Value {
    json!({
        "name": "popup",
        "description": "Create a simple popup with text display, text input, and multi-selection options.",
        "inputSchema": get_simple_input_schema()
    })
}

/// Get the simplified inputSchema without Group and Conditional elements
///
/// This radically simplified schema supports only:
/// - text: Static text display
/// - textbox: Text input field  
/// - multiselect: Multiple selection list
pub fn get_simple_input_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "json": {
                "type": "object",
                "properties": {
                    "title": {
                        "type": "string",
                        "description": "Title of the popup window"
                    },
                    "elements": {
                        "type": "array",
                        "description": "Array of UI elements (text, textbox, multiselect only)",
                        "items": {
                            "oneOf": [
                                // Text element - static display
                                {
                                    "type": "object",
                                    "properties": {
                                        "type": {"const": "text"},
                                        "content": {
                                            "type": "string",
                                            "description": "Text to display"
                                        }
                                    },
                                    "required": ["type", "content"],
                                    "additionalProperties": false
                                },
                                // Textbox element - text input
                                {
                                    "type": "object",
                                    "properties": {
                                        "type": {"const": "textbox"},
                                        "label": {
                                            "type": "string",
                                            "description": "Label for the text field"
                                        },
                                        "placeholder": {
                                            "type": "string",
                                            "description": "Placeholder text"
                                        },
                                        "rows": {
                                            "type": "integer",
                                            "description": "Number of rows (for multiline)",
                                            "minimum": 1
                                        }
                                    },
                                    "required": ["type", "label"],
                                    "additionalProperties": false
                                },
                                // Multiselect element - multiple selection
                                {
                                    "type": "object",
                                    "properties": {
                                        "type": {"const": "multiselect"},
                                        "label": {
                                            "type": "string",
                                            "description": "Label for multiselect"
                                        },
                                        "options": {
                                            "type": "array",
                                            "description": "Available options",
                                            "items": {"type": "string"},
                                            "minItems": 1
                                        }
                                    },
                                    "required": ["type", "label", "options"],
                                    "additionalProperties": false
                                }
                            ]
                        }
                    }
                },
                "required": ["elements"],
                "additionalProperties": false
            }
        },
        "required": ["json"],
        "additionalProperties": false
    })
}

/// Get a human-readable description of the popup schema
///
/// Useful for documentation or help text
pub fn get_schema_description() -> &'static str {
    "Popup JSON structure:
{
  \"title\": \"Window title\",
  \"elements\": [
    {\"type\": \"text\", \"content\": \"Display text\"},
    {\"type\": \"slider\", \"label\": \"Volume\", \"min\": 0, \"max\": 100, \"default\": 50},
    {\"type\": \"checkbox\", \"label\": \"Enable\", \"default\": true},
    {\"type\": \"textbox\", \"label\": \"Name\", \"placeholder\": \"Enter name\", \"rows\": 3},
    {\"type\": \"choice\", \"label\": \"Color\", \"options\": [\"Red\", \"Blue\"]},
    {\"type\": \"multiselect\", \"label\": \"Features\", \"options\": [\"A\", \"B\", \"C\"]},
    {\"type\": \"group\", \"label\": \"Settings\", \"elements\": [...]},
    {\"type\": \"conditional\", \"condition\": \"checkbox_label\", \"elements\": [...]}
  ]
}

Returns: {\"button\": \"submit\" | \"cancel\", \"field_label\": value, ...}"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_generation() {
        let schema = get_popup_tool_schema();
        assert!(schema.is_object());
        assert_eq!(schema["name"], "popup");
        assert!(schema["inputSchema"].is_object());
    }

    #[test]
    fn test_input_schema() {
        let schema = get_input_schema();
        assert!(schema.is_object());
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["json"].is_object());
    }
}
