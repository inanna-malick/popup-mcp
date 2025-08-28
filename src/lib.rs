pub mod gui;
pub mod json_parser;
pub mod models;
pub mod templates;
pub mod theme;
pub mod mcp_server;

#[cfg(test)]
mod tests;

pub use gui::render_popup;
pub use json_parser::{parse_popup_json, parse_popup_json_value};
pub use models::{PopupDefinition, PopupResult};
