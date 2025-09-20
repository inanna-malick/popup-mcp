pub mod gui;
pub mod json_parser;
pub mod mcp_server;
pub mod models;
pub mod schema;
pub mod templates;
pub mod theme;

#[cfg(test)]
mod tests;

pub use gui::render_popup;
#[cfg(feature = "async")]
pub use gui::render_popup_sequential;
pub use json_parser::{parse_popup_json, parse_popup_json_value};
pub use models::{
    ComparisonOp, Condition, Element, ElementValue, PopupDefinition, PopupResult, PopupState,
};
pub use schema::{
    get_input_schema, get_popup_tool_schema, get_schema_description, get_simple_input_schema,
    get_simple_popup_tool_schema,
};
