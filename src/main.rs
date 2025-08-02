mod client;
mod error;
mod message;
mod server;

use message::ChatMessage;
use tokio::sync::broadcast;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("[Main] Starting chat server...");

    let (broadcast_tx, _) = broadcast::channel::<ChatMessage>(32);

    let address = "127.0.0.1:8080";
    if let Err(e) = server::run_server(address, broadcast_tx).await {
        eprintln!("[Main] Server error: {:?}", e);
    }

    Ok(())
}
