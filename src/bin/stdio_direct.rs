//! MCP server for popup-mcp - enables AI assistants to create GUI popups

use anyhow::Result;
use mcpr::schema::json_rpc::{JSONRPCMessage, JSONRPCResponse};
use serde::Serialize;
use serde_json::Value;
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

fn main() -> Result<()> {
    // Set up logging to stderr
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .target(env_logger::Target::Stderr)
        .init();

    log::info!("Popup MCP server starting...");

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
                        JSONRPCResponse::new(
                            req.id,
                            serde_json::json!({
                                "tools": [
                                    {
                                        "name": "popup",
                                        "description": "Create a native GUI popup window using JSON structure. Elements require 'type' field. Example: {\"title\": \"Settings\", \"elements\": [{\"type\": \"text\", \"content\": \"Configure:\"}, {\"type\": \"slider\", \"label\": \"Volume\", \"min\": 0, \"max\": 100, \"default\": 50}, {\"type\": \"checkbox\", \"label\": \"Mute\", \"default\": false}, {\"type\": \"buttons\", \"labels\": [\"Save\", \"Cancel\"]}]}. Conditionals use simple strings: {\"type\": \"conditional\", \"condition\": \"ShowAdvanced\", \"elements\": [...]}",
                                        "inputSchema": {
                                            "type": "object",
                                            "properties": {
                                                "json": {
                                                    "type": "object",
                                                    "description": "Popup definition with 'title' and 'elements' array. Each element needs a 'type' field. Types: text (content), slider (label, min, max, default), checkbox (label, default), textbox (label, placeholder), choice (label, options), multiselect (label, options), buttons (labels), conditional (condition, elements), group (label, elements).",
                                                    "properties": {
                                                        "title": {
                                                            "type": "string",
                                                            "description": "Title of the popup window"
                                                        },
                                                        "elements": {
                                                            "type": "array",
                                                            "description": "Array of GUI elements"
                                                        }
                                                    },
                                                    "required": ["title", "elements"]
                                                }
                                            },
                                            "required": ["json"]
                                        }
                                    }
                                ]
                            }),
                        )
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

                        let result = match tool_name {
                            "popup" => {
                                let json_value = tool_args.get("json").cloned();
                                
                                log::info!("Showing popup with JSON: {:?}", json_value);
                                
                                // Check JSON value exists
                                if json_value.is_none() {
                                    log::error!("No JSON provided in tool arguments");
                                    error("Missing 'json' parameter in tool arguments")
                                } else {
                                    let json_str = serde_json::to_string(&json_value.unwrap()).unwrap_or_else(|e| {
                                        log::error!("Failed to serialize JSON: {}", e);
                                        "{}".to_string()
                                    });
                                    
                                    // Just spawn the popup-mcp binary and pipe the JSON
                                    // First try to find popup-mcp in the same directory as this binary
                                    let popup_path = std::env::current_exe()
                                        .ok()
                                        .and_then(|path| {
                                            log::info!("Current exe: {:?}", path);
                                            let dir = path.parent()?;
                                            log::info!("Parent dir: {:?}", dir);
                                            let popup = dir.join("popup-mcp");
                                            log::info!("Looking for popup binary at: {:?}", popup);
                                            if popup.exists() {
                                                log::info!("Found popup binary!");
                                                Some(popup)
                                            } else {
                                                log::warn!("Popup binary not found at {:?}", popup);
                                                // Try absolute path as fallback
                                                let fallback = std::path::PathBuf::from("/Users/inannamalick/claude_accessible/popup-mcp/target/release/popup-mcp");
                                                if fallback.exists() {
                                                    log::info!("Using fallback path: {:?}", fallback);
                                                    Some(fallback)
                                                } else {
                                                    None
                                                }
                                            }
                                        });
                                
                                    // Spawn popup binary directly without shell
                                    let child = if let Some(binary_path) = popup_path {
                                        log::info!("Spawning popup binary directly: {:?}", binary_path);
                                        std::process::Command::new(binary_path)
                                            .stdin(std::process::Stdio::piped())
                                            .stdout(std::process::Stdio::piped())
                                            .stderr(std::process::Stdio::piped())
                                            .spawn()
                                            .map_err(|e| format!("Failed to spawn popup subprocess: {}", e))
                                    } else {
                                        // Fallback to cargo run for development
                                        log::info!("Falling back to cargo run for popup");
                                        std::process::Command::new("cargo")
                                            .args(&["run", "--release", "--bin", "popup-mcp", "--quiet"])
                                            .current_dir(env!("CARGO_MANIFEST_DIR"))
                                            .stdin(std::process::Stdio::piped())
                                            .stdout(std::process::Stdio::piped())
                                            .stderr(std::process::Stdio::piped())
                                            .spawn()
                                            .map_err(|e| format!("Failed to spawn popup subprocess via cargo: {}", e))
                                    };
                                
                                    match child {
                                        Ok(mut child) => {
                                            log::info!("Subprocess spawned with PID: {:?}", child.id());
                                            
                                            // Write JSON to stdin
                                            match child.stdin.take() {
                                                Some(mut stdin) => {
                                                    use std::io::Write;
                                                    log::info!("Writing JSON to subprocess stdin...");
                                                    match stdin.write_all(json_str.as_bytes()) {
                                                    Ok(_) => {
                                                        // Close stdin to signal EOF
                                                        drop(stdin);
                                                        log::info!("JSON written successfully");
                                                        
                                                        // Wait for result
                                                        match child.wait_with_output() {
                                                    Ok(output) => {
                                                        let stdout_str = String::from_utf8_lossy(&output.stdout);
                                                        let stderr_str = String::from_utf8_lossy(&output.stderr);
                                                        
                                                        log::info!("Subprocess stdout: {}", stdout_str);
                                                        if !stderr_str.is_empty() {
                                                            log::info!("Subprocess stderr: {}", stderr_str);
                                                        }
                                                        
                                                        // Add small delay to ensure window system cleanup
                                                        std::thread::sleep(std::time::Duration::from_millis(100));
                                                        
                                                        if output.status.success() || !stdout_str.trim().is_empty() {
                                                            // Try to parse JSON even if exit code is non-zero
                                                            // (popup-mcp might exit with error code but still output JSON)
                                                            match serde_json::from_str::<Value>(&stdout_str) {
                                                                Ok(popup_result) => popup_result,
                                                                Err(e) => error(format!("Invalid JSON from popup: {}. Output was: {}", e, stdout_str))
                                                            }
                                                        } else {
                                                            error(format!("Popup process failed with status: {}. Stderr: {}", output.status, stderr_str))
                                                        }
                                            }
                                            Err(e) => error(format!("Failed to wait for popup: {}", e))
                                        }
                                                    }
                                                    Err(e) => error(format!("Failed to write JSON to subprocess: {}", e))
                                                }
                                            }
                                            None => error("Failed to get subprocess stdin")
                                        }
                                        }
                                        Err(e) => error(e)
                                    }
                                }
                            }
                            _ => error(format!("Unknown tool: {}", tool_name)),
                        };

                        JSONRPCResponse::new(
                            req.id,
                            serde_json::json!({
                                "content": [{
                                    "type": "text",
                                    "text": serde_json::to_string(&result)?
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
