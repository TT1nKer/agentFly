#[cfg(test)]
mod full_path_tests {
    use agent_bridge::crypto::*;
    use agent_bridge::verify::*;
    use agent_bridge::db::BridgeDb;
    use agent_bridge::pairing;
    use agent_bridge::adapters::tmux;
    use agent_bridge::event_log::store::EventLog;
    use agent_bridge::event_log::model::EventType;
    use chrono::Utc;
    use std::time::Duration;
    use std::thread;

    struct PhoneSimulator {
        signing_key: ed25519_dalek::SigningKey,
        verifying_key: ed25519_dalek::VerifyingKey,
        device_id: String,
        seq: i64,
    }

    impl PhoneSimulator {
        fn new() -> Self {
            let (sk, vk) = generate_keypair();
            let device_id = format!("phone_{:04}", rand::random::<u16>());
            PhoneSimulator {
                signing_key: sk,
                verifying_key: vk,
                device_id,
                seq: 0,
            }
        }

        fn make_signed_message(&mut self, msg_type: &str, payload: serde_json::Value) -> SignedMessage {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            self.seq += 1;
            let message_id = format!("{}_{:04}", msg_type.replace(".", "_"), rng.gen::<u16>());
            let timestamp_ms = Utc::now().timestamp_millis();
            let nonce_bytes: [u8; 16] = rng.gen();
            let nonce = base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                nonce_bytes,
            );
            let payload_sha256 = compute_payload_sha256(&payload);

            let signing_string = build_signing_string(
                &message_id, &self.device_id, msg_type, timestamp_ms,
                &nonce, self.seq, &payload_sha256,
            );
            let signature = sign(&self.signing_key, &signing_string);
            let signature_b64 = signature_to_base64(&signature);

            SignedMessage {
                version: 1,
                message_id,
                device_id: self.device_id.clone(),
                msg_type: msg_type.to_string(),
                timestamp_ms,
                nonce,
                seq: self.seq,
                payload,
                payload_sha256,
                signature: signature_b64,
            }
        }

        fn public_key_b64(&self) -> String {
            public_key_to_base64(&self.verifying_key)
        }
    }

    #[test]
    fn test_full_path_signed_session_create_input_output() {
        let tmpdir = std::env::temp_dir().join("agent_cockpit_phase6");
        let _ = std::fs::create_dir_all(&tmpdir);
        let db_path = tmpdir.join("test_phase6.db");
        let _ = std::fs::remove_file(&db_path);
        let tmux_name = "ac_phase6_test";
        let session_id = "sess_phase6_001";

        if tmux::session_exists(tmux_name) {
            tmux::kill_session(tmux_name).ok();
        }

        let db = BridgeDb::open(db_path.to_str().unwrap()).unwrap();

        let mut phone = PhoneSimulator::new();
        let pairing_code = pairing::generate_pairing_code(&db).unwrap();
        let device_id = pairing::verify_pairing_request(
            &db, &pairing_code, &phone.public_key_b64(), "PhoneSim", "android",
        ).unwrap();
        phone.device_id = device_id.clone();

        let mut ctx = VerifyContext::new(Utc::now().timestamp_millis());
        ctx.add_trusted_device(&device_id, phone.verifying_key);

        let create_msg = phone.make_signed_message(
            "session.create",
            serde_json::json!({
                "session_id": session_id,
                "kind": "shell",
                "workspace": tmpdir.to_str().unwrap(),
                "title": "Phase 6 Test Shell"
            }),
        );

        let result = verify_signed_message(&create_msg, &mut ctx);
        assert!(result.is_ok(), "session.create should pass verification: {:?}", result);

        let workspace = create_msg.payload["workspace"].as_str().unwrap();
        assert!(tmux::create_tmux_session(tmux_name, workspace, "/bin/bash").is_ok(),
            "Should create tmux session");
        assert!(tmux::session_exists(tmux_name));

        let log = EventLog::new_db(db);
        log.record(
            Some(session_id),
            EventType::SessionCreated,
            None,
            Some(&serde_json::to_string(&create_msg.payload).unwrap()),
        ).unwrap();

        thread::sleep(Duration::from_millis(200));

        ctx.now_ms = Utc::now().timestamp_millis();
        let input_msg = phone.make_signed_message(
            "session.input",
            serde_json::json!({
                "session_id": session_id,
                "content": "echo hello_from_phone"
            }),
        );

        let result = verify_signed_message(&input_msg, &mut ctx);
        assert!(result.is_ok(), "session.input should pass verification: {:?}", result);

        let input_content = input_msg.payload["content"].as_str().unwrap();
        assert!(tmux::send_keys(tmux_name, input_content).is_ok());

        log.record(
            Some(session_id),
            EventType::UserInput,
            Some(input_content),
            None,
        ).unwrap();

        thread::sleep(Duration::from_millis(500));

        let output = tmux::capture_output(tmux_name, 10).unwrap();
        assert!(output.contains("hello_from_phone"),
            "Output should contain the input content, got: {}", output);

        log.record(
            Some(session_id),
            EventType::AgentOutput,
            Some(&output),
            None,
        ).unwrap();

        let events = log.fetch_after(Some(session_id), 0).unwrap();
        assert_eq!(events.len(), 3, "Should have created, input, output events");
        assert_eq!(events[0].event_type, EventType::SessionCreated);
        assert_eq!(events[1].event_type, EventType::UserInput);
        assert_eq!(events[2].event_type, EventType::AgentOutput);

        // Verify unsigned message rejected
        let (_sk2, _) = generate_keypair();
        ctx.now_ms = Utc::now().timestamp_millis();
        let bad_input_msg = phone.make_signed_message(
            "session.input",
            serde_json::json!({"session_id": session_id, "content": "malicious"}),
        );
        let mut bad_msg = bad_input_msg;
        bad_msg.device_id = "evil_device".to_string();
        let result = verify_signed_message(&bad_msg, &mut ctx);
        assert!(result.is_err(), "Message from untrusted device should be rejected");

        // Cleanup
        assert!(tmux::kill_session(tmux_name).is_ok());
        assert!(!tmux::session_exists(tmux_name));

        let _ = std::fs::remove_dir_all(&tmpdir);
    }
}
