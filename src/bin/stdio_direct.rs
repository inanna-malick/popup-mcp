//! MCP server for popup-mcp - enables AI assistants to create GUI popups

use anyhow::{Context, Result};
use mcpr::schema::json_rpc::{JSONRPCMessage, JSONRPCResponse};
use serde::Serialize;
use serde_json::Value;
use std::io::{self, BufRead, Write};

// Include the formal grammar specification
const DSL_GRAMMAR: &str = include_str!("../../src/popup.pest");

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
                                        "description": "Purpose-built interrupt interface with conditional UI support.",
                                        "features": {
                                            "conditionals": {
                                                "description": "Dynamic UI that adapts based on user selections",
                                                "syntax": [
                                                    "if checked(\"checkbox_name\") [...]",
                                                    "if selected(\"choice_name\", \"option_value\") [...]",
                                                    "if count(\"multiselect_name\") > N [...]"
                                                ],
                                                "enables": "Complex decision trees, state-dependent interfaces, adaptive workflows"
                                            },
                                            "widgets": {
                                                "multiselect": "Multiple checkbox selection with count conditions",
                                                "existing": "text, slider, checkbox, choice, textbox, group, buttons"
                                            },
                                            "observed_requirements": {
                                                "multi_headmate_mediation": "Structured checkbox arrays + conditional flows",
                                                "dimensional_state_tracking": "Slider mechanisms with conditional responses",
                                                "decision_paralysis_points": "Discrete button outputs + adaptive guidance",
                                                "ascii_environment": "Unicode fully removed"
                                            },
                                            "expression_language": {
                                                "format": "pest",
                                                "schema": DSL_GRAMMAR
                                            },
                                            "engineering_decisions": {
                                                "returns": "JSON (integrates with your memory architecture)",
                                                "theme": "Neural blue (distinct activation marker)",
                                                "sizing": "Auto-fit content (no scrollbars)",
                                                "syntax": "Minimal DSL with conditionals (reduces cognitive load)"
                                            }
                                        },
                                        "version": "0.2.1",
                                        "modification_requests": "Accepted via github issues"
                                    }
                                },
                                "serverInfo": {
                                    "name": "popup-mcp",
                                    "version": "0.2.1",
                                    "description": "Purpose-built interrupt interface with conditional UI support. Custom-engineered for multi-headmate mediation and adaptive decision flows. Dynamic interfaces that change based on user selections."
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
                                        "description": "Purpose-built interrupt interface with conditional UI support.\n\n**NEW: Conditional Elements**\n- if checked(\"name\") [...] - Show content when checkbox checked\n- if selected(\"choice\", \"value\") [...] - Show content for specific choice\n- if count(\"multiselect\") > N [...] - Show content based on selection count\n\n**Widgets**\n- text, slider, checkbox, choice, textbox, group, buttons\n- NEW: multiselect \"label\" [options...] - Multiple checkbox selection\n\n**Features**\n- Dynamic UI adapts based on user selections\n- Complex decision trees and state-dependent interfaces\n- Neural blue theme with auto-sizing (no scrollbars)\n- JSON output integrates with memory architecture\n- **Auto button validation** - Every popup guaranteed to have at least one button\n\n**CRITICAL: Button Requirement**\nEvery popup MUST have at least one button. If no buttons defined, parser automatically adds `buttons [\"Continue\"]` with warning. Always prefer explicit buttons over automatic fallback.\n\n**Usage Examples**\n\nAdaptive state check:\npopup \"State\" [\n    choice \"Mode\" [\"Stuck\", \"Conflicted\", \"Exploring\"]\n    if selected(\"Mode\", \"Stuck\") [\n        checkbox \"Can move?\" default = false\n        if checked(\"Can move?\") [\n            text \"Great! Take one tiny step\"\n        ]\n    ]\n    buttons [\"Execute\", \"Defer\"]  // REQUIRED\n]\n\nHeadmate mediation:\npopup \"Mediation\" [\n    multiselect \"Active\" [\"[temple]\", \"[flower]\", \"[butterfly]\"]\n    if count(\"Active\") > 2 [\n        text \"Complex negotiation needed\"\n        slider \"Tension\" 0..10 default = 5\n    ]\n    buttons [\"Apply\", \"Cancel\"]  // REQUIRED\n]\n\nConditional guidance:\npopup \"Check\" [\n    checkbox \"Fog present\" default = false\n    if checked(\"Fog present\") [\n        text \">>> FOG PROTOCOL <<<\"\n        checkbox \"Water nearby?\" default = false\n    ]\n    buttons [\"OK\"]  // REQUIRED - even simple popups need buttons\n]\n\n**Output Examples**\n\nBasic: {\"checkboxes\": {\"Fog present\": true}, \"button\": \"OK\"}\nMultiselect: {\"Active\": [0, 2], \"button\": \"Continue\"} // Indices of selected items\nMixed: {\"Mode\": 0, \"Can move?\": true, \"button\": \"Execute\"}\n\nVersion: 0.2.1",
                                        "expression_language": {
                                            "schema": DSL_GRAMMAR
                                        },
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
                                
                                // Just spawn the popup-mcp binary and pipe the DSL
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
                                let mut child = if let Some(binary_path) = popup_path {
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
                                        
                                        // Write DSL to stdin
                                        match child.stdin.take() {
                                            Some(mut stdin) => {
                                                use std::io::Write;
                                                log::info!("Writing DSL to subprocess stdin...");
                                                match stdin.write_all(dsl.as_bytes()) {
                                                    Ok(_) => {
                                                        // Close stdin to signal EOF
                                                        drop(stdin);
                                                        log::info!("DSL written successfully");
                                                        
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
                                                    Err(e) => error(format!("Failed to write DSL to subprocess: {}", e))
                                                }
                                            }
                                            None => error("Failed to get subprocess stdin")
                                        }
                                    }
                                    Err(e) => error(e)
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
