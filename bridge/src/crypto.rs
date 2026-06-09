use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

pub fn generate_keypair() -> (SigningKey, VerifyingKey) {
    let signing_key = SigningKey::generate(&mut OsRng);
    let verifying_key = signing_key.verifying_key();
    (signing_key, verifying_key)
}

pub fn public_key_to_base64(vk: &VerifyingKey) -> String {
    BASE64.encode(vk.as_bytes())
}

pub fn public_key_from_base64(s: &str) -> Result<VerifyingKey, String> {
    let bytes = BASE64.decode(s).map_err(|e| format!("base64 decode: {}", e))?;
    VerifyingKey::from_bytes(&bytes.try_into().map_err(|_| "invalid public key length".to_string())?)
        .map_err(|e| format!("invalid public key: {}", e))
}

pub fn signature_to_base64(sig: &Signature) -> String {
    BASE64.encode(sig.to_bytes())
}

pub fn signature_from_base64(s: &str) -> Result<Signature, String> {
    let bytes = BASE64.decode(s).map_err(|e| format!("base64 decode: {}", e))?;
    Signature::from_slice(&bytes).map_err(|e| format!("invalid signature: {}", e))
}

pub fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

pub fn build_signing_string(
    message_id: &str,
    device_id: &str,
    msg_type: &str,
    timestamp_ms: i64,
    nonce: &str,
    seq: i64,
    payload_sha256: &str,
) -> String {
    format!(
        "v1\n\
         message_id={}\n\
         device_id={}\n\
         type={}\n\
         timestamp_ms={}\n\
         nonce={}\n\
         seq={}\n\
         payload_sha256={}",
        message_id, device_id, msg_type, timestamp_ms, nonce, seq, payload_sha256
    )
}

pub fn sign(signing_key: &SigningKey, signing_string: &str) -> Signature {
    signing_key.sign(signing_string.as_bytes())
}

pub fn verify(vk: &VerifyingKey, signing_string: &str, signature: &Signature) -> Result<(), String> {
    vk.verify(signing_string.as_bytes(), signature)
        .map_err(|e| format!("BAD_SIGNATURE: {}", e))
}

pub fn canonical_payload_hash(payload: &serde_json::Value) -> Result<String, String> {
    let sorted = sort_json_keys(payload);
    let canonical = serde_json::to_string(&sorted)
        .map_err(|e| format!("INVALID_JSON: {}", e))?;
    Ok(sha256_hex(canonical.as_bytes()))
}

fn sort_json_keys(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let mut entries: Vec<_> = map.iter().collect();
            entries.sort_by(|(a, _), (b, _)| a.cmp(b));
            let sorted: serde_json::Map<String, serde_json::Value> = entries
                .into_iter()
                .map(|(k, v)| (k.clone(), sort_json_keys(v)))
                .collect();
            serde_json::Value::Object(sorted)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(sort_json_keys).collect())
        }
        other => other.clone(),
    }
}

pub fn compute_payload_sha256(payload: &serde_json::Value) -> String {
    canonical_payload_hash(payload).unwrap_or_else(|_| {
        sha256_hex(serde_json::to_string(payload).unwrap_or_default().as_bytes())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generation() {
        let (_sk, vk) = generate_keypair();
        let vk_b64 = public_key_to_base64(&vk);
        let vk2 = public_key_from_base64(&vk_b64).unwrap();
        assert_eq!(vk, vk2);
    }

    #[test]
    fn test_sign_and_verify() {
        let (sk, vk) = generate_keypair();

        let signing_string = build_signing_string(
            "msg_001",
            "phone_abc",
            "session.input",
            1781000000000,
            "X0Jz2N8cCj9YhWm4xQw=",
            1042,
            "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08",
        );

        let signature = sign(&sk, &signing_string);
        assert!(verify(&vk, &signing_string, &signature).is_ok());
    }

    #[test]
    fn test_tampered_payload_rejected() {
        let (sk, vk) = generate_keypair();

        let signing_string = build_signing_string(
            "msg_001", "phone_abc", "session.input", 1781000000000,
            "X0Jz2N8cCj9YhWm4xQw=", 1042,
            "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08",
        );

        let signature = sign(&sk, &signing_string);

        let tampered = build_signing_string(
            "msg_001", "phone_abc", "session.input", 1781000000000,
            "X0Jz2N8cCj9YhWm4xQw=", 1042,
            "0000000000000000000000000000000000000000000000000000000000000000",
        );

        let result = verify(&vk, &tampered, &signature);
        assert!(result.is_err(), "Tampered payload should be rejected");
    }

    #[test]
    fn test_bad_signature_rejected() {
        let (_sk, vk) = generate_keypair();
        let (sk2, _) = generate_keypair();

        let signing_string = build_signing_string(
            "msg_001", "phone_abc", "session.input", 1781000000000,
            "X0Jz2N8cCj9YhWm4xQw=", 1042,
            "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08",
        );

        let bad_sig = sign(&sk2, &signing_string);
        let result = verify(&vk, &signing_string, &bad_sig);
        assert!(result.is_err(), "Bad signature should be rejected");
    }
}
