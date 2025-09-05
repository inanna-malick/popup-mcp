use anyhow::{anyhow, Result};
use handlebars::Handlebars;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::json_parser::parse_popup_json;
use crate::models::PopupDefinition;

/// Configuration for all templates
#[derive(Debug, Deserialize)]
pub struct TemplateConfig {
    #[serde(rename = "template")]
    pub templates: Vec<Template>,
}

/// A single template definition
#[derive(Debug, Deserialize, Clone)]
pub struct Template {
    /// Name of the template (becomes MCP tool name, must be valid identifier)
    pub name: String,
    /// Description for the MCP tool
    pub description: String,
    /// Path to template file relative to config directory
    pub file: String,
    /// Parameter definitions
    #[serde(default)]
    pub params: HashMap<String, TemplateParam>,
    /// Usage examples (optional)
    #[serde(default)]
    pub examples: Vec<String>,
    /// Additional notes about the template (optional)
    #[serde(default)]
    pub notes: Option<String>,
}

/// Definition of a template parameter
#[derive(Debug, Deserialize, Clone)]
pub struct TemplateParam {
    /// Type of the parameter
    #[serde(rename = "type")]
    pub param_type: ParamType,
    /// Description for the parameter
    #[serde(default)]
    pub description: Option<String>,
    /// Whether the parameter is required
    #[serde(default)]
    pub required: bool,
    /// Default value if not provided
    #[serde(default)]
    pub default: Option<Value>,
}

/// Supported parameter types
#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ParamType {
    String,
    Number,
    Boolean,
    Array,
}

/// A loaded template with its content
pub struct LoadedTemplate {
    pub config: Template,
    pub content: String,
    pub variables: Vec<String>, // List of {{var}} references found
}

/// Load templates from the user's config directory
pub fn load_templates() -> Result<Vec<LoadedTemplate>> {
    let home = std::env::var("HOME").map_err(|_| anyhow!("HOME environment variable not set"))?;
    let config_dir = PathBuf::from(home).join(".config").join("popup-mcp");

    log::debug!("Loading templates from: {:?}", config_dir);

    if !config_dir.exists() {
        log::debug!("Config directory does not exist: {:?}", config_dir);
        return Ok(Vec::new());
    }

    let config_path = config_dir.join("popup.toml");
    if !config_path.exists() {
        log::debug!("Config file does not exist: {:?}", config_path);
        return Ok(Vec::new());
    }

    log::info!("Loading template config from: {:?}", config_path);

    // Load the TOML config
    let config_str = fs::read_to_string(&config_path)?;
    let config: TemplateConfig =
        toml::from_str(&config_str).map_err(|e| anyhow!("Failed to parse popup.toml: {}", e))?;

    // Load and validate each template
    let mut loaded_templates = Vec::new();
    for template in config.templates {
        let template_path = config_dir.join(&template.file);

        // Validate template name is a valid identifier (no spaces, etc.)
        if !is_valid_tool_name(&template.name) {
            return Err(anyhow!(
                "Template name '{}' is not valid. Use only letters, numbers, and underscores.",
                template.name
            ));
        }

        // Load template content
        let content = fs::read_to_string(&template_path)
            .map_err(|e| anyhow!("Failed to load template {}: {}", template.file, e))?;

        // Extract variables from template
        let variables = extract_template_variables(&content);

        // Validate all variables have corresponding params
        for var in &variables {
            if !template.params.contains_key(var) {
                return Err(anyhow!(
                    "Template '{}' references variable '{{{{{}}}}}' but no parameter is defined",
                    template.name,
                    var
                ));
            }
        }

        loaded_templates.push(LoadedTemplate {
            config: template,
            content,
            variables,
        });
    }

    Ok(loaded_templates)
}

/// Check if a string is a valid MCP tool name
fn is_valid_tool_name(name: &str) -> bool {
    !name.is_empty()
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        && name.chars().next().unwrap().is_ascii_alphabetic()
}

