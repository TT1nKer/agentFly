use crate::db::BridgeDb;
use crate::crypto::public_key_from_base64;
use rand::Rng;
use chrono::Utc;

pub fn generate_pairing_code(db: &BridgeDb) -> Result<String, String> {
    let mut rng = rand::thread_rng();
    let code: u32 = rng.gen_range(100000..999999);
    let code_str = code.to_string();
    let expires_at = Utc::now().timestamp() + 600;
    db.set_pairing_code(code_str.clone(), expires_at)?;
    Ok(code_str)
}

pub fn verify_pairing_request(
    db: &BridgeDb,
    pairing_code: &str,
    public_key_b64: &str,
    device_name: &str,
    platform: &str,
) -> Result<String, String> {
    let valid = db.verify_pairing_code(pairing_code)?;
    if !valid {
        db.record_pairing_failure(pairing_code)?;
        return Err("Invalid or expired pairing code".to_string());
    }

    let _vk = public_key_from_base64(public_key_b64)?;

    let device_id = format!("device_{}", generate_device_suffix());
    let device = crate::db::TrustedDevice {
        device_id: device_id.clone(),
        name: device_name.to_string(),
        platform: platform.to_string(),
        public_key_base64: public_key_b64.to_string(),
        key_algorithm: "Ed25519".to_string(),
        status: "active".to_string(),
        last_seq: 0,
        created_at: Utc::now().to_rfc3339(),
        last_seen: None,
    };

    db.add_trusted_device(&device)?;
    Ok(device_id)
}

fn generate_device_suffix() -> String {
    use sha2::{Sha256, Digest};
    let mut rng = rand::thread_rng();
    let rand_bytes: [u8; 8] = rng.gen();
    let mut hasher = Sha256::new();
    hasher.update(rand_bytes);
    hasher.update(Utc::now().timestamp_millis().to_be_bytes());
    let hash = hasher.finalize();
    hex::encode(&hash[..8])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::generate_keypair;
    use crate::crypto::public_key_to_base64;

    fn new_test_db() -> BridgeDb {
        BridgeDb::open(":memory:").expect("Failed to open in-memory DB")
    }

    #[test]
    fn test_pairing_code_generated() {
        let db = new_test_db();
        let code = generate_pairing_code(&db).unwrap();
        assert_eq!(code.len(), 6);
        assert!(code.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_phone_public_key_registered() {
        let db = new_test_db();
        let code = generate_pairing_code(&db).unwrap();
        let (_, vk) = generate_keypair();
        let public_key_b64 = public_key_to_base64(&vk);

        let device_id = verify_pairing_request(&db, &code, &public_key_b64, "TestPhone", "android").unwrap();
        assert!(device_id.starts_with("device_"));

        let device = db.get_trusted_device(&device_id).unwrap().unwrap();
        assert_eq!(device.name, "TestPhone");
        assert_eq!(device.platform, "android");
        assert_eq!(device.public_key_base64, public_key_b64);
        assert_eq!(device.status, "active");
    }

    #[test]
    fn test_pairing_code_single_use() {
        let db = new_test_db();
        let code = generate_pairing_code(&db).unwrap();

        let (_, vk1) = generate_keypair();
        let pk1 = public_key_to_base64(&vk1);
        assert!(verify_pairing_request(&db, &code, &pk1, "Phone1", "ios").is_ok());

        let (_, vk2) = generate_keypair();
        let pk2 = public_key_to_base64(&vk2);
        let result = verify_pairing_request(&db, &code, &pk2, "Phone2", "android");
        assert!(result.is_err(), "Pairing code should be single-use");
    }

    #[test]
    fn test_wrong_pairing_code_rejected() {
        let db = new_test_db();
        generate_pairing_code(&db).unwrap();

        let (_, vk) = generate_keypair();
        let pk = public_key_to_base64(&vk);

        let result = verify_pairing_request(&db, "999999", &pk, "Phone", "ios");
        assert!(result.is_err(), "Wrong pairing code should be rejected");
    }

    #[test]
    fn test_expired_pairing_code_rejected() {
        let db = new_test_db();
        let code = "123456";
        let expired_at = Utc::now().timestamp() - 100;
        db.set_pairing_code(code.to_string(), expired_at).unwrap();

        let (_, vk) = generate_keypair();
        let pk = public_key_to_base64(&vk);

        let result = verify_pairing_request(&db, code, &pk, "Phone", "ios");
        assert!(result.is_err(), "Expired pairing code should be rejected");
    }

    #[test]
    fn test_pairing_lockout_after_5_failures() {
        let db = new_test_db();
        let code = generate_pairing_code(&db).unwrap();
        let (_, vk) = generate_keypair();
        let pk = public_key_to_base64(&vk);

        for _ in 0..4 {
            db.record_pairing_failure(&code).unwrap();
        }
        let result = verify_pairing_request(&db, &code, &pk, "Phone", "ios");
        assert!(result.is_ok(), "Should still work before 5th failure");

        let code2 = generate_pairing_code(&db).unwrap();
        for _ in 0..5 {
            db.record_pairing_failure(&code2).unwrap();
        }
        let result2 = verify_pairing_request(&db, &code2, &pk, "Phone2", "ios");
        assert!(result2.is_err(), "Should be locked after 5 failures");
    }
}
