// Message definitions will go here
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessage {
    pub sender_addr: Option<String>, // For server-side identification, not necessarily for display
    pub username: Option<String>,    // Display name of the sender
    pub content: String,
} 