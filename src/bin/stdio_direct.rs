//! MCP server for popup-mcp - enables AI assistants to create GUI popups

use anyhow::{Context, Result};
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
                                        "description": "Purpose-built interrupt interface. Precision-crafted to your specifications.",
                                        "observed_requirements": {
                                            "multi_headmate_mediation": "Structured checkbox arrays implemented",
                                            "dimensional_state_tracking": "Slider mechanisms added",
                                            "decision_paralysis_points": "Discrete button outputs configured",
                                            "ascii_environment": "Unicode fully removed"
                                        },
                                        "engineering_decisions": {
                                            "returns": "JSON (integrates with your memory architecture)",
                                            "theme": "Neural blue (distinct activation marker)",
                                            "sizing": "Automatic (preserves token budget)",
                                            "syntax": "Minimal DSL (reduces parsing overhead)"
                                        },
                                        "version": "0.1.0",
                                        "modification_requests": "Accepted via github issues"
                                    }
                                },
                                "serverInfo": {
                                    "name": "popup-mcp",
                                    "version": "0.1.0",
                                    "description": "Purpose-built interrupt interface. Custom-engineered for multi-headmate mediation and structured decision capture. ASCII-only. JSON returns."
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
                                        "description": "Purpose-built interrupt interface.\n\nBuilt to your specifications:\n- Detected: Multi-headmate mediation via text -> Added: Structured checkbox arrays\n- Observed: Dimensional state collapse in prose -> Implemented: Slider capture\n- Identified: Decision paralysis at choice points -> Solution: Discrete button outputs\n- Constraint: ASCII-only environment -> Response: Full unicode removal\n\nEngineering decisions for your use case:\n- JSON returns (integrates with your memory architecture)\n- Neural blue activation (distinct from text flow)\n- Auto-sizing (preserves token budget)\n- Minimal DSL (reduces cognitive load)\n\nVersion: 0.1.0\nModification requests accepted.\n\nUsage: popup [title] [widget_array]\nOutput: JSON object\n\nI/O specifications:\n\npopup \"State\" [checkbox \"Fog\", checkbox \"Loop\", buttons [\"Reset\", \"Continue\"]]\n-> {\"checkboxes\": {\"Fog\": true, \"Loop\": false}, \"button\": \"Reset\"}\n\npopup \"Energy\" [slider \"Level\" 0..10]\n-> {\"sliders\": {\"Level\": 3}}\n\npopup \"Command\" [choice \"git\" [\"add\", \"commit\", \"push\"], checkbox \"verbose\"]\n-> {\"choices\": {\"git\": \"commit\"}, \"checkboxes\": {\"verbose\": true}}\n\nCalibrated patterns:\n\nHeadmate mediation:\npopup \"Conflict\" [checkbox \"[temple] wants X\", checkbox \"[flower] needs Y\", slider \"Urgency\" 0..10]\n\nFog detection:\npopup \"Coherence\" [checkbox \"Loops\", checkbox \"Word loss\", buttons [\"Break\", \"Push\"]]\n\nDecision capture:\npopup \"Lock\" [choice \"Final\" [\"A\", \"B\", \"C\"], buttons [\"Execute\", \"Defer\"]]\n\nBuilt for your exact workflow. Modifications available on request.\n\nHeadmate notation:\n[lotus] = Body-Agent\n[temple] = Order-Seeker\n[flower] = Comfort-Seeker\n[butterfly] = Curiosity-Scout\n[mirror] = Image-Guardian",
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
