mod chat;
mod websocket;

use chat::ChatState;
use log::info;
use std::{env, sync::Arc};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    env_logger::init();

    let mut args = env::args().skip(1);
    let addr = args.next().unwrap_or_else(|| "127.0.0.1:9001".to_string());
    let room = args.next().unwrap_or_else(|| "general".to_string());

    let state = Arc::new(Mutex::new(ChatState::new()));

    info!("🚀 Chat server running at ws://{}", addr);
    info!("💬 Default room: {}", room);

    websocket::run_server(&addr, state).await
}
