// Client handling logic will go here
use crate::message::ChatMessage;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio::sync::broadcast;
use tokio_tungstenite::{
    tungstenite::protocol::Message,
    WebSocketStream,
};

const SERVER_USERNAME: &str = "Server";

#[derive(Deserialize)] // For messages incoming from client
struct ClientContentMessage { content: String }

pub async fn handle_client_connection(
    peer_addr: SocketAddr,
    ws_stream: WebSocketStream<TcpStream>,
    broadcast_tx: broadcast::Sender<ChatMessage>,
) {
    let peer_addr_str = peer_addr.to_string();
    println!("[Client {}] WebSocket connection established. Waiting for username...", peer_addr_str);

    let (mut ws_sender, mut ws_receiver) =
        ws_stream.split();
    
    let mut broadcast_rx = broadcast_tx.subscribe();
    let mut current_username: Option<String> = None;

    // Corrected example in the welcome message content
    let welcome_msg_content = "Welcome! Please send your desired username as your first message (e.g., ".to_string() 
        + "Bob"
        + ").";
    let welcome_msg = ChatMessage {
        sender_addr: None,
        username: Some(SERVER_USERNAME.to_string()),
        content: welcome_msg_content
    };
    match serde_json::to_string(&welcome_msg) {
        Ok(json_msg) => {
            if ws_sender.send(Message::Text(json_msg)).await.is_err() {
                eprintln!("[Client {}] Failed to send welcome message.", peer_addr_str);
            }
        }
        Err(e) => {
            eprintln!("[Client {}] Failed to serialize welcome message: {}", peer_addr_str, e);
        }
    }

    loop {
        tokio::select! {
            // 1. Message received from this specific WebSocket client
            Some(msg_result) = ws_receiver.next() => {
                match msg_result {
                    Ok(msg) => {
                        match msg {
                            Message::Text(text) => {
                                match serde_json::from_str::<ClientContentMessage>(&text) {
                                    Ok(client_msg) => {
                                        if current_username.is_none() { // This is the first message, treat as username
                                            let chosen_username = client_msg.content.trim();
                                            if chosen_username.is_empty() || chosen_username.eq_ignore_ascii_case(SERVER_USERNAME) || chosen_username.len() > 20 {
                                                let err_content = format!("Invalid username: '{}'. Must not be empty, '{}', or too long (max 20 chars).", chosen_username, SERVER_USERNAME);
                                                let err_chat_msg = ChatMessage { sender_addr: None, username: Some(SERVER_USERNAME.to_string()), content: err_content };
                                                if let Ok(json_err_msg) = serde_json::to_string(&err_chat_msg) {
                                                    if ws_sender.send(Message::Text(json_err_msg)).await.is_err() {
                                                        eprintln!("[Client {}] Failed to send username validation error.", peer_addr_str);
                                                        break; // Critical failure
                                                    }
                                                }
                                                // Client needs to try again, don't break connection unless send failed.
                                                continue;
                                            }
                                            current_username = Some(chosen_username.to_string());
                                            println!("[Client {}] Username set to: {}", peer_addr_str, chosen_username);
                                            let confirmation_content = format!("Username set to: {}. You can now send messages.", chosen_username);
                                            let confirmation_msg = ChatMessage {
                                                sender_addr: None, // Server message
                                                username: Some(SERVER_USERNAME.to_string()),
                                                content: confirmation_content,
                                            };
                                            if let Ok(json_confirmation) = serde_json::to_string(&confirmation_msg) {
                                                if ws_sender.send(Message::Text(json_confirmation)).await.is_err() {
                                                    eprintln!("[Client {}] Failed to send username confirmation.", peer_addr_str);
                                                    break; // If we can't confirm, something is wrong
                                                }
                                            }
                                        } else { // Username is set, this is a regular chat message
                                            if !process_chat_message(
                                                &peer_addr_str, 
                                                current_username.as_ref().unwrap(), // Safe to unwrap here
                                                client_msg.content,
                                                &broadcast_tx
                                            ).await {
                                                break; // process_chat_message indicates a desire to close
                                            }
                                        }
                                    }
                                    Err(e) => { // Deserialization error for ClientContentMessage
                                        eprintln!("[Client {}] Failed to deserialize from text: {}. Raw: {}", peer_addr_str, e, text);
                                        // Corrected example in the error message content
                                        let json_example_str = "{\"content\": \"your text\"}"; // Correct for Rust string literal
                                        let error_message_content = format!(
                                            "Error: Could not understand your message. Expected JSON like {}.",
                                            json_example_str
                                        );
                                        let err_chat_msg = ChatMessage { sender_addr: None, username: Some(SERVER_USERNAME.to_string()), content: error_message_content };
                                        if let Ok(json_err_msg) = serde_json::to_string(&err_chat_msg) {
                                            if ws_sender.send(Message::Text(json_err_msg)).await.is_err() {
                                                eprintln!("[Client {}] Failed to send deserialization error message.", peer_addr_str);
                                                break; 
                                            }
                                        }
                                    }
                                }
                            }
                            Message::Close(_) => {
                                println!("[Client {}] Received close message from client.", peer_addr_str);
                                break;
                            }
                            Message::Ping(ping_data) => {
                                println!("[Client {}] Received Ping, sending Pong", peer_addr_str);
                                if ws_sender.send(Message::Pong(ping_data)).await.is_err() {
                                    eprintln!("[Client {}] Error sending Pong.", peer_addr_str);
                                    break;
                                }
                            }
                            Message::Pong(_) => {
                                println!("[Client {}] Received Pong", peer_addr_str);
                            }
                            _ => { // Other message types (Binary, Frame)
                                println!("[Client {}] Received unhandled WebSocket message type.", peer_addr_str);
                            }
                        }
                    }
                    Err(e) => { // Error receiving from WebSocket stream
                        eprintln!("[Client {}] Error receiving WebSocket message: {:?}", peer_addr_str, e);
                        break;
                    }
                }
            }

            // 2. Message received from the broadcast channel (from other clients/server)
            res = broadcast_rx.recv() => {
                match res {
                    Ok(received_chat_msg) => {
                        if Some(peer_addr_str.clone()) == received_chat_msg.sender_addr {
                            // This is our own message that was broadcast, ignore it.
                            // println!("[Client {}] Ignoring own broadcasted message: {:?}", peer_addr_str, received_chat_msg);
                        } else {
                            println!("[Client {}] Received broadcast: {:?}. Sending to client.", peer_addr_str, received_chat_msg);
                            match serde_json::to_string(&received_chat_msg) {
                                Ok(json_msg) => {
                                    if ws_sender.send(Message::Text(json_msg)).await.is_err() {
                                        eprintln!("[Client {}] Failed to send broadcast message to WebSocket.", peer_addr_str);
                                        break;
                                    }
                                }
                                Err(e) => {
                                    eprintln!("[Client {}] Failed to serialize broadcast message for WebSocket: {}", peer_addr_str, e);
                                }
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(missed_count)) => {
                        eprintln!("[Client {}] Broadcast receiver lagged, missed {} messages.", peer_addr_str, missed_count);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        eprintln!("[Client {}] Broadcast channel closed. Closing connection.", peer_addr_str);
                        break;
                    }
                }
            }
        }
    }
    // Announce disconnection with username if available
    let final_log_id = current_username.unwrap_or_else(|| peer_addr_str.clone());
    println!("[Client {}] Disconnected.", final_log_id);
}

// New helper function to specifically process and broadcast chat messages
async fn process_chat_message(
    sender_addr_str: &str,    
    sender_username: &str,
    content: String,
    broadcast_tx: &broadcast::Sender<ChatMessage>,
) -> bool { // Returns false if the connection should be closed due to this processing
    let outgoing_chat_msg = ChatMessage {
        sender_addr: Some(sender_addr_str.to_string()),
        username: Some(sender_username.to_string()),
        content,
    };

    println!("[Client {} ({})] Broadcasting: {:?}", sender_addr_str, sender_username, outgoing_chat_msg);
    if let Err(e) = broadcast_tx.send(outgoing_chat_msg) {
        eprintln!("[Client {} ({})] Failed to broadcast message: {}. (Active receivers: {})", sender_addr_str, sender_username, e, broadcast_tx.receiver_count());
        // Depending on policy, a failed broadcast might not mean this client must disconnect.
        // If it's critical all messages are sent, one might return false here.
    }
    true // Assume success unless a critical error occurs that requires closing this client's connection
}