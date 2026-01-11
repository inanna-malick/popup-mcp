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
        "description": "Create rich dialogue trees with cascading conditional branches. Build entire decision flows in a single interaction - when Choice A reveals Options B & C, which themselves unlock Paths D, E, F. Think interactive fiction: the user sees their choices unfold dynamically as they engage, discovering the full conversation tree through their selections. Instead of ping-ponging through multiple rounds ('What type?' → wait → 'What size?' → wait → 'What color?'), present the entire adaptive form that reshapes itself based on each choice. Every checkbox that reveals sliders, every dropdown that unlocks text fields, every multiselect that exposes new option groups - these create a responsive dialogue that guides users through complex decision spaces without round-trip latency.",
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
            "title": {
                "type": "string",
                "description": "Title of the popup window"
            },
            "elements": {
                "type": "array",
                "description": "Array of UI elements to display",
                "items": {
                    "oneOf": [
                        // Text element (V2: element-as-key, id optional)
                        {
                            "type": "object",
                            "properties": {
                                "text": {
                                    "type": "string",
                                    "description": "Text to display"
                                },
                                "id": {
                                    "type": "string",
                                    "description": "Optional element ID (text elements don't need state tracking)"
                                },
                                "when": {
                                    "type": "string",
                                    "description": "Optional when clause for conditional visibility"
                                }
                            },
                            "required": ["text"],
                            "additionalProperties": false
                        },
                        // Slider element (V2: slider is the key, id required)
                        {
                            "type": "object",
                            "properties": {
                                "slider": {
                                    "type": "string",
                                    "description": "Label for the slider"
                                },
                                "id": {
                                    "type": "string",
                                    "description": "Element ID for state tracking (required)"
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
                                },
                                "when": {
                                    "type": "string",
                                    "description": "Optional when clause for conditional visibility"
                                },
                                "reveals": {
                                    "$ref": "#/properties/elements",
                                    "description": "Optional inline conditional elements shown when slider is active"
                                }
                            },
                            "required": ["slider", "id", "min", "max"],
                            "additionalProperties": false
                        },
                        // Checkbox element (V2: checkbox is the key, id required)
                        {
                            "type": "object",
                            "properties": {
                                "checkbox": {
                                    "type": "string",
                                    "description": "Label for the checkbox"
                                },
                                "id": {
                                    "type": "string",
                                    "description": "Element ID for state tracking (required)"
                                },
                                "default": {
                                    "type": "boolean",
                                    "description": "Default checked state"
                                },
                                "when": {
                                    "type": "string",
                                    "description": "Optional when clause for conditional visibility"
                                },
                                "reveals": {
                                    "$ref": "#/properties/elements",
                                    "description": "Optional inline conditional elements shown when checkbox is checked"
                                }
                            },
                            "required": ["checkbox", "id"],
                            "additionalProperties": false
                        },
                        // Textbox element (V2: textbox is the key, id required)
                        {
                            "type": "object",
                            "properties": {
                                "textbox": {
                                    "type": "string",
                                    "description": "Label for the text input"
                                },
                                "id": {
                                    "type": "string",
                                    "description": "Element ID for state tracking (required)"
                                },
                                "placeholder": {
                                    "type": "string",
                                    "description": "Placeholder text (optional)"
                                },
                                "rows": {
                                    "type": "integer",
                                    "minimum": 1,
                                    "description": "Number of rows for multiline input (optional)"
                                },
                                "when": {
                                    "type": "string",
                                    "description": "Optional when clause for conditional visibility"
                                }
                            },
                            "required": ["textbox", "id"],
                            "additionalProperties": false
                        },
                        // Multiselect element (V2: multiselect is the key, id required, option-as-key for children)
                        {
                            "type": "object",
                            "properties": {
                                "multiselect": {
                                    "type": "string",
                                    "description": "Label for the multiselect"
                                },
                                "id": {
                                    "type": "string",
                                    "description": "Element ID for state tracking (required)"
                                },
                                "options": {
                                    "type": "array",
                                    "items": {
                                        "type": "string"
                                    },
                                    "minItems": 1,
                                    "description": "Array of option strings"
                                },
                                "when": {
                                    "type": "string",
                                    "description": "Optional when clause for conditional visibility"
                                },
                                "reveals": {
                                    "$ref": "#/properties/elements",
                                    "description": "Optional inline conditional elements shown when any option is selected"
                                }
                            },
                            "required": ["multiselect", "id", "options"],
                            "patternProperties": {
                                "^(?!multiselect|id|options|when|reveals).*$": {
                                    "$ref": "#/properties/elements",
                                    "description": "Option-as-key: Use option text as JSON key for child elements"
                                }
                            },
                            "additionalProperties": false
                        },
                        // Choice element (V2: choice is the key, id required, option-as-key for children)
                        {
                            "type": "object",
                            "properties": {
                                "choice": {
                                    "type": "string",
                                    "description": "Label for the dropdown"
                                },
                                "id": {
                                    "type": "string",
                                    "description": "Element ID for state tracking (required)"
                                },
                                "options": {
                                    "type": "array",
                                    "items": {
                                        "type": "string"
                                    },
                                    "minItems": 1,
                                    "description": "Array of option strings"
                                },
                                "default": {
                                    "type": "integer",
                                    "minimum": 0,
                                    "description": "Default selected option index (optional, no selection if omitted)"
                                },
                                "when": {
                                    "type": "string",
                                    "description": "Optional when clause for conditional visibility"
                                },
                                "reveals": {
                                    "$ref": "#/properties/elements",
                                    "description": "Optional inline conditional elements shown when any option is selected"
                                }
                            },
                            "required": ["choice", "id", "options"],
                            "patternProperties": {
                                "^(?!choice|id|options|default|when|reveals).*$": {
                                    "$ref": "#/properties/elements",
                                    "description": "Option-as-key: Use option text as JSON key for child elements"
                                }
                            },
                            "additionalProperties": false
                        },
                        // Group element (V2: group is the key)
                        {
                            "type": "object",
                            "properties": {
                                "group": {
                                    "type": "string",
                                    "description": "Label for the group"
                                },
                                "elements": {
                                    "$ref": "#/properties/elements",
                                    "description": "Nested elements within the group"
                                },
                                "when": {
                                    "type": "string",
                                    "description": "Optional when clause for conditional visibility"
                                }
                            },
                            "required": ["group", "elements"],
                            "additionalProperties": false
                        }
                    ]
                }
            }
        },
        "required": ["elements"],
        "additionalProperties": false,
        "examples": [
            {
                "title": "Quick confirmation",
                "elements": [
                    {"text": "Are you sure you want to proceed?", "id": "confirm_text"},
                    {"checkbox": "Don't ask again", "id": "dont_ask"}
                ]
            },
            {
                "title": "User preferences",
                "elements": [
                    {"choice": "Theme", "id": "theme", "options": ["Light", "Dark", "Auto"], "default": 2},
                    {"slider": "Font size", "id": "font_size", "min": 8, "max": 24, "default": 14}
                ]
            },
            {
                "title": "Conditional form",
                "elements": [
                    {"checkbox": "Enable advanced options", "id": "advanced", "reveals": [
                        {"slider": "Debug level", "id": "debug", "min": 0, "max": 5}
                    ]}
                ]
            }
        ]
    })
}

