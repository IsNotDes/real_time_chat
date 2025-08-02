// Server logic will go here 

use crate::client::handle_client_connection;
use crate::error::ChatError;
use crate::message::ChatMessage;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio_tungstenite::accept_async;

pub async fn run_server(
    address: &str,
    broadcast_tx: broadcast::Sender<ChatMessage>,
) -> Result<(), ChatError> {
    let listener = TcpListener::bind(address).await.map_err(|e| {
        eprintln!("[Server] Failed to bind to address {}: {}", address, e);
        ChatError::GenericError
    })?;
    println!("[Server] Listening on: {}", address);

    while let Ok((stream, peer_addr)) = listener.accept().await {
        println!("[Server] Incoming TCP connection from: {}", peer_addr);
        
        let client_broadcast_tx = broadcast_tx.clone();

        tokio::spawn(async move {
            match accept_async(stream).await {
                Ok(ws_stream) => {
                    handle_client_connection(peer_addr, ws_stream, client_broadcast_tx).await;
                }
                Err(e) => {
                    println!("[Server] WebSocket handshake error with {}: {}", peer_addr, e);
                }
            }
        });
    }
    Ok(())
} 