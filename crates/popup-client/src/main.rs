use anyhow::{Context, Result};
use clap::Parser;
use futures_util::{SinkExt, StreamExt};
use popup_common::protocol::{ClientMessage, ServerMessage};
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[derive(Debug, Clone, Deserialize)]
struct Config {
    server_url: String,
    device_name: Option<String>,
    #[serde(default = "default_gui_binary")]
    gui_binary: String,
}

fn default_gui_binary() -> String {
    "popup-gui".to_string()
}

#[derive(Parser)]
#[command(name = "popup-client")]
#[command(about = "WebSocket client daemon for remote popup GUI")]
struct Args {
    /// WebSocket server URL (overrides config)
    #[arg(long)]
    server_url: Option<String>,

    /// Device name for identification (overrides config)
    #[arg(long)]
    device_name: Option<String>,

    /// Path to config file (default: ~/.config/popup-client/config.toml)
    #[arg(long)]
    config: Option<PathBuf>,
}

struct PopupClient {
    config: Config,
    active_popups: HashMap<String, Child>,
    result_tx: mpsc::UnboundedSender<(String, popup_common::PopupResult)>,
}

impl PopupClient {
    fn new(
        config: Config,
        result_tx: mpsc::UnboundedSender<(String, popup_common::PopupResult)>,
    ) -> Self {
        Self {
            config,
            active_popups: HashMap::new(),
            result_tx,
        }
    }

    async fn handle_show_popup(
        &mut self,
        id: String,
        definition: popup_common::PopupDefinition,
    ) -> Result<()> {
        log::info!("Spawning GUI for popup {}", id);

        // Serialize popup definition to JSON
        let json_input = serde_json::to_string(&definition)?;

        // Spawn popup-gui subprocess with --stdin
        let mut child = Command::new(&self.config.gui_binary)
            .arg("--stdin")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .context("Failed to spawn popup-gui subprocess")?;

        // Write JSON to stdin
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(json_input.as_bytes()).await?;
            stdin.write_all(b"\n").await?;
            drop(stdin); // Close stdin
        }

        // Store child process (will be moved out by caller for monitoring)
        self.active_popups.insert(id, child);

        Ok(())
    }

    async fn handle_close_popup(&mut self, id: &str) {
        log::info!("Received close_popup for {}", id);

        // Check if we have this popup running
        if let Some(mut child) = self.active_popups.remove(id) {
            log::info!("Killing GUI subprocess for popup {}", id);
            if let Err(e) = child.kill().await {
                log::error!("Failed to kill subprocess for {}: {}", id, e);
            }
        }
    }

    async fn send_ready_message(
        ws_write: &mut futures_util::stream::SplitSink<
            tokio_tungstenite::WebSocketStream<
                tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
            >,
            Message,
        >,
        device_name: Option<String>,
    ) -> Result<()> {
        let ready_msg = ClientMessage::ready(device_name);
        let json = serde_json::to_string(&ready_msg)?;
        ws_write.send(Message::Text(json.into())).await?;
        Ok(())
    }
}

async fn monitor_subprocess(
    id: String,
    mut child: Child,
    result_tx: mpsc::UnboundedSender<(String, popup_common::PopupResult)>,
) {
    // Read stdout to get result
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        // Read all output (should be single JSON line)
        let mut output = String::new();
        while let Ok(Some(line)) = lines.next_line().await {
            output.push_str(&line);
        }

        // Wait for process to exit
        match child.wait().await {
            Ok(status) => {
                log::info!("GUI subprocess for {} exited with status: {}", id, status);

                // Parse result from stdout
                match serde_json::from_str::<popup_common::PopupResult>(&output) {
                    Ok(result) => {
                        log::info!("Sending result for popup {}", id);
                        let _ = result_tx.send((id, result));
                    }
                    Err(e) => {
                        log::error!("Failed to parse result for {}: {}", id, e);
                        // Send cancelled result on parse error
                        let _ = result_tx.send((id, popup_common::PopupResult::Cancelled));
                    }
                }
            }
            Err(e) => {
                log::error!("Error waiting for subprocess {}: {}", id, e);
                let _ = result_tx.send((id, popup_common::PopupResult::Cancelled));
            }
        }
    }
}

