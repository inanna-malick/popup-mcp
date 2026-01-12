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
        "description": "Author a dialogue tree that collapses multiple conversation turns into one interaction.\n\nCORE PATTERN: Anticipate likely user responses. For each anticipated answer, encode branch-specific followup questions. Nest 3-5 levels deep.\n\nANTI-PATTERN: Don't build flat forms. If an answer would prompt a followup question, encode that followup as a nested branch NOW.\n\nBRANCHING MECHANICS:\n- option-as-key: {\"select\": \"Lang\", \"id\": \"x\", \"options\": [\"Rust\", \"Go\"], \"Rust\": [{...rust-specific followups...}]}\n- reveals: {\"check\": \"Advanced\", \"id\": \"x\", \"reveals\": [{...shown when checked...}]}\n- when: {\"slider\": \"X\", \"when\": \"advanced && selected(lang, 'Rust')\"}\n\nELEMENTS: text, slider, check, input, select (single-select), multi, group\n\nAUTO-INJECTED 'OTHER' OPTION:\nAll 'select' and 'multi' elements automatically receive an 'Other (please specify)' option. When selected, a text input appears for custom values. DO NOT manually add 'Other' options - they are added automatically.\n\nRETURN FORMAT:\n- select/multi: Returns selected option text (\"Rust\", not index)\n- When 'Other' is selected: Returns both \"Other (please specify)\" in the selection AND a separate \"<id>_other_text\" field with the custom text\n- Example: {\"mode\": \"Other (please specify)\", \"mode_other_text\": \"Custom mode\"}\n- slider: integer value (7)\n- Only visible elements included\n\nRETURNS: {\"status\": \"completed\"|\"cancelled\", \"button\": \"submit\"|\"cancel\", \"<id>\": value}\n\nNOTE: Widget type names (text, slider, check, input, select, multi, group) are reserved and cannot be used as option values in select/multi.",
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
                                    "description": "Condition for visibility. Syntax: id (truthy check), selected(id, \"value\"), count(id) > N, with &&/||/! operators"
                                },
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
                                    "description": "Condition for visibility. Syntax: id (truthy check), selected(id, \"value\"), count(id) > N, with &&/||/! operators"
                                }
                            },
                            "required": ["slider", "id", "min", "max"],
                            "additionalProperties": false
                        },
                        // Checkbox element (V2: checkbox is the key, id required)
                        {
                            "type": "object",
                            "properties": {
                                "check": {
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
                                    "description": "Condition for visibility. Syntax: id (truthy check), selected(id, \"value\"), count(id) > N, with &&/||/! operators"
                                },
                                "reveals": {
                                    "$ref": "#/properties/elements",
                                    "description": "Child elements shown when checkbox is checked"
                                },
                            },
                            "required": ["check", "id"],
                            "additionalProperties": false
                        },
                        // Textbox element (V2: textbox is the key, id required)
                        {
                            "type": "object",
                            "properties": {
                                "input": {
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
                                    "description": "Condition for visibility. Syntax: id (truthy check), selected(id, \"value\"), count(id) > N, with &&/||/! operators"
                                },
                            },
                            "required": ["input", "id"],
                            "additionalProperties": false
                        },
                        // Multiselect element (V2: multiselect is the key, id required, option-as-key for children)
                        {
                            "type": "object",
                            "properties": {
                                "multi": {
                                    "type": "string",
                                    "description": "Label for the multiselect. NOTE: An 'Other (please specify)' option is automatically added - do not include it manually."
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
                                    "description": "Array of option strings. 'Other (please specify)' is automatically appended - do not add manually. When 'Other' is selected, result includes both the selection and a '<id>_other_text' field."
                                },
                                "when": {
                                    "type": "string",
                                    "description": "Condition for visibility. Syntax: id (truthy check), selected(id, \"value\"), count(id) > N, with &&/||/! operators"
                                },
                                "reveals": {
                                    "$ref": "#/properties/elements",
                                    "description": "Child elements shown when any option is selected"
                                },
                            },
                            "required": ["multi", "id", "options"],
                            "patternProperties": {
                                "^(?!multi|id|options|when|reveals).*$": {
                                    "$ref": "#/properties/elements",
                                    "description": "Option-as-key: Use option text as JSON key for child elements shown when that option is selected"
                                }
                            },
                            "additionalProperties": false
                        },
                        // Choice element (V2: choice is the key, id required, option-as-key for children)
                        {
                            "type": "object",
                            "properties": {
                                "select": {
                                    "type": "string",
                                    "description": "Label for the dropdown. NOTE: An 'Other (please specify)' option is automatically added - do not include it manually."
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
                                    "description": "Array of option strings. 'Other (please specify)' is automatically appended - do not add manually. When 'Other' is selected, result includes '<id>': 'Other (please specify)' and '<id>_other_text': 'custom value'."
                                },
                                "default": {
                                    "type": "string",
                                    "description": "Default selected option value (must match an option, omit for no selection)"
                                },
                                "when": {
                                    "type": "string",
                                    "description": "Condition for visibility. Syntax: id (truthy check), selected(id, \"value\"), count(id) > N, with &&/||/! operators"
                                },
                                "reveals": {
                                    "$ref": "#/properties/elements",
                                    "description": "Child elements shown when any option is selected"
                                },
                            },
                            "required": ["select", "id", "options"],
                            "patternProperties": {
                                "^(?!select|id|options|default|when|reveals).*$": {
                                    "$ref": "#/properties/elements",
                                    "description": "Option-as-key: Use option text as JSON key for child elements shown when that option is selected"
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
                                    "description": "Condition for visibility. Syntax: id (truthy check), selected(id, \"value\"), count(id) > N, with &&/||/! operators"
                                },
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
                    {"check": "Don't ask again", "id": "dont_ask"}
                ]
            },
            {
                "title": "User preferences",
                "elements": [
                    {"select": "Theme", "id": "theme", "options": ["Light", "Dark", "Auto"], "default": "Auto"},
                    {"slider": "Font size", "id": "font_size", "min": 8, "max": 24, "default": 14}
                ]
            },
            {
                "title": "Conditional form",
                "elements": [
                    {"check": "Enable advanced options", "id": "advanced", "reveals": [
                        {"slider": "Debug level", "id": "debug", "min": 0, "max": 5}
                    ]}
                ]
            },
            {
                "title": "Deep nesting (3 levels) - anticipate answers",
                "elements": [
                    {"select": "Project", "id": "proj", "options": ["Web", "CLI", "Library"],
                     "Web": [{"select": "Framework", "id": "fw", "options": ["React", "Vue"],
                              "React": [{"check": "TypeScript", "id": "ts", "reveals": [
                                  {"check": "Strict mode", "id": "strict"}
                              ]}]}],
                     "CLI": [{"select": "Language", "id": "lang", "options": ["Rust", "Go"]}]}
                ]
            }
        ]
    })
}

