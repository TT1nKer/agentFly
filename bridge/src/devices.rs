use crate::db::BridgeDb;

pub fn list_devices(db: &BridgeDb) -> Result<Vec<crate::db::TrustedDevice>, String> {
    db.list_trusted_devices()
}

pub fn revoke_device(db: &BridgeDb, device_id: &str) -> Result<(), String> {
    db.revoke_device(device_id)
}

pub fn is_device_active(db: &BridgeDb, device_id: &str) -> Result<bool, String> {
    match db.get_trusted_device(device_id)? {
        Some(d) => Ok(d.status == "active"),
        None => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::TrustedDevice;

    fn new_test_db() -> BridgeDb {
        BridgeDb::open(":memory:").expect("Failed to open in-memory DB")
    }

    fn add_test_device(db: &BridgeDb, device_id: &str) {
        let device = TrustedDevice {
            device_id: device_id.to_string(),
            name: "TestPhone".to_string(),
            platform: "ios".to_string(),
            public_key_base64: "dGVzdF9wdWJsaWNfa2V5".to_string(),
            key_algorithm: "Ed25519".to_string(),
            status: "active".to_string(),
            last_seq: 0,
            created_at: chrono::Utc::now().to_rfc3339(),
            last_seen: None,
        };
        db.add_trusted_device(&device).unwrap();
    }

    #[test]
    fn test_revoke_device() {
        let db = new_test_db();
        add_test_device(&db, "device_001");

        assert!(is_device_active(&db, "device_001").unwrap());

        revoke_device(&db, "device_001").unwrap();
        assert!(!is_device_active(&db, "device_001").unwrap());

        let list = list_devices(&db).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].status, "revoked");
    }

    #[test]
    fn test_list_devices() {
        let db = new_test_db();
        add_test_device(&db, "device_a");
        add_test_device(&db, "device_b");

        let list = list_devices(&db).unwrap();
        assert_eq!(list.len(), 2);
    }
}