/// Get a human-readable description of the popup schema
///
/// Useful for documentation or help text
pub fn get_schema_description() -> &'static str {
    "Popup JSON structure (V2 element-as-key format):
{
  \"title\": \"Window title\",
  \"elements\": [
    {\"text\": \"Display text\", \"id\": \"optional_id\"},
    {\"slider\": \"Volume\", \"id\": \"volume\", \"min\": 0, \"max\": 100, \"default\": 50},
    {\"checkbox\": \"Enable\", \"id\": \"enable\", \"default\": true, \"reveals\": [...]},
    {\"textbox\": \"Name\", \"id\": \"name\", \"placeholder\": \"Enter name\", \"rows\": 3},
    {\"choice\": \"Color\", \"id\": \"color\", \"options\": [\"Red\", \"Blue\"], \"Blue\": [...]},
    {\"multiselect\": \"Features\", \"id\": \"features\", \"options\": [\"A\", \"B\"], \"A\": [...]},
    {\"group\": \"Settings\", \"elements\": [...]},
  ]
}

V2 Features:
- Element-as-key: Widget type is the JSON key (\"slider\": \"Label\", not \"type\": \"slider\")
- ID-based state: All interactive elements require \"id\" field for state tracking
- When clauses: Any element can have \"when\": \"@id && count(@other) > 2\" for conditional visibility
- Reveals: Inline conditionals via \"reveals\": [...] on checkbox/multiselect/choice
- Option-as-key nesting: Use option text as JSON key for per-option children

Returns: {\"button\": \"submit\" | \"cancel\", \"element_id\": value, ...}"
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
        assert!(schema["properties"]["title"].is_object());
        assert!(schema["properties"]["elements"].is_object());
    }
}
