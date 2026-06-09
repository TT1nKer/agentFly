#[cfg(test)]
mod echo_loop_tests {
    use agent_bridge::crypto::*;
    use agent_bridge::verify::*;
    use agent_bridge::adapters::echo;
    use agent_bridge::db::BridgeDb;
    use agent_bridge::pairing;
    use chrono::Utc;

    fn setup_device_and_db() -> (ed25519_dalek::SigningKey, ed25519_dalek::VerifyingKey, BridgeDb, String) {
        let (sk, vk) = generate_keypair();
        let public_key_b64 = public_key_to_base64(&vk);
        let db = BridgeDb::open(":memory:").unwrap();

        let code = pairing::generate_pairing_code(&db).unwrap();
        let device_id = pairing::verify_pairing_request(
            &db, &code, &public_key_b64, "TestPhone", "android",
        ).unwrap();

        (sk, vk, db, device_id)
    }

    fn make_echo_ping_message(
        sk: &ed25519_dalek::SigningKey,
        device_id: &str,
        seq: i64,
        content: &str,
    ) -> SignedMessage {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let message_id = format!("echo_{:04}", rng.gen::<u16>());
        let timestamp_ms = Utc::now().timestamp_millis();
        let nonce_bytes: [u8; 16] = rng.gen();
        let nonce = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            nonce_bytes,
        );
        let payload = serde_json::json!({"echo": content});
        let payload_sha256 = compute_payload_sha256(&payload);

        let signing_string = build_signing_string(
            &message_id, device_id, "echo.ping", timestamp_ms, &nonce, seq, &payload_sha256,
        );
        let signature = sign(sk, &signing_string);
        let signature_b64 = signature_to_base64(&signature);

        SignedMessage {
            version: 1,
            message_id,
            device_id: device_id.to_string(),
            msg_type: "echo.ping".to_string(),
            timestamp_ms,
            nonce,
            seq,
            payload,
            payload_sha256,
            signature: signature_b64,
        }
    }

    #[test]
    fn test_echo_loop_with_valid_signature() {
        let (sk, vk, _db, device_id) = setup_device_and_db();

        let mut ctx = VerifyContext::new(Utc::now().timestamp_millis());
        ctx.add_trusted_device(&device_id, vk);

        let msg = make_echo_ping_message(&sk, &device_id, 1, "hello world");

        let result = verify_signed_message(&msg, &mut ctx);
        assert!(result.is_ok(), "Valid echo.ping should pass verification: {:?}", result);

        let response = echo::handle_echo(msg.payload["echo"].as_str().unwrap_or(""));
        assert_eq!(response, "echo.pong: hello world");
    }

    #[test]
    fn test_echo_loop_with_tampered_payload_rejected() {
        let (sk, vk, _db, device_id) = setup_device_and_db();

        let mut ctx = VerifyContext::new(Utc::now().timestamp_millis());
        ctx.add_trusted_device(&device_id, vk);

        let mut msg = make_echo_ping_message(&sk, &device_id, 1, "good morning");
        msg.payload = serde_json::json!({"echo": "evil command"});

        let result = verify_signed_message(&msg, &mut ctx);
        assert!(result.is_err(), "Tampered echo.ping should be rejected");
    }

    #[test]
    fn test_echo_loop_unsigned_message_rejected() {
        let (_sk, vk, _db, device_id) = setup_device_and_db();
        let (sk2, _vk2) = generate_keypair();

        let mut ctx = VerifyContext::new(Utc::now().timestamp_millis());
        ctx.add_trusted_device(&device_id, vk);

        let unsigned_msg = make_echo_ping_message(&sk2, "unknown_device", 1, "hello");

        let result = verify_signed_message(&unsigned_msg, &mut ctx);
        assert!(result.is_err(), "Unsigned/untrusted message should be rejected");
    }

    #[test]
    fn test_echo_loop_multiple_messages() {
        let (sk, vk, _db, device_id) = setup_device_and_db();

        let mut ctx = VerifyContext::new(Utc::now().timestamp_millis());
        ctx.add_trusted_device(&device_id, vk);

        for i in 1..=3 {
            ctx.now_ms = Utc::now().timestamp_millis();
            let msg = make_echo_ping_message(&sk, &device_id, i,
                &format!("message {}", i));
            let result = verify_signed_message(&msg, &mut ctx);
            assert!(result.is_ok(), "Message {} should pass: {:?}", i, result);

            let content = msg.payload["echo"].as_str().unwrap();
            let response = echo::handle_echo(content);
            assert_eq!(response, format!("echo.pong: message {}", i));
        }
    }

    #[test]
    fn test_echo_loop_seq_must_increase() {
        let (sk, vk, _db, device_id) = setup_device_and_db();

        let mut ctx = VerifyContext::new(Utc::now().timestamp_millis());
        ctx.add_trusted_device(&device_id, vk);

        let msg1 = make_echo_ping_message(&sk, &device_id, 10, "first");
        assert!(verify_signed_message(&msg1, &mut ctx).is_ok());

        ctx.now_ms = Utc::now().timestamp_millis();
        let msg2 = make_echo_ping_message(&sk, &device_id, 5, "bad seq");
        let result = verify_signed_message(&msg2, &mut ctx);
        assert!(result.is_err(), "Seq must increase, but got: {:?}", result);
    }
}
