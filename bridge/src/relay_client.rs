use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use futures_util::{SinkExt, StreamExt};
use crate::crypto::generate_keypair;
use crate::verify::{VerifyContext, verify_signed_message, parse_signed_message};
use crate::adapters::{echo, shell};
use crate::db::BridgeDb;
use crate::event_log::store::EventLog;
use crate::event_log::model::EventType;
use chrono::Utc;

pub struct BridgeClient {
    relay_url: String,
    bridge_id: String,
    db: BridgeDb,
    log: EventLog,
}

impl BridgeClient {
    pub fn new(relay_url: &str, db_path: &str) -> Result<Self, String> {
        let (_sk, _vk) = generate_keypair();
        let bridge_id = format!("bridge_{:04}", rand::random::<u16>());
        let db = BridgeDb::open(db_path)?;
        let log = EventLog::new_db(BridgeDb::open(db_path)?);
        Ok(BridgeClient {
            relay_url: relay_url.to_string(),
            bridge_id,
            db,
            log,
        })
    }

    pub async fn run(&self) -> Result<(), String> {
        let url = format!(
            "{}/ws?device_id={}&device_type=bridge",
            self.relay_url, self.bridge_id
        );

        println!("[bridge] connecting to relay: {}", url);

        let (ws_stream, _) = connect_async(&url).await
            .map_err(|e| format!("connect: {}", e))?;

        println!("[bridge] connected as {}", self.bridge_id);

        let (mut write, mut read) = ws_stream.split();

        let mut verify_ctx = VerifyContext::new(Utc::now().timestamp_millis());
        self.populate_trusted_devices(&mut verify_ctx)?;

        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    let response = self.handle_message(&text, &mut verify_ctx)?;

                    if let Some(ref resp_text) = response {
                        write.send(Message::Text(resp_text.clone())).await
                            .map_err(|e| format!("send: {}", e))?;
                    }
                }
                Ok(Message::Close(_)) => {
                    println!("[bridge] relay closed connection");
                    break;
                }
                Err(e) => {
                    println!("[bridge] ws error: {}", e);
                    break;
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn populate_trusted_devices(&self, ctx: &mut VerifyContext) -> Result<(), String> {
        let devices = self.db.list_trusted_devices()?;
        for d in devices {
            if d.status == "active" {
                match crate::crypto::public_key_from_base64(&d.public_key_base64) {
                    Ok(vk) => {
                        ctx.add_trusted_device(&d.device_id, vk);
                        ctx.device_last_seq.insert(d.device_id.clone(), d.last_seq);
                    }
                    Err(e) => println!("[bridge] skip device {}: {}", d.device_id, e),
                }
            } else {
                ctx.revoke_device(&d.device_id);
            }
        }
        Ok(())
    }

    fn handle_message(&self, text: &str, ctx: &mut VerifyContext) -> Result<Option<String>, String> {
        let msg = match parse_signed_message(text) {
            Ok(m) => m,
            Err(e) => {
                let error_resp = serde_json::json!({
                    "type": "error",
                    "error": format!("{}", e),
                    "from": self.bridge_id,
                });
                return Ok(Some(error_resp.to_string()));
            }
        };

        ctx.now_ms = Utc::now().timestamp_millis();

        let is_nonce_used = self.db.is_nonce_used(&msg.device_id, &msg.nonce)?;
        if is_nonce_used {
            ctx.used_nonces.insert((msg.device_id.clone(), msg.nonce.clone()));
        }

        match verify_signed_message(&msg, ctx) {
            Ok(_) => {
                self.db.add_used_nonce(&msg.device_id, &msg.nonce, msg.timestamp_ms, &msg.message_id)?;
                self.db.update_last_seq(&msg.device_id, msg.seq)?;

                let device_info = format!("device_id={} message_id={}", msg.device_id, msg.message_id);
                println!("[bridge] verified: {} type={}", device_info, msg.msg_type);

                match msg.msg_type.as_str() {
                    "echo.ping" => {
                        let content = msg.payload.get("echo").and_then(|v| v.as_str()).unwrap_or("");
                        let pong = echo::handle_echo(content);
                        let resp = serde_json::json!({
                            "type": "echo.pong",
                            "content": pong,
                            "from": self.bridge_id,
                            "to": msg.device_id,
                        });
                        Ok(Some(resp.to_string()))
                    }
                    "session.create" => {
                        let session_id = msg.payload.get("session_id").and_then(|v| v.as_str()).unwrap_or("default");
                        let workspace = msg.payload.get("workspace").and_then(|v| v.as_str()).unwrap_or("/tmp");
                        let _title = msg.payload.get("title").and_then(|v| v.as_str()).unwrap_or(session_id);

                        shell::create_shell_session(session_id, workspace)?;

                        self.log.record(Some(session_id), EventType::SessionCreated,
                            Some(&format!("workspace={}", workspace)), None)?;

                        let resp = serde_json::json!({
                            "type": "session.created",
                            "session_id": session_id,
                            "from": self.bridge_id,
                            "to": msg.device_id,
                        });
                        Ok(Some(resp.to_string()))
                    }
                    "session.input" => {
                        let session_id = msg.payload.get("session_id").and_then(|v| v.as_str()).unwrap_or("default");
                        let content = msg.payload.get("content").and_then(|v| v.as_str()).unwrap_or("");

                        shell::send_shell_input(session_id, content)?;
                        self.log.record(Some(session_id), EventType::UserInput, Some(content), None)?;

                        let _ = std::thread::sleep(std::time::Duration::from_millis(200));

                        let output = shell::capture_shell_output(session_id, 20).unwrap_or_default();
                        self.log.record(Some(session_id), EventType::AgentOutput, Some(&output), None)?;

                        let resp = serde_json::json!({
                            "type": "agent.output",
                            "session_id": session_id,
                            "content": output,
                            "from": self.bridge_id,
                            "to": msg.device_id,
                        });
                        Ok(Some(resp.to_string()))
                    }
                    "session.stop" => {
                        let session_id = msg.payload.get("session_id").and_then(|v| v.as_str()).unwrap_or("default");
                        shell::stop_shell_session(session_id)?;
                        self.log.record(Some(session_id), EventType::SessionStopped, None, None)?;
                        let resp = serde_json::json!({
                            "type": "session.stopped",
                            "session_id": session_id,
                            "from": self.bridge_id,
                            "to": msg.device_id,
                        });
                        Ok(Some(resp.to_string()))
                    }
                    _ => {
                        let resp = serde_json::json!({
                            "type": "error",
                            "error": format!("UNKNOWN_TYPE: {}", msg.msg_type),
                            "from": self.bridge_id,
                            "to": msg.device_id,
                        });
                        Ok(Some(resp.to_string()))
                    }
                }
            }
            Err(e) => {
                println!("[bridge] verification failed: {} for device={} message={}", e, msg.device_id, msg.message_id);
                let resp = serde_json::json!({
                    "type": "error",
                    "error": format!("{}", e),
                    "from": self.bridge_id,
                    "to": msg.device_id,
                    "message_id": msg.message_id,
                });
                Ok(Some(resp.to_string()))
            }
        }
    }
}
