//! MCP server module for popup-mcp - enables AI assistants to create GUI popups

use crate::templates;
use anyhow::Result;
use mcpr::schema::json_rpc::{JSONRPCMessage, JSONRPCResponse};
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::io::{self, BufRead, Write};

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

fn error(msg: impl std::fmt::Display) -> Value {
    serde_json::to_value(ErrorResponse {
        error: msg.to_string(),
    })
    .unwrap()
}

fn spawn_popup_subprocess(json_str: &str) -> Result<Value, String> {
    // Try to find the popup binary:
    // 1. Current executable (self-spawning)
    // 2. "popup" in PATH
    let binary_path = std::env::current_exe()
        .ok()
        .filter(|p| p.exists())
        .or_else(|| which::which("popup").ok())
        .ok_or_else(|| {
            "Could not find popup binary (not in PATH and current_exe failed)".to_string()
        })?;

    log::info!("Spawning popup binary with --stdin: {:?}", binary_path);

    // Spawn the popup binary with --stdin flag
    let mut child = std::process::Command::new(binary_path)
        .arg("--stdin")
        .env("WAYLAND_DISPLAY", "") // Force X11 on WSL
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn popup subprocess: {}", e))?;

    log::info!("Subprocess spawned with PID: {:?}", child.id());

    // Write JSON to stdin
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| "Failed to get subprocess stdin".to_string())?;

    use std::io::Write;
    log::info!("Writing JSON to subprocess stdin...");
    stdin
        .write_all(json_str.as_bytes())
        .map_err(|e| format!("Failed to write JSON to subprocess: {}", e))?;
    drop(stdin); // Close stdin to signal EOF

    // Wait for subprocess to complete and get output
    let output = child
        .wait_with_output()
        .map_err(|e| format!("Failed to wait for popup: {}", e))?;

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let stderr_str = String::from_utf8_lossy(&output.stderr);

    if !stderr_str.is_empty() {
        log::info!("Subprocess stderr: {}", stderr_str);
    }

    // Small delay to ensure window cleanup
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Parse the output
    if output.status.success() || !stdout_str.trim().is_empty() {
        serde_json::from_str::<Value>(&stdout_str)
            .map_err(|e| format!("Invalid JSON from popup: {}. Output was: {}", e, stdout_str))
    } else {
        Err(format!(
            "Popup process failed with status: {}. Stderr: {}",
            output.status, stderr_str
        ))
    }
}

pub struct ServerArgs {
    pub include_only: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
    pub list_templates: bool,
}