async fn run_client(config: Config) -> Result<()> {
    let mut backoff_seconds = 1;
    const MAX_BACKOFF: u64 = 60;

    loop {
        log::info!("Connecting to {}", config.server_url);

        match connect_to_server(&config).await {
            Ok(_) => {
                log::info!("Connection closed, reconnecting...");
                backoff_seconds = 1; // Reset backoff on successful connection
            }
            Err(e) => {
                log::error!("Connection error: {}", e);
            }
        }

        log::info!("Reconnecting in {} seconds...", backoff_seconds);
        sleep(Duration::from_secs(backoff_seconds)).await;

        // Exponential backoff
        backoff_seconds = (backoff_seconds * 2).min(MAX_BACKOFF);
    }
}

async fn connect_to_server(config: &Config) -> Result<()> {
    // Connect to WebSocket server
    log::debug!("Attempting WebSocket connection to {}", config.server_url);

    let (ws_stream, response) = match connect_async(&config.server_url).await {
        Ok(result) => result,
        Err(e) => {
            log::error!("WebSocket connection failed: {}", e);
            log::error!("Error type: {:?}", e);
            if let Some(source) = e.source() {
                log::error!("Caused by: {}", source);
            }
            return Err(e).context("Failed to connect to WebSocket server");
        }
    };

    log::debug!("WebSocket handshake response: {:?}", response);

    log::info!("Connected to server");

    let (mut ws_write, mut ws_read) = ws_stream.split();

    // Create channel for subprocess results
    let (result_tx, mut result_rx) = mpsc::unbounded_channel();

    // Create client state
    let mut client = PopupClient::new(config.clone(), result_tx);

    // Send ready message
    PopupClient::send_ready_message(&mut ws_write, config.device_name.clone()).await?;

    loop {
        tokio::select! {
            // Receive messages from server
            Some(msg) = ws_read.next() => {
                match msg? {
                    Message::Text(text) => {
                        match serde_json::from_str::<ServerMessage>(&text) {
                            Ok(server_msg) => {
                                match server_msg {
                                    ServerMessage::ShowPopup { id, definition, timeout_ms } => {
                                        log::info!("Received show_popup: id={}, timeout={}ms", id, timeout_ms);

                                        // Spawn subprocess and monitoring task
                                        if let Err(e) = client.handle_show_popup(id.clone(), definition).await {
                                            log::error!("Failed to spawn popup: {}", e);
                                        } else {
                                            // Start monitoring task for this popup
                                            if let Some(child) = client.active_popups.remove(&id) {
                                                let tx = client.result_tx.clone();
                                                tokio::spawn(monitor_subprocess(id.clone(), child, tx));
                                            }
                                        }
                                    }
                                    ServerMessage::ClosePopup { id } => {
                                        client.handle_close_popup(&id).await;
                                    }
                                    ServerMessage::Ping => {
                                        let pong = ClientMessage::pong();
                                        let json = serde_json::to_string(&pong)?;
                                        ws_write.send(Message::Text(json.into())).await?;
                                    }
                                }
                            }
                            Err(e) => {
                                log::error!("Failed to parse server message: {}", e);
                            }
                        }
                    }
                    Message::Close(_) => {
                        log::info!("Server closed connection");
                        break;
                    }
                    _ => {}
                }
            }

            // Send results from subprocesses
            Some((popup_id, result)) = result_rx.recv() => {
                log::info!("Sending result for popup {}", popup_id);
                let result_msg = ClientMessage::result(popup_id, result);
                let json = serde_json::to_string(&result_msg)?;
                ws_write.send(Message::Text(json.into())).await?;
            }
        }
    }

    Ok(())
}

fn load_config(args: &Args) -> Result<Config> {
    let config_path = if let Some(path) = &args.config {
        path.clone()
    } else {
        let home = dirs::home_dir().context("Could not determine home directory")?;
        home.join(".config")
            .join("popup-client")
            .join("config.toml")
    };

    let config_str = std::fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;

    let mut config: Config = toml::from_str(&config_str)
        .with_context(|| format!("Failed to parse config file: {}", config_path.display()))?;

    // Override with CLI args
    if let Some(ref url) = args.server_url {
        config.server_url = url.clone();
    }

    if let Some(ref name) = args.device_name {
        config.device_name = Some(name.clone());
    }

    Ok(config)
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let args = Args::parse();

    // Load config
    let config = load_config(&args)?;

    log::info!("Starting popup client daemon");
    log::info!("Server: {}", config.server_url);
    if let Some(ref device) = config.device_name {
        log::info!("Device: {}", device);
    }

    run_client(config).await
}
