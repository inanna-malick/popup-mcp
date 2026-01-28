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
        "description": "Create a rich, branching GUI interaction that captures a full decision tree in a single turn.\n\nPHILOSOPHY: Stop the back-and-forth. Don't ask a question, wait for an answer, and then ask another. Instead, anticipate the user's choices and pre-load the appropriate follow-up questions into the interface.\n\nCORE MECHANIC: Deeply nested conditional logic.\n- If asking 'Deployment Environment', immediately nest 'Production' and 'Staging' specific config fields under those respective options.\n\nCRITICAL STRUCTURAL RULE: Branch definitions must be INSIDE the parent widget object, NOT as the next item in the list.\n\n✅ CORRECT (Nested):\n[\n  { \"select\": \"Mode\", \"options\": \"A, B\", \"A\": [{...}], \"B\": [{...}] }\n]\n\n❌ INCORRECT (Sibling):\n[\n  { \"select\": \"Mode\", \"options\": \"A, B\" },\n  { \"A\": [{...}] } // Error: This is a standalone object\n]\n\nBRANCHING SYNTAX:\n- Option-Specific Children: \"Prod\": [{...prod_fields...}] (Preferred)\n- Checkbox/Reveal: \"reveals\": [{...config...}]\n- Complex Logic: \"when\": \"env == 'Prod' && !use_existing_key\"\n\nAUTO-INJECTED 'OTHER':\n'select' and 'multi' widgets automatically get an 'Other (please specify)' option. Do NOT add it manually.\n\nRETURNS: {\"status\": \"completed\", \"button\": \"submit\", \"field_id\": value}\n- select/multi return the text value (e.g., \"Prod\")",
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
                "description": "A semantically-meaningful short sentence-length description of the popup's purpose (e.g., 'Configure your profile settings' or 'Confirm deletion of the project')."
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
                                    "description": "Element ID for state tracking (Optional: Auto-generated from label if omitted)"
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
                                    "oneOf": [
                                        { "$ref": "#/properties/elements" },
                                        { "$ref": "#/properties/elements/items" }
                                    ],
                                    "description": "Child elements shown when checkbox is checked. Can be an array or a single element object."
                                },
                            },
                            "required": ["check"],
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
                                    "description": "Element ID for state tracking (Optional: Auto-generated from label if omitted)"
                                },
                                "options": {
                                    "oneOf": [
                                        {
                                            "type": "array",
                                            "items": { "type": "string" },
                                            "minItems": 1
                                        },
                                        {
                                            "type": "string",
                                            "description": "Comma-separated string of options (e.g. 'A, B, C')"
                                        }
                                    ],
                                    "description": "Options to select from. Can be an array or a comma-separated string."
                                },
                                "when": {
                                    "type": "string",
                                    "description": "Condition for visibility. Syntax: id (truthy check), selected(id, \"value\"), count(id) > N, with &&/||/! operators"
                                },
                                "reveals": {
                                    "oneOf": [
                                        { "$ref": "#/properties/elements" },
                                        { "$ref": "#/properties/elements/items" }
                                    ],
                                    "description": "Child elements shown when any option is selected. Can be an array or a single element object."
                                },
                            },
                            "required": ["multi", "options"],
                            "patternProperties": {
                                "^(?!multi|id|options|when|reveals).*$": {
                                    "oneOf": [
                                        { "$ref": "#/properties/elements" },
                                        { "$ref": "#/properties/elements/items" },
                                        { "type": "string", "description": "Implicit Text element" }
                                    ],
                                    "description": "Option-as-key: Use option text as JSON key for child elements. Can be Array (multiple), Object (single), or String (text)."
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
                                    "description": "Element ID for state tracking (Optional: Auto-generated from label if omitted)"
                                },
                                "options": {
                                    "oneOf": [
                                        {
                                            "type": "array",
                                            "items": { "type": "string" },
                                            "minItems": 1
                                        },
                                        {
                                            "type": "string",
                                            "description": "Comma-separated string of options (e.g. 'A, B, C')"
                                        }
                                    ],
                                    "description": "Options to select from. Can be an array or a comma-separated string."
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
                                    "oneOf": [
                                        { "$ref": "#/properties/elements" },
                                        { "$ref": "#/properties/elements/items" }
                                    ],
                                    "description": "Child elements shown when any option is selected. Can be an array or a single element object."
                                },
                            },
                            "required": ["select", "options"],
                            "patternProperties": {
                                "^(?!select|id|options|default|when|reveals).*$": {
                                    "oneOf": [
                                        { "$ref": "#/properties/elements" },
                                        { "$ref": "#/properties/elements/items" },
                                        { "type": "string", "description": "Implicit Text element" }
                                    ],
                                    "description": "Option-as-key: Use option text as JSON key for child elements. Can be Array (multiple), Object (single), or String (text)."
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
        "required": ["title", "elements"],
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
    "Create a rich, branching GUI interaction that captures a full decision tree in a single turn.

PHILOSOPHY: Stop the back-and-forth. Don't ask a question, wait for an answer, and then ask another.
Instead, anticipate the user's choices and pre-load the appropriate follow-up questions into the interface.

Your goal is to receive a complete, actionable state from the user in ONE interaction.

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

BRANCHING SYNTAX:
- Option-Specific Children: \"Prod\": [{...prod_fields...}] (Preferred)
- Checkbox/Reveal: \"reveals\": [{...config...}]
- Complex Logic: \"when\": \"env == 'Prod' && !use_existing_key\"

RETURNS: {\"status\": \"completed\", \"<id>\": value, ...}
- select/multi: selected text
- 'Other': returns both selection AND <id>_other_text field"
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