fn filter_templates(
    all_templates: Vec<templates::LoadedTemplate>,
    args: &ServerArgs,
) -> Result<Vec<templates::LoadedTemplate>> {
    let mut filtered = all_templates;

    // Apply include-only filter (takes precedence)
    if let Some(ref include_list) = args.include_only {
        log::info!("Filtering to include only: {:?}", include_list);

        // Check that all requested templates exist
        let available_names: std::collections::HashSet<&str> =
            filtered.iter().map(|t| t.config.name.as_str()).collect();

        for name in include_list {
            if !available_names.contains(name.as_str()) {
                return Err(anyhow::anyhow!(
                    "Template '{}' not found. Available templates: {}",
                    name,
                    available_names
                        .iter()
                        .copied()
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
        }

        filtered.retain(|template| include_list.contains(&template.config.name));
    }
    // Apply exclude filter (only if include-only not specified)
    else if let Some(ref exclude_list) = args.exclude {
        log::info!("Excluding templates: {:?}", exclude_list);
        filtered.retain(|template| !exclude_list.contains(&template.config.name));
    }

    log::info!("Using {} filtered templates", filtered.len());
    for template in &filtered {
        log::info!(
            "  - {}: {}",
            template.config.name,
            template.config.description
        );
    }

    Ok(filtered)
}

pub fn run(args: ServerArgs) -> Result<()> {
    // Set up logging to stderr
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .target(env_logger::Target::Stderr)
        .init();

    log::info!("Popup MCP server starting...");

    // Load templates from config
    let all_templates = match templates::load_templates() {
        Ok(templates) => {
            log::info!("Loaded {} templates", templates.len());
            for template in &templates {
                log::info!(
                    "  - {}: {}",
                    template.config.name,
                    template.config.description
                );
            }
            templates
        }
        Err(e) => {
            log::warn!(
                "Failed to load templates: {}. Continuing without templates.",
                e
            );
            Vec::new()
        }
    };

    // Filter templates based on CLI arguments
    let loaded_templates = filter_templates(all_templates, &args)?;

    // Handle --list-templates flag (after filtering)
    if args.list_templates {
        println!("Available templates:");
        for template in &loaded_templates {
            println!(
                "  - {}: {}",
                template.config.name, template.config.description
            );
        }
        return Ok(());
    }

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    // Read messages line by line
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        log::debug!("Received: {}", line);

        // Parse the JSON-RPC message
        match serde_json::from_str::<JSONRPCMessage>(&line) {
            Ok(JSONRPCMessage::Request(req)) => {
                log::info!("Request: {} (id: {:?})", req.method, req.id);

                let response = match req.method.as_str() {
                    "initialize" => {
                        log::debug!("Handling initialization");
                        JSONRPCResponse::new(
                            req.id,
                            serde_json::json!({
                                "protocolVersion": "2024-11-05",
                                "capabilities": {
                                    "tools": {},
                                    "gui_popups": {
                                        "description": "Create native GUI popups using JSON structure for precise control.",
                                        "features": {
                                            "json_based": "Clean JSON structure for defining GUI elements",
                                            "conditional_ui": "Dynamic interfaces that show/hide elements based on user selections",
                                            "rich_widgets": "Sliders with percentage display, multiselect with All/None buttons, text fields with character count",
                                            "keyboard_nav": "Full keyboard support with Tab/Arrow/Escape navigation"
                                        },
                                        "element_types": {
                                            "text": "Static text display",
                                            "slider": "Numeric range selector with min/max/default",
                                            "checkbox": "Boolean toggle with default state",
                                            "choice": "Single selection from options",
                                            "multiselect": "Multiple selection from options",
                                            "textbox": "Text input with optional placeholder",
                                            "buttons": "Action buttons",
                                            "conditional": "Show/hide elements based on conditions",
                                            "group": "Group related elements"
                                        },
                                        "version": "0.4.0"
                                    }
                                },
                                "serverInfo": {
                                    "name": "popup-mcp",
                                    "version": "0.4.0",
                                    "description": "Native GUI popup server for MCP. Create interactive forms, settings dialogs, and confirmation prompts using JSON structure. Features conditional UI, rich widgets, and full keyboard navigation."
                                }
                            }),
                        )
                    }
                    "tools/list" => {
                        log::debug!("Handling tools/list");

                        // Build tools array starting with the main popup tool
                        let mut tools = vec![crate::schema::get_popup_tool_schema()];

                        // Add template tools
                        for template in &loaded_templates {
                            let mut description = template.config.description.clone();

                            // Add examples if present
                            if !template.config.examples.is_empty() {
                                description.push_str("\n\nExamples:\n");
                                for example in &template.config.examples {
                                    description.push_str(&format!("- {}\n", example));
                                }
                            }

                            // Add notes if present
                            if let Some(notes) = &template.config.notes {
                                description.push_str(&format!("\n\nNotes: {}", notes));
                            }

                            tools.push(serde_json::json!({
                                "name": template.config.name,
                                "description": description,
                                "inputSchema": templates::generate_tool_schema(&template.config)
                            }));
                        }

                        JSONRPCResponse::new(req.id, serde_json::json!({ "tools": tools }))
                    }
                    "resources/list" => {
                        log::debug!("Handling resources/list");
                        JSONRPCResponse::new(req.id, serde_json::json!({"resources": []}))
                    }
                    "prompts/list" => {
                        log::debug!("Handling prompts/list");
                        JSONRPCResponse::new(req.id, serde_json::json!({"prompts": []}))
                    }
                    "tools/call" => {
                        log::debug!("Tool call params: {:?}", req.params);

                        let params = req.params.unwrap_or(Value::Null);
                        let tool_name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
                        let tool_args = params.get("arguments").cloned().unwrap_or(Value::Null);

                        let result = if tool_name == "popup" {
                            // tool_args IS the popup definition (title, elements, etc.)
                            log::info!("Showing popup with args: {:?}", tool_args);

                            let json_str = serde_json::to_string(&tool_args).unwrap_or_else(|e| {
                                log::error!("Failed to serialize JSON: {}", e);
                                "{}".to_string()
                            });

                            // Spawn popup subprocess and get result
                            match spawn_popup_subprocess(&json_str) {
                                Ok(popup_result) => popup_result,
                                Err(e) => error(e),
                            }
                        } else {
                            // Check if it's a template tool
                            if let Some(template) =
                                loaded_templates.iter().find(|t| t.config.name == tool_name)
                            {
                                log::info!("Invoking template: {}", tool_name);

                                // Convert tool_args to HashMap<String, Value>
                                let params = if let Some(obj) = tool_args.as_object() {
                                    obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
                                } else {
                                    HashMap::new()
                                };

                                // Instantiate the template
                                match templates::instantiate_template(template, &params) {
                                    Ok(popup_def) => {
                                        // Convert popup definition to JSON and run it
                                        let json_str = serde_json::to_string(&popup_def)
                                            .unwrap_or_else(|e| {
                                                log::error!(
                                                    "Failed to serialize popup definition: {}",
                                                    e
                                                );
                                                "{}".to_string()
                                            });

                                        // Spawn popup subprocess and get result
                                        match spawn_popup_subprocess(&json_str) {
                                            Ok(popup_result) => popup_result,
                                            Err(e) => error(e),
                                        }
                                    }
                                    Err(e) => {
                                        error(format!("Failed to instantiate template: {}", e))
                                    }
                                }
                            } else {
                                error(format!("Unknown tool: {}", tool_name))
                            }
                        };

                        JSONRPCResponse::new(
                            req.id,
                            serde_json::json!({
                                "content": [{
                                    "type": "text",
                                    "text": serde_json::to_string_pretty(&result)?
                                }]
                            }),
                        )
                    }
                    _ => {
                        log::debug!("Unknown method: {}", req.method);
                        continue;
                    }
                };

                // Send response
                let response_str = serde_json::to_string(&JSONRPCMessage::Response(response))?;
                stdout.write_all(response_str.as_bytes())?;
                stdout.write_all(b"\n")?;
                stdout.flush()?;
                log::debug!("Sent response for {}", req.method);
            }
            Ok(msg) => {
                log::debug!("Other message type: {:?}", msg);
            }
            Err(e) => {
                log::error!("Parse error: {}", e);
            }
        }
    }

    log::info!("Server exiting");
    Ok(())
}
