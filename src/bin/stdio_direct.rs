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
                                        "description": "Create native GUI popups with a simple, natural DSL. Smart widget detection automatically creates the right controls based on your values.",
                                        "features": {
                                            "smart_detection": "Parser automatically detects widget types from value patterns",
                                            "conditional_ui": "Dynamic interfaces that show/hide elements based on user selections",
                                            "rich_widgets": "Sliders with percentage display, multiselect with All/None buttons, text fields with character count",
                                            "keyboard_nav": "Full keyboard support with Tab/Arrow/Escape navigation"
                                        },
                                        "widget_patterns": {
                                            "slider": "Label: 0-100 = 50 (range with optional default)",
                                            "checkbox": "Label: yes/no/true/false/✓/☐",
                                            "choice": "Label: Option1 | Option2 | Option3",
                                            "multiselect": "Label: [Option1, Option2, Option3]",
                                            "textbox": "Label: @placeholder text",
                                            "buttons": "[Save | Cancel] or 'Save or Cancel' or '→ Continue'"
                                        },
                                        "conditional_syntax": {
                                            "format": "[if condition] { elements }",
                                            "examples": [
                                                "[if Show advanced] { Debug: 0-10 }",
                                                "[if Theme = Dark] { Contrast: high | normal }",
                                                "[if Tags > 2] { Priority: @Set priority }"
                                            ]
                                        },
                                        "version": "0.3.0"
                                    }
                                },
                                "serverInfo": {
                                    "name": "popup-mcp",
                                    "version": "0.3.0",
                                    "description": "Native GUI popup server for MCP. Create interactive forms, settings dialogs, and confirmation prompts using a simple DSL. Features smart widget detection, conditional UI, and full keyboard navigation."
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
                                        "description": "Create a native GUI popup window. The DSL uses natural syntax with smart widget detection. Example: 'Settings\nVolume: 0-100 = 75\nTheme: Light | Dark\n[Save | Cancel]' creates a popup with a slider, choice, and buttons. Conditional UI supported with [if condition] { elements } syntax.",
                                        "inputSchema": {
                                            "type": "object",
                                            "properties": {
                                                "dsl": {
                                                    "type": "string",
                                                    "description": "Popup definition in DSL format. One element per line. Patterns: 'Label: 0-100' (slider), 'Label: yes/no' (checkbox), 'Label: A | B | C' (choice), 'Label: [A, B, C]' (multiselect), 'Label: @hint' (textbox), '[Save | Cancel]' (buttons), '[if condition] { elements }' (conditional)."
                                                },
                                                "title": {
                                                    "type": "string",
                                                    "description": "Optional title for the popup window"
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
                            "popup" => {
                                let dsl = tool_args.get("dsl").and_then(|d| d.as_str()).unwrap_or("");
                                let title = tool_args.get("title").and_then(|t| t.as_str());
                                
                                log::info!("Showing popup with DSL: {}", dsl);
                                if let Some(title) = title {
                                    log::info!("Title: {}", title);
                                }
                                
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
                                let child = if let Some(binary_path) = popup_path {
                                    log::info!("Spawning popup binary directly: {:?}", binary_path);
                                    let mut cmd = std::process::Command::new(binary_path);
                                    if let Some(title) = title {
                                        cmd.args(&["--title", title]);
                                    }
                                    cmd
                                        .stdin(std::process::Stdio::piped())
                                        .stdout(std::process::Stdio::piped())
                                        .stderr(std::process::Stdio::piped())
                                        .spawn()
                                        .map_err(|e| format!("Failed to spawn popup subprocess: {}", e))
                                } else {
                                    // Fallback to cargo run for development
                                    log::info!("Falling back to cargo run for popup");
                                    let mut args = vec!["run", "--release", "--bin", "popup-mcp", "--quiet", "--"];
                                    if let Some(title) = title {
                                        args.extend(&["--title", title]);
                                    }
                                    std::process::Command::new("cargo")
                                        .args(&args)
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
