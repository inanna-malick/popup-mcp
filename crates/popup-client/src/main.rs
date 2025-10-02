// Popup WebSocket client daemon
// Connects to Cloudflare Durable Object, receives popup requests, spawns GUI, sends results back

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    log::info!("Popup client starting...");

    // TODO: Load config, connect to DO WebSocket, handle messages

    Ok(())
}
