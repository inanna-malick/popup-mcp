pub mod gui;
pub mod json_parser;
pub mod mcp_server;
pub mod schema;
pub mod templates;
pub mod theme;

#[cfg(test)]
mod tests;

// Re-export from popup-common
pub use popup_common::{
    ComparisonOp, Condition, Element, ElementValue, PopupDefinition, PopupResult, PopupState,
};

// Re-export from this crate
pub use gui::render_popup;
#[cfg(feature = "async")]
pub use gui::render_popup_sequential;
pub use json_parser::{
    detect_popup_format, parse_popup_from_direct, parse_popup_from_mcp_wrapper, parse_popup_json,
    parse_popup_json_value, validate_popup_json,
};
pub use schema::{
    get_input_schema, get_popup_tool_schema, get_schema_description, get_simple_input_schema,
    get_simple_popup_tool_schema,
};
