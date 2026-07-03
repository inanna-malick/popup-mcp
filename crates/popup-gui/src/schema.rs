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
        "description": get_schema_description(),
        "inputSchema": get_input_schema()
    })
}

/// Get just the inputSchema portion of the tool definition
///
/// Use this if you want to customize the name or description
/// but need the proper schema for the popup structure.
pub fn get_input_schema() -> serde_json::Value {
    let when_desc = "Visibility condition over other elements' ids. Syntax: bare id is a truthy check (\"advanced\"), selected(id, value) tests one option (value may be an unquoted phrase: selected(strategy, Regenerate from sandbox); quote it if it contains a closing paren), count(id) > N counts selections, any(...)/all(...) combine conditions, numbers may be negative, combine with && || ! and parentheses. Conditions only see visible elements' values: an id hidden inside an untaken branch reads as unset (falsy, unselected, count 0), so referencing it is safe without repeating the branch's condition.";

    json!({
        "type": "object",
        "properties": {
            "title": {
                "type": "string",
                "description": "Short sentence describing what this interaction decides (e.g. 'Configure deployment target')"
            },
            "submit": {
                "type": "string",
                "description": "Optional label for the submit button (default \"Submit\"). Use a verb matching the consequence, e.g. \"Deploy\", \"Send reply\"."
            },
            "notes": {
                "type": "boolean",
                "description": "Whether to show the collapsed free-text 'Add a note' field (default true). Leave enabled: it catches answers your options didn't anticipate."
            },
            "elements": {
                "type": "array",
                "description": "UI elements in display order. The element type is the JSON key that carries the label.",
                "items": {
                    "oneOf": [
                        {
                            "type": "object",
                            "description": "Static text",
                            "properties": {
                                "text": {"type": "string", "description": "Text to display"},
                                "id": {"type": "string", "description": "Optional id"},
                                "when": {"type": "string", "description": when_desc}
                            },
                            "required": ["text"],
                            "additionalProperties": false
                        },
                        {
                            "type": "object",
                            "description": "Markdown block for richer framing/context",
                            "properties": {
                                "markdown": {"type": "string", "description": "Markdown content"},
                                "id": {"type": "string", "description": "Optional id"},
                                "when": {"type": "string", "description": when_desc}
                            },
                            "required": ["markdown"],
                            "additionalProperties": false
                        },
                        {
                            "type": "object",
                            "description": "Numeric slider",
                            "properties": {
                                "slider": {"type": "string", "description": "Label"},
                                "id": {"type": "string", "description": "Unique id (auto-derived from label if omitted)"},
                                "min": {"type": "number"},
                                "max": {"type": "number"},
                                "default": {"type": "number", "description": "Defaults to midpoint"},
                                "step": {"type": "number", "description": "Increment (default 1). Whole steps return integers."},
                                "when": {"type": "string", "description": when_desc}
                            },
                            "required": ["slider", "min", "max"],
                            "additionalProperties": false
                        },
                        {
                            "type": "object",
                            "description": "Checkbox; nest follow-ups under `reveals` to show them only when checked",
                            "properties": {
                                "check": {"type": "string", "description": "Label"},
                                "id": {"type": "string", "description": "Unique id (auto-derived from label if omitted)"},
                                "default": {"type": "boolean", "description": "Initial checked state (default false)"},
                                "reveals": {
                                    "$ref": "#/properties/elements",
                                    "description": "Elements shown while checked - put checked-branch follow-up questions here"
                                },
                                "when": {"type": "string", "description": when_desc}
                            },
                            "required": ["check"],
                            "additionalProperties": false
                        },
                        {
                            "type": "object",
                            "description": "Text input",
                            "properties": {
                                "input": {"type": "string", "description": "Label"},
                                "id": {"type": "string", "description": "Unique id (auto-derived from label if omitted)"},
                                "placeholder": {"type": "string"},
                                "default": {"type": "string", "description": "Prefilled text. Propose your best guess so the user edits instead of typing from scratch."},
                                "rows": {"type": "integer", "minimum": 1, "description": "Rows for multiline input"},
                                "when": {"type": "string", "description": when_desc}
                            },
                            "required": ["input"],
                            "additionalProperties": false
                        },
                        {
                            "type": "object",
                            "description": "Single choice. BRANCH POINT: use an option's text as a direct JSON key mapping to an array of follow-up elements shown when that option is selected.",
                            "properties": {
                                "select": {"type": "string", "description": "Label"},
                                "id": {"type": "string", "description": "Unique id (auto-derived from label if omitted)"},
                                "options": {
                                    "type": "array",
                                    "minItems": 1,
                                    "items": {
                                        "oneOf": [
                                            {"type": "string"},
                                            {
                                                "type": "object",
                                                "properties": {
                                                    "value": {"type": "string"},
                                                    "description": {"type": "string", "description": "Shown under the option - use to explain tradeoffs"}
                                                },
                                                "required": ["value", "description"]
                                            }
                                        ]
                                    },
                                    "description": "Options. An 'Other (please specify)' option + text input is appended automatically; never add one yourself."
                                },
                                "default": {"type": "string", "description": "Preselected option text. Set to your recommendation."},
                                "style": {"type": "string", "enum": ["radio", "dropdown"], "description": "Override rendering. Auto: radio (all options visible) up to 6 options, dropdown beyond."},
                                "reveals": {
                                    "$ref": "#/properties/elements",
                                    "description": "Elements shown once anything is selected (branch-independent follow-ups)"
                                },
                                "when": {"type": "string", "description": when_desc}
                            },
                            "required": ["select", "options"],
                            "additionalProperties": {
                                "$ref": "#/properties/elements",
                                "description": "Option-as-key branch: key must exactly match an option's text; value is the follow-up elements for that branch"
                            }
                        },
                        {
                            "type": "object",
                            "description": "Multiple choice. Same option-as-key branching as select; a branch shows while its option is checked.",
                            "properties": {
                                "multi": {"type": "string", "description": "Label"},
                                "id": {"type": "string", "description": "Unique id (auto-derived from label if omitted)"},
                                "options": {
                                    "type": "array",
                                    "minItems": 1,
                                    "items": {
                                        "oneOf": [
                                            {"type": "string"},
                                            {
                                                "type": "object",
                                                "properties": {
                                                    "value": {"type": "string"},
                                                    "description": {"type": "string"}
                                                },
                                                "required": ["value", "description"]
                                            }
                                        ]
                                    },
                                    "description": "Options. 'Other (please specify)' is appended automatically."
                                },
                                "default": {
                                    "type": "array",
                                    "items": {"type": "string"},
                                    "description": "Option texts pre-checked on open. Set to your recommendation."
                                },
                                "reveals": {
                                    "$ref": "#/properties/elements",
                                    "description": "Elements shown once any option is selected"
                                },
                                "when": {"type": "string", "description": when_desc}
                            },
                            "required": ["multi", "options"],
                            "additionalProperties": {
                                "$ref": "#/properties/elements",
                                "description": "Option-as-key branch: key must exactly match an option's text"
                            }
                        },
                        {
                            "type": "object",
                            "description": "Visual grouping of related elements",
                            "properties": {
                                "group": {"type": "string", "description": "Group label"},
                                "id": {"type": "string"},
                                "elements": {"$ref": "#/properties/elements"},
                                "when": {"type": "string", "description": when_desc}
                            },
                            "required": ["group", "elements"],
                            "additionalProperties": false
                        },
                        {
                            "type": "object",
                            "description": "Wizard step. TOP-LEVEL ONLY: when used, every top-level element must be a page. One page shows at a time with Back/Next; Submit appears on the last page. Use for long flows that would otherwise be a wall of form; when-clauses can reference ids on earlier pages, and a page's own `when` skips it dynamically.",
                            "properties": {
                                "page": {"type": "string", "description": "Step label shown as the page heading"},
                                "id": {"type": "string"},
                                "elements": {"$ref": "#/properties/elements"},
                                "when": {"type": "string", "description": when_desc}
                            },
                            "required": ["page", "elements"],
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
                "title": "How should I fix the failing auth tests?",
                "elements": [
                    {"select": "Approach", "id": "approach", "options": [
                        {"value": "Patch the mock", "description": "Fast, but the mock drifts further from prod"},
                        {"value": "Rewrite against test containers", "description": "Slower, removes the drift for good"}
                     ],
                     "default": "Patch the mock",
                     "Rewrite against test containers": [
                        {"check": "Also migrate the session tests", "id": "migrate_sessions"},
                        {"slider": "Time budget (hours)", "id": "budget", "min": 1, "max": 8, "default": 3}
                     ]}
                ]
            },
            {
                "title": "Configure the new service",
                "elements": [
                    {"select": "Language", "id": "lang", "options": ["Rust", "TypeScript"],
                     "Rust": [{"select": "Async runtime", "id": "runtime", "options": ["tokio", "async-std"], "default": "tokio"}],
                     "TypeScript": [{"check": "Strict mode", "id": "strict", "default": true}]},
                    {"check": "Set up CI now", "id": "ci", "default": true, "reveals": [
                        {"multi": "CI jobs", "id": "ci_jobs", "options": ["test", "lint", "release"]}
                    ]},
                    {"input": "Service name", "id": "name", "default": "my-service"}
                ]
            }
        ]
    })
}

/// The popup tool description shown to MCP clients.
///
/// This is effectively a prompt: it teaches the calling model how to author
/// conversation trees. Result-format claims here must stay in sync with
/// PopupResult::from_state.
pub fn get_schema_description() -> &'static str {
    "Open a native GUI form and wait for the user to fill it in. Use this instead of asking questions in chat: one well-built popup replaces an entire back-and-forth conversation.

AUTHORING MINDSET: You are compressing a dialogue tree into one screen. For every question, predict the likely answers; for each answer that would trigger a follow-up question, NEST that follow-up inside the answer so it appears only when chosen. Flat forms that ignore answers waste the round trip. Nesting 3-5 levels deep is normal.

BRANCHING TOOLS (in order of preference):
1. option-as-key on select/multi - follow-ups for one specific answer:
   {\"select\": \"Database\", \"id\": \"db\", \"options\": [\"Postgres\", \"SQLite\"],
    \"Postgres\": [{\"input\": \"Connection string\", \"id\": \"conn\"}]}
2. reveals on check (shown while checked) or select/multi (shown once anything is selected):
   {\"check\": \"Enable retries\", \"id\": \"retries\", \"reveals\": [{\"slider\": \"Max attempts\", \"id\": \"attempts\", \"min\": 1, \"max\": 10}]}
3. when - cross-element conditions anywhere:
   {\"text\": \"Heads up: slow\", \"when\": \"selected(db, SQLite) && count(features) > 2\"}
   Conditions only see visible elements' values: an id hidden inside an untaken branch reads as unset (falsy/unselected/count 0), even if it has a default, so cross-branch references are safe as-is.

ELEMENTS: text, markdown, slider (min/max/step/default), check, input (placeholder/default/rows), select, multi, group, page. The element type is the JSON key and its value is the label; ids are auto-derived from labels when omitted (\"Enable Feature\" -> enable_feature) but explicit ids are more reliable for when-clauses.

WIZARD: for flows too long for one screen, make every top-level element a page: {\"page\": \"Step label\", \"elements\": [...]}. One page shows at a time (Back/Next, Submit on the last); a page's own when-clause can skip it based on earlier answers. Prefer one screen when it fits - pages cost the user a click each.

DO:
- Give options a description (\"options\": [{\"value\": \"X\", \"description\": \"tradeoff...\"}]) - descriptions render under each option and help the user decide
- Set defaults to your recommendation so the user confirms instead of choosing from scratch: select takes an option text, multi takes an array of option texts, inputs take prefilled text
- Name the submit button after the consequence (\"submit\": \"Deploy\")

DON'T:
- Don't add an 'Other' option - every select/multi automatically gets 'Other (please specify)' with a text input
- Don't reuse ids, and don't use reserved ids: status, button, error, _notes
- Don't use widget type names (text, slider, check, input, select, multi, group) as option values

VALIDATION: definitions are checked before anything is shown; errors come back with precise fixes (unknown keys, typo'd branch keys, bad when-refs). Fix and retry - the user never sees invalid popups.

RESULT: {\"status\": \"completed\", \"button\": \"submit\", \"<id>\": <value>, ...} or {\"status\": \"cancelled\"} (user pressed Escape - do not retry the same popup).
Value semantics:
- key absent -> that element was never shown (its branch wasn't taken)
- \"\" for input -> shown but left blank; null for select -> shown, nothing chosen; [] for multi -> shown, nothing checked; false for check -> shown, unchecked
- select returns the option text; multi returns an array of option texts
- slider returns a number (integer when whole)
- 'Other' chosen -> selection contains \"Other (please specify)\" plus \"<id>_other_text\" with the custom text
- \"_notes\" -> free-form note the user added; treat it as extra instructions
- \"_labels\" -> map of returned ids to their human-facing labels (self-documenting results)"
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
        assert!(schema["properties"]["submit"].is_object());
        assert!(schema["properties"]["notes"].is_object());
    }

    #[test]
    fn test_schema_examples_are_valid_definitions() {
        // The examples shown to calling models must parse and validate
        let schema = get_input_schema();
        let examples = schema["examples"].as_array().unwrap();
        assert!(!examples.is_empty());
        for example in examples {
            let def: popup_common::PopupDefinition =
                serde_json::from_value(example.clone()).expect("example must deserialize");
            let issues = popup_common::validate_definition(&def);
            assert!(
                issues.is_empty(),
                "schema example failed validation: {:?}",
                issues
            );
        }
    }
}
