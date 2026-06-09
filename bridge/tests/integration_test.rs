#[cfg(test)]
mod integration_tests {
    use tokio_tungstenite::connect_async;
    use tokio_tungstenite::tungstenite::Message;
    use futures_util::{SinkExt, StreamExt};
    use agent_bridge::crypto::*;
    use agent_bridge::db::BridgeDb;
    use agent_bridge::pairing;
    use chrono::Utc;
    use std::sync::Arc;
    use std::time::Duration;
    use std::collections::HashMap;
    use axum::extract::{WebSocketUpgrade, Query};

    async fn wait_for_relay(url: &str, timeout: Duration) -> bool {
        let start = std::time::Instant::now();
        while start.elapsed() < timeout {
            if let Ok(resp) = reqwest::get(url).await {
                if resp.status().is_success() {
                    return true;
                }
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        false
    }

    struct PhoneWsClient {
        device_id: String,
        signing_key: ed25519_dalek::SigningKey,
        seq: i64,
    }

    impl PhoneWsClient {
        fn new() -> Self {
            let (sk, _) = generate_keypair();
            let device_id = format!("phone_test_{:04}", rand::random::<u16>());
            PhoneWsClient { device_id: device_id.clone(), signing_key: sk, seq: 0 }
        }

        fn public_key_b64(&self) -> String {
            public_key_to_base64(&self.signing_key.verifying_key())
        }

        fn make_signed_echo(&mut self, content: &str, to: &str) -> serde_json::Value {
            self.seq += 1;
            let message_id = format!("echo_{:04}", rand::random::<u16>());
            let timestamp_ms = Utc::now().timestamp_millis();
            let nonce_bytes: [u8; 16] = rand::random();
            let nonce = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, nonce_bytes);
            let payload = serde_json::json!({"echo": content});
            let payload_sha256 = compute_payload_sha256(&payload);

            let signing_string = build_signing_string(
                &message_id, &self.device_id, "echo.ping", timestamp_ms,
                &nonce, self.seq, &payload_sha256,
            );
            let signature = sign(&self.signing_key, &signing_string);

            serde_json::json!({
                "version": 1,
                "message_id": message_id,
                "device_id": self.device_id,
                "type": "echo.ping",
                "timestamp_ms": timestamp_ms,
                "nonce": nonce,
                "seq": self.seq,
                "payload": payload,
                "payload_sha256": payload_sha256,
                "signature": signature_to_base64(&signature),
                "to": to,
            })
        }
    }

    #[tokio::test]
    async fn test_relay_bridge_echo_loop() {
        let relay_port = find_available_port();
        let relay_url = format!("http://127.0.0.1:{}", relay_port);
        let relay_ws_url = format!("ws://127.0.0.1:{}", relay_port);

        let relay_state = Arc::new(agent_relay::ws::RelayState::new());
        let app = axum::Router::new()
            .route("/health", axum::routing::get(|| async { axum::Json(serde_json::json!({"status":"ok"})) }))
            .route("/ws", axum::routing::get(ws_handler))
            .with_state(relay_state.clone());

        let relay_handle = tokio::spawn(async move {
            let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", relay_port)).await.unwrap();
            axum::serve(listener, app).await.unwrap();
        });

        tokio::time::sleep(Duration::from_millis(300)).await;

        assert!(wait_for_relay(&format!("{}/health", relay_url), Duration::from_secs(5)).await,
            "Relay health check failed");

        let db = BridgeDb::open(":memory:").unwrap();
        let mut phone = PhoneWsClient::new();
        let pairing_code = pairing::generate_pairing_code(&db).unwrap();
        let phone_device_id = pairing::verify_pairing_request(
            &db, &pairing_code, &phone.public_key_b64(), "PhoneSim", "android",
        ).unwrap();
        phone.device_id = phone_device_id;

        let bridge_id = format!("bridge_test_{:04}", rand::random::<u16>());
        let bridge_id_clone = bridge_id.clone();

        let bridge_ws_url = format!("{}/ws?device_id={}&device_type=bridge", relay_ws_url, bridge_id);
        let (bridge_ws, _) = connect_async(&bridge_ws_url).await.unwrap();
        let (bridge_write, mut bridge_read) = bridge_ws.split();

        let mut bridge_write = bridge_write;

        let bridge_handle = tokio::spawn(async move {
            while let Some(Ok(msg)) = bridge_read.next().await {
                if let Message::Text(text) = msg {
                    let parsed: serde_json::Value = serde_json::from_str(&text).unwrap_or_default();
                    let from = parsed.get("device_id").and_then(|v| v.as_str()).unwrap_or("");
                    let content = parsed.get("payload").and_then(|v| v.get("echo")).and_then(|v| v.as_str()).unwrap_or("");
                    let resp = serde_json::json!({
                        "type": "echo.pong",
                        "content": format!("echo.pong: {}", content),
                        "from": bridge_id_clone,
                        "to": from,
                    });
                    bridge_write.send(Message::Text(resp.to_string())).await.ok();
                }
            }
        });

        tokio::time::sleep(Duration::from_millis(200)).await;

        let phone_ws_url = format!("{}/ws?device_id={}&device_type=phone", relay_ws_url, phone.device_id);
        let (phone_ws, _) = connect_async(&phone_ws_url).await.unwrap();
        let (mut phone_write, mut phone_read) = phone_ws.split();

        tokio::time::sleep(Duration::from_millis(200)).await;

        let echo_msg = phone.make_signed_echo("hello docker test", &bridge_id);
        phone_write.send(Message::Text(echo_msg.to_string())).await.unwrap();

        let response = tokio::time::timeout(Duration::from_secs(5), async {
            while let Some(Ok(msg)) = phone_read.next().await {
                if let Message::Text(text) = msg {
                    if text.contains("echo.pong") {
                        return text;
                    }
                }
            }
            String::new()
        }).await.unwrap_or_default();

        assert!(response.contains("echo.pong"), "Should receive echo.pong, got: {}", &response[..200.min(response.len())]);
        assert!(response.contains("hello docker test"), "Response should contain original content");

        let _ = phone_write.close().await;
        bridge_handle.abort();
        relay_handle.abort();
    }

    async fn ws_handler(
        ws: WebSocketUpgrade,
        Query(params): Query<HashMap<String, String>>,
        state: axum::extract::State<Arc<agent_relay::ws::RelayState>>,
    ) -> impl axum::response::IntoResponse {
        let device_id = params.get("device_id").cloned().unwrap_or_default();
        let device_type = params.get("device_type").cloned().unwrap_or_default();
        let state_inner = state.0.clone();
        ws.on_upgrade(move |socket| agent_relay::ws::handle_ws(socket, device_id, device_type, state_inner))
    }

    fn find_available_port() -> u16 {
        use std::net::TcpListener;
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.local_addr().unwrap().port()
    }
}
