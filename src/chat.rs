use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use tokio_tungstenite::tungstenite::Message;

pub type Tx = mpsc::UnboundedSender<Message>;

#[derive(Clone)]
pub struct Client {
    pub nickname: String,
    pub tx: Tx,
    pub color: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessage {
    pub msg_type: String,
    pub sender: String,
    pub color: String,
    pub content: String,
    pub room: String,
    pub client_count: usize,
}

#[derive(Clone)]
pub struct Room {
    pub clients: Vec<Client>,
    pub password: Option<String>,
}

#[derive(Default)]
pub struct ChatState {
    pub rooms: HashMap<String, Room>,
}

impl ChatState {
    pub fn new() -> Self {
        Self {
            rooms: HashMap::new(),
        }
    }

    /// Join a room, removing the client from any previous rooms
    pub fn join_room(
        &mut self,
        room: &str,
        client: Client,
        password: Option<String>,
    ) -> Result<(), String> {
        // Remove client from all other rooms
        for (_, r) in self.rooms.iter_mut() {
            let mut removed = false;
            r.clients.retain(|c| {
                if c.tx.same_channel(&client.tx) {
                    removed = true;
                    false
                } else {
                    true
                }
            });
            if removed {
                let exit_msg = ChatMessage {
                    msg_type: "info".to_string(),
                    sender: "server".to_string(),
                    color: "0000ff".to_string(),
                    content: format!("{} has left the room...", client.nickname),
                    room: room.to_string(),
                    client_count: r.clients.len(),
                };
                let json_msg = serde_json::to_string(&exit_msg).unwrap();
                r.clients
                    .retain(|c| c.tx.send(Message::Text(json_msg.clone())).is_ok());
            }
        }

        // Join the new room
        match self.rooms.get_mut(room) {
            Some(r) => {
                if r.password.as_ref() != password.as_ref() {
                    return Err("Incorrect password".to_string());
                }
                let join_msg = ChatMessage {
                    msg_type: "info".to_string(),
                    sender: "server".to_string(),
                    color: "0000ff".to_string(),
                    content: format!("{} has joined the room...", client.nickname),
                    room: room.to_string(),
                    client_count: r.clients.len() + 1,
                };
                let json_msg = serde_json::to_string(&join_msg).unwrap();
                r.clients
                    .retain(|c| c.tx.send(Message::Text(json_msg.clone())).is_ok());
                r.clients.push(client);
                Ok(())
            }
            None => {
                self.rooms.insert(
                    room.to_string(),
                    Room {
                        clients: vec![client],
                        password,
                    },
                );
                Ok(())
            }
        }
    }

    /// Broadcast a message to all clients in the room
    /// Uses the nickname stored in the Client object
    pub fn broadcast(&mut self, room: &str, msg: &str, sender_tx: &Tx) {
        if let Some(r) = self.rooms.get_mut(room) {
            // Find the sender nickname
            if let Some(sender) = r.clients.iter().find(|c| c.tx.same_channel(sender_tx)) {
                let chat_msg = ChatMessage {
                    msg_type: "chat".to_string(),
                    sender: sender.nickname.clone(),
                    color: sender.color.clone(),
                    content: msg.to_string(),
                    room: room.to_string(),
                    client_count: r.clients.len(),
                };
                let json_msg = serde_json::to_string(&chat_msg).unwrap();

                r.clients
                    .retain(|c| c.tx.send(Message::Text(json_msg.clone())).is_ok());
            }
        }
    }
}

pub type SharedState = Arc<Mutex<ChatState>>;
