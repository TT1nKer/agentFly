use axum::extract::ws::{Message, WebSocket};
use futures_util::StreamExt;
use std::sync::Arc;
use tokio::sync::Mutex;

pub type WsSender = Arc<Mutex<futures_util::stream::SplitSink<WebSocket, Message>>>;

pub async fn handle_ws(socket: WebSocket, device_id: String) {
    let (sender, mut receiver) = socket.split();
    let _sender = Arc::new(Mutex::new(sender));

    println!("Device {} connected", device_id);

    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Text(text) => {
                println!("Received from {}: {}", device_id, &text[..100.min(text.len())]);
            }
            Message::Close(_) => {
                println!("Device {} disconnected", device_id);
                break;
            }
            _ => {}
        }
    }
}
