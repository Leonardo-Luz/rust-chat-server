use crate::chat::{Client, SharedState};
use futures::{SinkExt, StreamExt};
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

                    if ws_sink
                        .send(Message::Text("Enter your nickname:".into()))
                        .await
                        .is_err()
                    {
                        return;
                    }

                    let nickname = match ws_stream.next().await {
                        Some(Ok(Message::Text(name))) => name,
                        _ => return,
                    };

                    let mut current_room = "general".to_string();

                    let client = Client {
                        nickname: nickname.clone(),
                        tx: tx.clone(),
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
                                        };
                                        let mut s = state_clone.lock().await;
                                        match s.join_room(new_room, client, password) {
                                            Ok(_) => {
                                                current_room = new_room.to_string();
                                                tx_clone
                                                    .send(Message::Text(format!(
                                                        "Joined room {}",
                                                        current_room
                                                    )))
                                                    .unwrap();
                                                log::info!(
                                                    "{} joined room {}",
                                                    nickname_clone,
                                                    current_room
                                                );
                                            }
                                            Err(e) => {
                                                tx_clone
                                                    .send(Message::Text(format!(
                                                        "Failed to join room: {}",
                                                        e
                                                    )))
                                                    .unwrap();
                                            }
                                        }
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
