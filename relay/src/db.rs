use rusqlite::{Connection, params};

pub struct RelayDb {
    conn: Connection,
}

impl RelayDb {
    pub fn open(path: &str) -> Result<Self, String> {
        let conn = Connection::open(path).map_err(|e| format!("DB open: {}", e))?;

        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS devices (
                device_id TEXT PRIMARY KEY,
                device_type TEXT NOT NULL,
                name TEXT NOT NULL,
                token_hash TEXT,
                status TEXT NOT NULL DEFAULT 'online',
                created_at TEXT NOT NULL,
                last_seen_at TEXT
            );

            CREATE TABLE IF NOT EXISTS messages (
                message_id TEXT PRIMARY KEY,
                from_device_id TEXT NOT NULL,
                to_device_id TEXT NOT NULL,
                type TEXT NOT NULL,
                payload_json TEXT NOT NULL,
                created_at TEXT NOT NULL,
                delivered_at TEXT
            );
        ").map_err(|e| format!("DB init: {}", e))?;

        Ok(RelayDb { conn })
    }

    pub fn register_device(&self, device_id: &str, device_type: &str, name: &str) -> Result<(), String> {
        let conn = &self.conn;
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT OR REPLACE INTO devices (device_id, device_type, name, status, created_at, last_seen_at) VALUES (?1, ?2, ?3, 'online', ?4, ?4)",
            params![device_id, device_type, name, now],
        ).map_err(|e| format!("register_device: {}", e))?;
        Ok(())
    }

    pub fn update_last_seen(&self, device_id: &str) -> Result<(), String> {
        let conn = &self.conn;
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE devices SET last_seen_at = ?1 WHERE device_id = ?2",
            params![now, device_id],
        ).map_err(|e| format!("update_last_seen: {}", e))?;
        Ok(())
    }

    pub fn store_message(&self, message_id: &str, from: &str, to: &str, msg_type: &str, payload_json: &str) -> Result<(), String> {
        let conn = &self.conn;
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO messages (message_id, from_device_id, to_device_id, type, payload_json, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![message_id, from, to, msg_type, payload_json, now],
        ).map_err(|e| format!("store_message: {}", e))?;
        Ok(())
    }
}
