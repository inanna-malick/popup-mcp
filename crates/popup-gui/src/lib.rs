pub mod gui;
pub mod json_parser;
pub mod mcp_server;
pub mod schema;
pub mod templates;
pub mod theme;
pub mod transform;

#[cfg(test)]
mod tests;

// Re-export from popup-common
pub use popup_common::{Element, ElementValue, PopupDefinition, PopupResult, PopupState};

// Re-export from this crate
pub use gui::render_popup;
pub use json_parser::{
    detect_popup_format, parse_popup_from_direct, parse_popup_from_mcp_wrapper, parse_popup_json,
    parse_popup_json_value, validate_popup_json,
};
pub use schema::{get_input_schema, get_popup_tool_schema, get_schema_description};
pub use transform::inject_other_options;