/// Get a human-readable description of the popup schema
///
/// Useful for documentation or help text
pub fn get_schema_description() -> &'static str {
    "Author dialogue trees that collapse multiple conversation turns into one interaction.

CORE PATTERN: Anticipate likely user responses. For each anticipated answer,
encode branch-specific followup questions. Nest 3-5 levels deep.

STRUCTURE:
{
  \"title\": \"Window title\",
  \"elements\": [
    {\"text\": \"Display text\"},
    {\"slider\": \"Label\", \"id\": \"x\", \"min\": 0, \"max\": 100},
    {\"check\": \"Label\", \"id\": \"x\", \"reveals\": [...]},
    {\"input\": \"Label\", \"id\": \"x\", \"placeholder\": \"...\"},
    {\"select\": \"Label\", \"id\": \"x\", \"options\": [\"A\", \"B\"], \"A\": [...]},
    {\"multi\": \"Label\", \"id\": \"x\", \"options\": [\"A\", \"B\"], \"A\": [...]},
    {\"group\": \"Label\", \"elements\": [...]}
  ]
}

BRANCHING:
- option-as-key: \"A\": [...] adds children shown when option A selected
- reveals: [...] adds children shown when parent is active
- when: \"id && selected(other, 'value')\" for cross-element conditions

RETURNS: {\"status\": \"completed\", \"<id>\": value, ...} or {\"status\": \"cancelled\"}
- select/multi: selected text (not index)
- slider: integer value
- Only visible elements included

NOTE: Widget type names are reserved and cannot be used as option values."
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
