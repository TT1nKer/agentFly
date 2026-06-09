use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};

#[derive(Debug, Clone)]
pub struct DeviceConnection {
    pub device_id: String,
    pub device_type: String,
    pub sender: Arc<Mutex<futures_util::stream::SplitSink<WebSocket, Message>>>,
}

pub struct RelayState {
    pub devices: Mutex<HashMap<String, DeviceConnection>>,
    pub shutdown_tx: broadcast::Sender<()>,
}

impl RelayState {
    pub fn new() -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        RelayState {
            devices: Mutex::new(HashMap::new()),
            shutdown_tx,
        }
    }

    pub async fn register(&self, device_id: String, device_type: String, sender: Arc<Mutex<futures_util::stream::SplitSink<WebSocket, Message>>>) {
        let mut devices = self.devices.lock().await;
        devices.insert(device_id.clone(), DeviceConnection {
            device_id: device_id.clone(),
            device_type,
            sender,
        });
        println!("[relay] device registered: {} (total: {})", device_id, devices.len());
    }

    pub async fn unregister(&self, device_id: &str) {
        let mut devices = self.devices.lock().await;
        devices.remove(device_id);
        println!("[relay] device removed: {} (total: {})", device_id, devices.len());
    }

    pub async fn send_to(&self, target_id: &str, msg: &str) -> Result<(), String> {
        let devices = self.devices.lock().await;
        match devices.get(target_id) {
            Some(conn) => {
                let mut sender = conn.sender.lock().await;
                sender.send(Message::Text(msg.to_string())).await
                    .map_err(|e| format!("send error: {}", e))?;
                Ok(())
            }
            None => Err(format!("device {} not connected", target_id)),
        }
    }

    pub async fn list_devices(&self) -> Vec<(String, String)> {
        let devices = self.devices.lock().await;
        devices.iter().map(|(id, conn)| (id.clone(), conn.device_type.clone())).collect()
    }
}

pub async fn handle_ws(socket: WebSocket, device_id: String, device_type: String, state: Arc<RelayState>) {
    let (sender, mut receiver) = socket.split();
    let sender = Arc::new(Mutex::new(sender));

    state.register(device_id.clone(), device_type.clone(), sender.clone()).await;

    let _device_id_clone = device_id.clone();
    let _state_clone = state.clone();

    let result: Result<(), axum::Error> = async {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    match serde_json::from_str::<Value>(&text) {
                        Ok(json) => {
                            let target = json.get("to").and_then(|v| v.as_str()).unwrap_or("");

                            if target.is_empty() {
                                let response = serde_json::json!({
                                    "type": "error",
                                    "error": "MISSING_TO_FIELD",
                                    "from": "relay",
                                    "message_id": json.get("message_id").and_then(|v| v.as_str()).unwrap_or(""),
                                });
                                let mut s = sender.lock().await;
                                s.send(Message::Text(response.to_string())).await?;
                                continue;
                            }

                            match state.send_to(target, &text).await {
                                Ok(_) => {
                                    println!("[relay] {} -> {}", device_id, target);
                                }
                                Err(e) => {
                                    let response = serde_json::json!({
                                        "type": "error",
                                        "error": "DELIVERY_FAILED",
                                        "detail": e,
                                        "from": "relay",
                                        "message_id": json.get("message_id").and_then(|v| v.as_str()).unwrap_or(""),
                                    });
                                    let mut s = sender.lock().await;
                                    s.send(Message::Text(response.to_string())).await?;
                                }
                            }
                        }
                        Err(_) => {
                            let response = serde_json::json!({
                                "type": "error",
                                "error": "INVALID_JSON",
                                "from": "relay"
                            });
                            let mut s = sender.lock().await;
                            s.send(Message::Text(response.to_string())).await?;
                        }
                    }
                }
                Ok(Message::Close(_)) => break,
                Ok(Message::Ping(data)) => {
                    let mut s = sender.lock().await;
                    s.send(Message::Pong(data)).await?;
                }
                Err(e) => {
                    println!("[relay] ws error from {}: {}", device_id, e);
                    break;
                }
                _ => {}
            }
        }
        Ok(())
    }.await;

    state.unregister(&device_id).await;

    if let Err(e) = result {
        println!("[relay] connection closed for {}: {}", device_id, e);
    }
}
