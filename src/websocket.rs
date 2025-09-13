use crate::chat::{ChatMessage, Client, SharedState};
use futures::{SinkExt, StreamExt};
use serde_json;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio_tungstenite::{accept_async, tungstenite::Message};

pub async fn run_server(addr: &str, state: SharedState) -> tokio::io::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    log::info!("Listening on {}", addr);

    while let Ok((stream, addr)) = listener.accept().await {
        let state = state.clone();
        log::info!("New client connected: {}", addr);

        tokio::spawn(async move {
            match accept_async(stream).await {
                Ok(ws_stream) => {
                    let (mut ws_sink, mut ws_stream) = ws_stream.split();

                    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

                    let welcome_msg = ChatMessage {
                        msg_type: "info".to_string(),
                        sender: "server".to_string(),
                        color: "0000ff".to_string(),
                        content: "Enter your nickname:".to_string(),
                        room: "".to_string(),
                        client_count: 0,
                    };
                    let json_welcome_msg = serde_json::to_string(&welcome_msg).unwrap();

                    if ws_sink.send(Message::Text(json_welcome_msg)).await.is_err() {
                        return;
                    }

                    let nickname = match ws_stream.next().await {
                        Some(Ok(Message::Text(name))) => name,
                        _ => return,
                    };

                    let color_prompt_msg = ChatMessage {
                        msg_type: "info".to_string(),
                        sender: "server".to_string(),
                        color: "0000ff".to_string(),
                        content: "Enter your hex color (e.g., #RRGGBB):".to_string(),
                        room: "".to_string(),
                        client_count: 0,
                    };
                    let json_color_prompt_msg = serde_json::to_string(&color_prompt_msg).unwrap();

                    if ws_sink
                        .send(Message::Text(json_color_prompt_msg))
                        .await
                        .is_err()
                    {
                        return;
                    }

                    let color = match ws_stream.next().await {
                        Some(Ok(Message::Text(hex_color))) => hex_color,
                        _ => return,
                    };

                    let mut current_room = "general".to_string();

                    let client = Client {
                        nickname: nickname.clone(),
                        tx: tx.clone(),
                        color: color.clone(),
                    };
                    {
                        let mut s = state.lock().await;
                        s.join_room(&current_room, client, None).unwrap();
                    }

                    log::info!("{} joined the room {}", nickname, current_room);

                    let mut ws_sink_outgoing = ws_sink;
                    tokio::spawn(async move {
                        while let Some(msg) = rx.recv().await {
                            if ws_sink_outgoing.send(msg).await.is_err() {
                                break;
                            }
                        }
                    });

                    let state_clone = state.clone();
                    let tx_clone = tx.clone();
                    let nickname_clone = nickname.clone();
                    let color_clone = color.clone();
                    tokio::spawn(async move {
                        while let Some(Ok(msg)) = ws_stream.next().await {
                            if let Message::Text(text) = msg {
                                if text.starts_with("/join ") {
                                    let mut parts = text.split_whitespace();
                                    parts.next();
                                    if let Some(new_room) = parts.next() {
                                        let password = parts.next().map(|s| s.to_string());
                                        let client = Client {
                                            nickname: nickname_clone.clone(),
                                            tx: tx_clone.clone(),
                                            color: color_clone.clone(),
                                        };
                                        let mut s = state_clone.lock().await;
                                        match s.join_room(new_room, client, password) {
                                            Ok(_) => {
                                                current_room = new_room.to_string();
                                                let join_success_msg = ChatMessage {
                                                    msg_type: "info".to_string(),
                                                    sender: "server".to_string(),
                                                    color: "0000ff".to_string(),
                                                    content: format!(
                                                        "Joined room {}",
                                                        current_room
                                                    ),
                                                    room: current_room.clone(),
                                                    client_count: 0,
                                                };
                                                let json_join_success_msg =
                                                    serde_json::to_string(&join_success_msg)
                                                        .unwrap();
                                                tx_clone
                                                    .send(Message::Text(json_join_success_msg))
                                                    .unwrap();
                                                log::info!(
                                                    "{} joined room {}",
                                                    nickname_clone,
                                                    current_room
                                                );
                                            }
                                            Err(e) => {
                                                let join_fail_msg = ChatMessage {
                                                    msg_type: "error".to_string(),
                                                    sender: "server".to_string(),
                                                    color: "FF0000".to_string(),
                                                    content: format!("Failed to join room: {}", e),
                                                    room: current_room.clone(),
                                                    client_count: 0,
                                                };
                                                let json_join_fail_msg =
                                                    serde_json::to_string(&join_fail_msg).unwrap();
                                                tx_clone
                                                    .send(Message::Text(json_join_fail_msg))
                                                    .unwrap();
                                            }
                                        }
                                    }
                                } else if text.starts_with("/color ") {
                                    let mut parts = text.split_whitespace();
                                    parts.next(); // Skip "/color"
                                    if let Some(new_color) = parts.next() {
                                        let mut s = state_clone.lock().await;
                                        let mut color_updated = false;
                                        for (_, room) in s.rooms.iter_mut() {
                                            for client in room.clients.iter_mut() {
                                                if client.tx.same_channel(&tx_clone) {
                                                    client.color = new_color.to_string();
                                                    color_updated = true;
                                                    break;
                                                }
                                            }
                                            if color_updated {
                                                break;
                                            }
                                        }

                                        if color_updated {
                                            let color_success_msg = ChatMessage {
                                                msg_type: "info".to_string(),
                                                sender: "server".to_string(),
                                                color: new_color.to_string(),
                                                content: format!(
                                                    "Your color has been set to #{}",
                                                    new_color
                                                ),
                                                room: current_room.clone(),
                                                client_count: 0,
                                            };
                                            let json_color_success_msg =
                                                serde_json::to_string(&color_success_msg).unwrap();
                                            tx_clone
                                                .send(Message::Text(json_color_success_msg))
                                                .unwrap();
                                            log::info!(
                                                "{} changed color to #{}",
                                                nickname_clone,
                                                new_color
                                            );
                                        } else {
                                            let color_fail_msg = ChatMessage {
                                                msg_type: "error".to_string(),
                                                sender: "server".to_string(),
                                                color: "FF0000".to_string(),
                                                content:
                                                    "Failed to change color. Are you in a room?"
                                                        .to_string(),
                                                room: current_room.clone(),
                                                client_count: 0,
                                            };
                                            let json_color_fail_msg =
                                                serde_json::to_string(&color_fail_msg).unwrap();
                                            tx_clone
                                                .send(Message::Text(json_color_fail_msg))
                                                .unwrap();
                                        }
                                    } else {
                                        let color_fail_msg = ChatMessage {
                                            msg_type: "error".to_string(),
                                            sender: "server".to_string(),
                                            color: "FF0000".to_string(),
                                            content: "Please provide a hex color (e.g., #RRGGBB)"
                                                .to_string(),
                                            room: current_room.clone(),
                                            client_count: 0,
                                        };
                                        let json_color_fail_msg =
                                            serde_json::to_string(&color_fail_msg).unwrap();
                                        tx_clone.send(Message::Text(json_color_fail_msg)).unwrap();
                                    }
                                } else {
                                    let mut s = state_clone.lock().await;
                                    s.broadcast(&current_room, &text, &tx_clone);
                                }
                            }
                        }
                        log::info!("Client {} disconnected", nickname_clone);
                    });
                }
                Err(e) => log::error!("WebSocket handshake failed: {}", e),
            }
        });
    }

    Ok(())
}
