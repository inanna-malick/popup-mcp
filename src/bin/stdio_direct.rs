//! MCP server for popup-mcp - enables AI assistants to create GUI popups

use anyhow::{Context, Result};
use mcpr::schema::json_rpc::{JSONRPCMessage, JSONRPCResponse};
use popup_mcp::{parse_popup_dsl, render_popup};
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
                                        "description": "Create native GUI popup windows using a simple DSL",
                                        "features": [
                                            "Text labels and explanations",
                                            "Sliders with min/max/default values",
                                            "Checkboxes with labels",
                                            "Radio button groups (choices)",
                                            "Text input fields with placeholders",
                                            "Multiline text areas",
                                            "Grouped sections",
                                            "Multiple action buttons"
                                        ]
                                    }
                                },
                                "serverInfo": {
                                    "name": "popup-mcp",
                                    "version": "0.1.0",
                                    "description": "MCP server for creating native GUI popups"
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
                                        "name": "popup_show",
                                        "description": "Display an interactive popup window using DSL and return user selections as JSON",
                                        "inputSchema": {
                                            "type": "object",
                                            "properties": {
                                                "dsl": {
                                                    "type": "string",
                                                    "description": "Popup DSL expression defining the UI elements"
                                                }
                                            },
                                            "required": ["dsl"]
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
                            "popup_show" => {
                                let dsl = tool_args.get("dsl").and_then(|d| d.as_str()).unwrap_or("");
                                
                                log::info!("Showing popup with DSL: {}", dsl);
                                
                                match parse_popup_dsl(dsl) {
                                    Ok(definition) => {
                                        match render_popup(definition) {
                                            Ok(popup_result) => {
                                                serde_json::to_value(&popup_result).unwrap()
                                            }
                                            Err(e) => error(format!("Failed to render popup: {}", e))
                                        }
                                    }
                                    Err(e) => error(format!("Failed to parse DSL: {}", e))
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