/// Extract all {{variable}} references from a Handlebars template
fn extract_template_variables(template: &str) -> Vec<String> {
    let mut variables = Vec::new();
    let mut in_var = false;
    let mut var_name = String::new();
    let mut chars = template.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '{' && chars.peek() == Some(&'{') {
            chars.next(); // consume second {
            in_var = true;
            var_name.clear();
        } else if in_var && ch == '}' && chars.peek() == Some(&'}') {
            chars.next(); // consume second }
            in_var = false;

            // Parse the variable name (might have whitespace or be in #if, #each, etc.)
            let trimmed = var_name.trim();
            if !trimmed.is_empty() && !trimmed.starts_with('#') && !trimmed.starts_with('/') {
                // Simple variable reference
                let var = trimmed.split_whitespace().next().unwrap_or("");
                if !var.is_empty() && !variables.contains(&var.to_string()) {
                    variables.push(var.to_string());
                }
            }
        } else if in_var {
            var_name.push(ch);
        }
    }

    variables
}

/// Instantiate a template with given parameters
pub fn instantiate_template(
    template: &LoadedTemplate,
    params: &HashMap<String, Value>,
) -> Result<PopupDefinition> {
    // Prepare parameters with defaults
    let mut full_params = HashMap::new();

    for (name, param_def) in &template.config.params {
        if let Some(value) = params.get(name) {
            // Use provided value
            full_params.insert(name.clone(), value.clone());
        } else if let Some(default) = &param_def.default {
            // Use default
            full_params.insert(name.clone(), default.clone());
        } else if param_def.required {
            // Required parameter missing
            return Err(anyhow!("Required parameter '{}' not provided", name));
        }
    }

    // Render template with Handlebars
    let mut handlebars = Handlebars::new();

    // Disable HTML escaping since we're generating JSON
    handlebars.set_strict_mode(false);
    handlebars.register_escape_fn(handlebars::no_escape);

    let json_str = handlebars
        .render_template(&template.content, &full_params)
        .map_err(|e| anyhow!("Failed to render template: {}", e))?;

    // Parse the generated JSON
    parse_popup_json(&json_str).map_err(|e| anyhow!("Generated invalid JSON from template: {}", e))
}

/// Generate MCP tool schema for a template
pub fn generate_tool_schema(template: &Template) -> Value {
    let mut properties = serde_json::Map::new();
    let mut required = Vec::new();

    for (name, param) in &template.params {
        let mut prop = serde_json::Map::new();

        // Set type
        prop.insert(
            "type".to_string(),
            Value::String(
                match param.param_type {
                    ParamType::String => "string",
                    ParamType::Number => "number",
                    ParamType::Boolean => "boolean",
                    ParamType::Array => "array",
                }
                .to_string(),
            ),
        );

        // Add description if present
        if let Some(desc) = &param.description {
            prop.insert("description".to_string(), Value::String(desc.clone()));
        }

        // Add default if present
        if let Some(default) = &param.default {
            prop.insert("default".to_string(), default.clone());
        }

        properties.insert(name.clone(), Value::Object(prop));

        if param.required {
            required.push(Value::String(name.clone()));
        }
    }

    serde_json::json!({
        "type": "object",
        "properties": properties,
        "required": required
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_variables() {
        let template = r#"
            Hello {{name}}!
            Your age is {{age}}.
            {{#if premium}}
                Premium user
            {{/if}}
            Items: {{#each items}}{{this}}{{/each}}
        "#;

        let vars = extract_template_variables(template);
        assert!(vars.contains(&"name".to_string()));
        assert!(vars.contains(&"age".to_string()));
        assert!(vars.contains(&"premium".to_string()));
        assert!(vars.contains(&"items".to_string()));
        assert!(vars.contains(&"this".to_string()));
    }

    #[test]
    fn test_valid_tool_names() {
        assert!(is_valid_tool_name("confirm_delete"));
        assert!(is_valid_tool_name("quickSettings"));
        assert!(is_valid_tool_name("tool123"));

        assert!(!is_valid_tool_name("confirm-delete")); // hyphen
        assert!(!is_valid_tool_name("confirm delete")); // space
        assert!(!is_valid_tool_name("123tool")); // starts with number
        assert!(!is_valid_tool_name("")); // empty
    }
}
