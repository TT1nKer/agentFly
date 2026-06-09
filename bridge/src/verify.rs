use crate::crypto::{signature_from_base64, build_signing_string, verify, compute_payload_sha256};

#[derive(Debug)]
pub struct SignedMessage {
    pub version: i64,
    pub message_id: String,
    pub device_id: String,
    pub msg_type: String,
    pub timestamp_ms: i64,
    pub nonce: String,
    pub seq: i64,
    pub payload: serde_json::Value,
    pub payload_sha256: String,
    pub signature: String,
}

#[derive(Debug, PartialEq)]
pub enum VerifyError {
    InvalidJson,
    MissingField { field: String },
    DeviceNotTrusted,
    DeviceRevoked,
    BadTimestamp,
    ReplayDetected,
    BadSequence { expected_min: i64, got: i64 },
    BadPayloadHash,
    BadSignature,
    InternalError { reason: String },
}

impl std::fmt::Display for VerifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerifyError::InvalidJson => write!(f, "INVALID_JSON"),
            VerifyError::MissingField { field } => write!(f, "MISSING_FIELD: {}", field),
            VerifyError::DeviceNotTrusted => write!(f, "DEVICE_NOT_TRUSTED"),
            VerifyError::DeviceRevoked => write!(f, "DEVICE_REVOKED"),
            VerifyError::BadTimestamp => write!(f, "BAD_TIMESTAMP"),
            VerifyError::ReplayDetected => write!(f, "REPLAY_DETECTED"),
            VerifyError::BadSequence { expected_min, got } => write!(f, "BAD_SEQUENCE: expected > {}, got {}", expected_min, got),
            VerifyError::BadPayloadHash => write!(f, "BAD_PAYLOAD_HASH"),
            VerifyError::BadSignature => write!(f, "BAD_SIGNATURE"),
            VerifyError::InternalError { reason } => write!(f, "INTERNAL_ERROR: {}", reason),
        }
    }
}

pub fn parse_signed_message(json_str: &str) -> Result<SignedMessage, VerifyError> {
    let v: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|_| VerifyError::InvalidJson)?;

    let get_string = |key: &str| -> Result<String, VerifyError> {
        v.get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| VerifyError::MissingField { field: key.to_string() })
    };

    let get_i64 = |key: &str| -> Result<i64, VerifyError> {
        v.get(key)
            .and_then(|v| v.as_i64())
            .ok_or_else(|| VerifyError::MissingField { field: key.to_string() })
    };

    Ok(SignedMessage {
        version: get_i64("version")?,
        message_id: get_string("message_id")?,
        device_id: get_string("device_id")?,
        msg_type: get_string("type")?,
        timestamp_ms: get_i64("timestamp_ms")?,
        nonce: get_string("nonce")?,
        seq: get_i64("seq")?,
        payload: v.get("payload").cloned().ok_or_else(|| VerifyError::MissingField { field: "payload".to_string() })?,
        payload_sha256: get_string("payload_sha256")?,
        signature: get_string("signature")?,
    })
}

pub struct VerifyContext {
    pub trusted_public_keys: std::collections::HashMap<String, ed25519_dalek::VerifyingKey>,
    pub revoked_devices: std::collections::HashSet<String>,
    pub used_nonces: std::collections::HashSet<(String, String)>,
    pub device_last_seq: std::collections::HashMap<String, i64>,
    pub now_ms: i64,
    pub time_tolerance_ms: i64,
}

impl VerifyContext {
    pub fn new(now_ms: i64) -> Self {
        VerifyContext {
            trusted_public_keys: std::collections::HashMap::new(),
            revoked_devices: std::collections::HashSet::new(),
            used_nonces: std::collections::HashSet::new(),
            device_last_seq: std::collections::HashMap::new(),
            now_ms,
            time_tolerance_ms: 5 * 60 * 1000,
        }
    }

    pub fn add_trusted_device(&mut self, device_id: &str, vk: ed25519_dalek::VerifyingKey) {
        self.trusted_public_keys.insert(device_id.to_string(), vk);
    }

