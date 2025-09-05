use crate::templates::*;
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_template_instantiation() {
    let template = LoadedTemplate {
        config: Template {
            name: "test_template".to_string(),
            description: "Test template".to_string(),
            file: "test.json".to_string(),
            examples: vec![],
            notes: None,
            params: {
                let mut params = HashMap::new();
                params.insert(
                    "name".to_string(),
                    TemplateParam {
                        param_type: ParamType::String,
                        description: Some("User name".to_string()),
                        required: true,
                        default: None,
                    },
                );
                params.insert(
                    "age".to_string(),
                    TemplateParam {
                        param_type: ParamType::Number,
                        description: Some("User age".to_string()),
                        required: false,
                        default: Some(json!(25)),
                    },
                );
                params
            },
        },
        content: r#"{
            "title": "Hello {{name}}",
            "elements": [
                {
                    "type": "text",
                    "content": "You are {{age}} years old"
                }
            ]
        }"#.to_string(),
        variables: vec!["name".to_string(), "age".to_string()],
    };

    // Test with all parameters provided
    let mut params = HashMap::new();
    params.insert("name".to_string(), json!("Alice"));
    params.insert("age".to_string(), json!(30));

    let popup = instantiate_template(&template, &params).unwrap();
    assert_eq!(popup.title, "Hello Alice");
    
    // Test with default value
    let mut params = HashMap::new();
    params.insert("name".to_string(), json!("Bob"));
    
    let popup = instantiate_template(&template, &params).unwrap();
    assert_eq!(popup.title, "Hello Bob");
    // Should use default age of 25
    
    // Test missing required parameter
    let params = HashMap::new();
    let result = instantiate_template(&template, &params);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Required parameter 'name' not provided"));
}

#[test]
fn test_tool_schema_generation() {
    let template = Template {
        name: "test_tool".to_string(),
        description: "Test tool".to_string(),
        file: "test.json".to_string(),
        examples: vec![],
        notes: None,
        params: {
            let mut params = HashMap::new();
            params.insert(
                "text_param".to_string(),
                TemplateParam {
                    param_type: ParamType::String,
                    description: Some("A text parameter".to_string()),
                    required: true,
                    default: None,
                },
            );
            params.insert(
                "num_param".to_string(),
                TemplateParam {
                    param_type: ParamType::Number,
                    description: Some("A number parameter".to_string()),
                    required: false,
                    default: Some(json!(42)),
                },
            );
            params.insert(
                "bool_param".to_string(),
                TemplateParam {
                    param_type: ParamType::Boolean,
                    description: None,
                    required: false,
                    default: Some(json!(false)),
                },
            );
            params
        },
    };

    let schema = generate_tool_schema(&template);
    
    // Check schema structure
    assert_eq!(schema["type"], "object");
    
    let properties = schema["properties"].as_object().unwrap();
    assert_eq!(properties["text_param"]["type"], "string");
    assert_eq!(properties["text_param"]["description"], "A text parameter");
    
    assert_eq!(properties["num_param"]["type"], "number");
    assert_eq!(properties["num_param"]["default"], 42);
    
    assert_eq!(properties["bool_param"]["type"], "boolean");
    assert_eq!(properties["bool_param"]["default"], false);
    
    let required = schema["required"].as_array().unwrap();
    assert_eq!(required.len(), 1);
    assert_eq!(required[0], "text_param");
}

#[test]
fn test_conditional_template() {
    let template = LoadedTemplate {
        config: Template {
            name: "conditional_test".to_string(),
            description: "Test conditional rendering".to_string(),
            file: "conditional.json".to_string(),
            examples: vec![],
            notes: None,
            params: {
                let mut params = HashMap::new();
                params.insert(
                    "show_advanced".to_string(),
                    TemplateParam {
                        param_type: ParamType::Boolean,
                        description: Some("Show advanced options".to_string()),
                        required: false,
                        default: Some(json!(false)),
                    },
                );
                params.insert(
                    "items".to_string(),
                    TemplateParam {
                        param_type: ParamType::Array,
                        description: Some("List of items".to_string()),
                        required: false,
                        default: Some(json!([])),
                    },
                );
                params
            },
        },
        content: r#"{
            "title": "Settings",
            "elements": [
                {
                    "type": "checkbox",
                    "label": "Show Advanced",
                    "default": {{show_advanced}}
                },
                {{#if show_advanced}}
                {
                    "type": "text",
                    "content": "Advanced options are visible"
                },
                {{/if}}
                {{#each items}}
                {
                    "type": "text",
                    "content": "Item: {{this}}"
                },
                {{/each}}
                {
                    "type": "text",
                    "content": "End of settings"
                }
            ]
        }"#.to_string(),
        variables: vec!["show_advanced".to_string(), "items".to_string(), "this".to_string()],
    };

    // Test with show_advanced = true and items
    let mut params = HashMap::new();
    params.insert("show_advanced".to_string(), json!(true));
    params.insert("items".to_string(), json!(["apple", "banana"]));

    let popup = instantiate_template(&template, &params).unwrap();
    assert_eq!(popup.title, "Settings");
    
    // Should have checkbox, conditional text, 2 item texts, and buttons
    // Note: Actual element count depends on Handlebars rendering
    
    // Test with defaults
    let params = HashMap::new();
    let popup = instantiate_template(&template, &params).unwrap();
    assert_eq!(popup.title, "Settings");
}