    pub fn revoke_device(&mut self, device_id: &str) {
        self.revoked_devices.insert(device_id.to_string());
        self.trusted_public_keys.remove(device_id);
    }

    pub fn add_used_nonce(&mut self, device_id: &str, nonce: &str) {
        self.used_nonces.insert((device_id.to_string(), nonce.to_string()));
    }

    pub fn update_last_seq(&mut self, device_id: &str, seq: i64) {
        self.device_last_seq.insert(device_id.to_string(), seq);
    }
}

pub fn verify_signed_message(msg: &SignedMessage, ctx: &mut VerifyContext) -> Result<(), VerifyError> {
    if msg.version != 1 {
        return Err(VerifyError::InternalError { reason: format!("unsupported version: {}", msg.version) });
    }

    if ctx.revoked_devices.contains(&msg.device_id) {
        return Err(VerifyError::DeviceRevoked);
    }

    let vk = ctx.trusted_public_keys.get(&msg.device_id)
        .ok_or(VerifyError::DeviceNotTrusted)?;

    let drift = (msg.timestamp_ms - ctx.now_ms).abs();
    if drift > ctx.time_tolerance_ms {
        return Err(VerifyError::BadTimestamp);
    }

    let nonce_key = (msg.device_id.clone(), msg.nonce.clone());
    if ctx.used_nonces.contains(&nonce_key) {
        return Err(VerifyError::ReplayDetected);
    }

    let last_seq = ctx.device_last_seq.get(&msg.device_id).copied().unwrap_or(0);
    if msg.seq <= last_seq {
        return Err(VerifyError::BadSequence { expected_min: last_seq, got: msg.seq });
    }

    let computed_hash = compute_payload_sha256(&msg.payload);
    if computed_hash != msg.payload_sha256 {
        return Err(VerifyError::BadPayloadHash);
    }

    let signing_string = build_signing_string(
        &msg.message_id,
        &msg.device_id,
        &msg.msg_type,
        msg.timestamp_ms,
        &msg.nonce,
        msg.seq,
        &msg.payload_sha256,
    );

    let signature = signature_from_base64(&msg.signature)
        .map_err(|e| VerifyError::InternalError { reason: e })?;

    verify(vk, &signing_string, &signature)
        .map_err(|_| VerifyError::BadSignature)?;

    ctx.add_used_nonce(&msg.device_id, &msg.nonce);
    ctx.update_last_seq(&msg.device_id, msg.seq);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::{generate_keypair, sign, signature_to_base64, build_signing_string, compute_payload_sha256};

    fn make_signed_message(
        signing_key: &ed25519_dalek::SigningKey,
        device_id: &str,
        msg_type: &str,
        payload: serde_json::Value,
    ) -> SignedMessage {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let message_id = format!("msg_{:04}", rng.gen::<u16>());
        let timestamp_ms = chrono::Utc::now().timestamp_millis();
        let nonce_bytes: [u8; 16] = rng.gen();
        let nonce = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, nonce_bytes);
        let seq = rng.gen_range(1000..9999);
        let payload_sha256 = compute_payload_sha256(&payload);

        let signing_string = build_signing_string(
            &message_id, device_id, msg_type, timestamp_ms, &nonce, seq, &payload_sha256,
        );
        let signature = sign(signing_key, &signing_string);
        let signature_b64 = signature_to_base64(&signature);

        SignedMessage {
            version: 1,
            message_id,
            device_id: device_id.to_string(),
            msg_type: msg_type.to_string(),
            timestamp_ms,
            nonce,
            seq,
            payload,
            payload_sha256,
            signature: signature_b64,
        }
    }

    #[test]
    fn test_valid_signature_passes() {
        let (sk, vk) = generate_keypair();
        let device_id = "phone_abc";
        let mut ctx = VerifyContext::new(chrono::Utc::now().timestamp_millis());
        ctx.add_trusted_device(device_id, vk);

        let msg = make_signed_message(&sk, device_id, "session.input",
            serde_json::json!({"session_id": "sess_001", "content": "hello"}));

        let result = verify_signed_message(&msg, &mut ctx);
        assert!(result.is_ok(), "Valid signature should pass, got: {:?}", result);
    }

    #[test]
    fn test_tampered_payload_rejected() {
        let (sk, vk) = generate_keypair();
        let device_id = "phone_abc";
        let mut ctx = VerifyContext::new(chrono::Utc::now().timestamp_millis());
        ctx.add_trusted_device(device_id, vk);

        let mut msg = make_signed_message(&sk, device_id, "session.input",
            serde_json::json!({"session_id": "sess_001", "content": "hello"}));
        msg.payload = serde_json::json!({"session_id": "sess_001", "content": "evil_command"});

        let result = verify_signed_message(&msg, &mut ctx);
        assert!(result.is_err(), "Tampered payload should be rejected");
        assert_eq!(result.unwrap_err(), VerifyError::BadPayloadHash);
    }

    #[test]
    fn test_bad_signature_rejected() {
        let (_sk, vk) = generate_keypair();
        let (sk2, _) = generate_keypair();
        let device_id = "phone_abc";
        let mut ctx = VerifyContext::new(chrono::Utc::now().timestamp_millis());
        ctx.add_trusted_device(device_id, vk);

        let mut msg = make_signed_message(&sk2, device_id, "session.input",
            serde_json::json!({"session_id": "sess_001", "content": "hello"}));
        msg.device_id = device_id.to_string();

        let result = verify_signed_message(&msg, &mut ctx);
        assert!(result.is_err(), "Bad signature should be rejected");
    }

    #[test]
    fn test_replay_nonce_rejected() {
        let (sk, vk) = generate_keypair();
        let device_id = "phone_abc";
        let mut ctx = VerifyContext::new(chrono::Utc::now().timestamp_millis());
        ctx.add_trusted_device(device_id, vk);

        let msg = make_signed_message(&sk, device_id, "session.input",
            serde_json::json!({"session_id": "sess_001", "content": "hello"}));

        let mut ctx2 = VerifyContext::new(chrono::Utc::now().timestamp_millis());
        ctx2.add_trusted_device(device_id, vk);

        assert!(verify_signed_message(&msg, &mut ctx).is_ok());
        let result = verify_signed_message(&msg, &mut ctx);
        assert!(result.is_err(), "Replay should be rejected");
        assert_eq!(result.unwrap_err(), VerifyError::ReplayDetected);
    }

    #[test]
    fn test_bad_seq_rejected() {
        let (sk, vk) = generate_keypair();
        let device_id = "phone_abc";
        let now = chrono::Utc::now().timestamp_millis();
        let mut ctx = VerifyContext::new(now);
        ctx.add_trusted_device(device_id, vk);

        let msg1 = make_signed_message(&sk, device_id, "session.input",
            serde_json::json!({"session_id": "sess_001", "content": "first"}));
        assert!(verify_signed_message(&msg1, &mut ctx).is_ok());

        ctx.now_ms = chrono::Utc::now().timestamp_millis();

        let msg2 = SignedMessage {
            version: 1,
            message_id: "msg_badseq".to_string(),
            device_id: device_id.to_string(),
            msg_type: "session.input".to_string(),
            timestamp_ms: ctx.now_ms,
            seq: msg1.seq - 1,
            payload: serde_json::json!({"session_id": "sess_001", "content": "bad_seq"}),
            payload_sha256: "".to_string(),
            nonce: "bad_seq_nonce_b64".to_string(),
            signature: "bad_seq_sig_b64".to_string(),
        };

        let result = verify_signed_message(&msg2, &mut ctx);
        assert!(result.is_err(), "Bad seq should be rejected");
    }

    #[test]
    fn test_device_not_trusted_rejected() {
        let (sk, _vk) = generate_keypair();
        let device_id = "unknown_phone";
        let mut ctx = VerifyContext::new(chrono::Utc::now().timestamp_millis());

        let msg = make_signed_message(&sk, device_id, "session.input",
            serde_json::json!({"session_id": "sess_001", "content": "hello"}));

        let result = verify_signed_message(&msg, &mut ctx);
        assert_eq!(result.unwrap_err(), VerifyError::DeviceNotTrusted);
    }
}
