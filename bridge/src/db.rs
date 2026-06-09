use rusqlite::{Connection, params};

#[derive(Debug, Clone)]
pub struct TrustedDevice {
    pub device_id: String,
    pub name: String,
    pub platform: String,
    pub public_key_base64: String,
    pub key_algorithm: String,
    pub status: String,
    pub last_seq: i64,
    pub created_at: String,
    pub last_seen: Option<String>,
}

pub struct BridgeDb {
    conn: Connection,
}

impl BridgeDb {
    pub fn open(path: &str) -> Result<Self, String> {
        let conn = Connection::open(path).map_err(|e| format!("DB open: {}", e))?;

        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS trusted_devices (
                device_id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                platform TEXT NOT NULL,
                public_key_base64 TEXT NOT NULL,
                key_algorithm TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'active',
                last_seq INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                revoked_at TEXT,
                last_seen TEXT
            );

            CREATE TABLE IF NOT EXISTS used_nonces (
                device_id TEXT NOT NULL,
                nonce TEXT NOT NULL,
                timestamp_ms INTEGER NOT NULL,
                message_id TEXT NOT NULL,
                created_at TEXT NOT NULL,
                PRIMARY KEY (device_id, nonce)
            );

            CREATE TABLE IF NOT EXISTS sessions (
                session_id TEXT PRIMARY KEY,
                kind TEXT NOT NULL,
                title TEXT NOT NULL,
                workspace TEXT NOT NULL,
                tmux_name TEXT,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS events (
                event_id TEXT PRIMARY KEY,
                session_id TEXT,
                seq INTEGER NOT NULL,
                type TEXT NOT NULL,
                content TEXT,
                payload_json TEXT,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS pairing_codes (
                code TEXT PRIMARY KEY,
                expires_at INTEGER NOT NULL,
                used INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL
            );
        ").map_err(|e| format!("DB init: {}", e))?;

        Ok(BridgeDb { conn })
    }

    pub fn set_pairing_code(&self, code: String, expires_at: i64) -> Result<(), String> {
        let conn = &self.conn;
        conn.execute(
            "INSERT OR REPLACE INTO pairing_codes (code, expires_at, used, created_at) VALUES (?1, ?2, 0, ?3)",
            params![code, expires_at, chrono::Utc::now().to_rfc3339()],
        ).map_err(|e| format!("set_pairing_code: {}", e))?;
        Ok(())
    }

    pub fn verify_pairing_code(&self, code: &str) -> Result<bool, String> {
        let conn = &self.conn;
        let mut stmt = conn.prepare(
            "SELECT expires_at, used FROM pairing_codes WHERE code = ?1"
        ).map_err(|e| format!("verify_pairing_code: {}", e))?;

        let result = stmt.query_row(params![code], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, i32>(1)?))
        });

        match result {
            Ok((expires_at, used)) => {
                if used != 0 {
                    return Ok(false);
                }
                let now = chrono::Utc::now().timestamp();
                if now > expires_at {
                    return Ok(false);
                }
                conn.execute("UPDATE pairing_codes SET used = 1 WHERE code = ?1", params![code])
                    .map_err(|e| format!("mark_pairing_code: {}", e))?;
                Ok(true)
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(false),
            Err(e) => Err(format!("verify_pairing_code: {}", e)),
        }
    }

    pub fn add_trusted_device(&self, device: &TrustedDevice) -> Result<(), String> {
        let conn = &self.conn;
        conn.execute(
            "INSERT OR REPLACE INTO trusted_devices (device_id, name, platform, public_key_base64, key_algorithm, status, last_seq, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                device.device_id, device.name, device.platform,
                device.public_key_base64, device.key_algorithm,
                device.status, device.last_seq, device.created_at,
            ],
        ).map_err(|e| format!("add_trusted_device: {}", e))?;
        Ok(())
    }

    pub fn list_trusted_devices(&self) -> Result<Vec<TrustedDevice>, String> {
        let conn = &self.conn;
        let mut stmt = conn.prepare(
            "SELECT device_id, name, platform, public_key_base64, key_algorithm, status, last_seq, created_at, last_seen
             FROM trusted_devices ORDER BY created_at DESC"
        ).map_err(|e| format!("list_trusted_devices: {}", e))?;

        let devices = stmt.query_map([], |row| {
            Ok(TrustedDevice {
                device_id: row.get(0)?,
                name: row.get(1)?,
                platform: row.get(2)?,
                public_key_base64: row.get(3)?,
                key_algorithm: row.get(4)?,
                status: row.get(5)?,
                last_seq: row.get(6)?,
                created_at: row.get(7)?,
                last_seen: row.get(8)?,
            })
        }).map_err(|e| format!("list_trusted_devices: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("list_trusted_devices: {}", e))?;

        Ok(devices)
    }

    pub fn get_trusted_device(&self, device_id: &str) -> Result<Option<TrustedDevice>, String> {
        let conn = &self.conn;
        let mut stmt = conn.prepare(
            "SELECT device_id, name, platform, public_key_base64, key_algorithm, status, last_seq, created_at, last_seen
             FROM trusted_devices WHERE device_id = ?1 AND status = 'active'"
        ).map_err(|e| format!("get_trusted_device: {}", e))?;

        let result = stmt.query_row(params![device_id], |row| {
            Ok(TrustedDevice {
                device_id: row.get(0)?,
                name: row.get(1)?,
                platform: row.get(2)?,
                public_key_base64: row.get(3)?,
                key_algorithm: row.get(4)?,
                status: row.get(5)?,
                last_seq: row.get(6)?,
                created_at: row.get(7)?,
                last_seen: row.get(8)?,
            })
        });

        match result {
            Ok(d) => Ok(Some(d)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(format!("get_trusted_device: {}", e)),
        }
    }

    pub fn revoke_device(&self, device_id: &str) -> Result<(), String> {
        let conn = &self.conn;
        let rows = conn.execute(
            "UPDATE trusted_devices SET status = 'revoked', revoked_at = ?1 WHERE device_id = ?2",
            params![chrono::Utc::now().to_rfc3339(), device_id],
        ).map_err(|e| format!("revoke_device: {}", e))?;

        if rows == 0 {
            return Err("Device not found".to_string());
        }
        Ok(())
    }

    pub fn is_nonce_used(&self, device_id: &str, nonce: &str) -> Result<bool, String> {
        let conn = &self.conn;
        let mut stmt = conn.prepare(
            "SELECT COUNT(*) FROM used_nonces WHERE device_id = ?1 AND nonce = ?2"
        ).map_err(|e| format!("is_nonce_used: {}", e))?;

        let count: i64 = stmt.query_row(params![device_id, nonce], |row| row.get(0))
            .map_err(|e| format!("is_nonce_used: {}", e))?;

        Ok(count > 0)
    }

    pub fn add_used_nonce(&self, device_id: &str, nonce: &str, timestamp_ms: i64, message_id: &str) -> Result<(), String> {
        let conn = &self.conn;
        conn.execute(
            "INSERT INTO used_nonces (device_id, nonce, timestamp_ms, message_id, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![device_id, nonce, timestamp_ms, message_id, chrono::Utc::now().to_rfc3339()],
        ).map_err(|e| format!("add_used_nonce: {}", e))?;
        Ok(())
    }

    pub fn update_last_seq(&self, device_id: &str, seq: i64) -> Result<(), String> {
        let conn = &self.conn;
        conn.execute(
            "UPDATE trusted_devices SET last_seq = ?1, last_seen = ?2 WHERE device_id = ?3",
            params![seq, chrono::Utc::now().to_rfc3339(), device_id],
        ).map_err(|e| format!("update_last_seq: {}", e))?;
        Ok(())
    }

    pub fn add_event(&self, event_id: &str, session_id: Option<&str>, seq: i64, event_type: &str, content: Option<&str>, payload_json: Option<&str>) -> Result<(), String> {
        let conn = &self.conn;
        conn.execute(
            "INSERT INTO events (event_id, session_id, seq, type, content, payload_json, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![event_id, session_id, seq, event_type, content, payload_json, chrono::Utc::now().to_rfc3339()],
        ).map_err(|e| format!("add_event: {}", e))?;
        Ok(())
    }

    pub fn add_session(&self, session_id: &str, kind: &str, title: &str, workspace: &str, tmux_name: Option<&str>, status: &str) -> Result<(), String> {
        let conn = &self.conn;
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT OR REPLACE INTO sessions (session_id, kind, title, workspace, tmux_name, status, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
            params![session_id, kind, title, workspace, tmux_name, status, now],
        ).map_err(|e| format!("add_session: {}", e))?;
        Ok(())
    }

    pub fn next_event_seq(&self, session_id: Option<&str>) -> Result<i64, String> {
        let conn = &self.conn;
        match session_id {
            Some(sid) => {
                let mut stmt = conn.prepare(
                    "SELECT COALESCE(MAX(seq), 0) + 1 FROM events WHERE session_id = ?1"
                ).map_err(|e| format!("next_event_seq: {}", e))?;
                stmt.query_row(params![sid], |row| row.get(0))
                    .map_err(|e| format!("next_event_seq: {}", e))
            }
            None => {
                let mut stmt = conn.prepare(
                    "SELECT COALESCE(MAX(seq), 0) + 1 FROM events WHERE session_id IS NULL"
                ).map_err(|e| format!("next_event_seq: {}", e))?;
                stmt.query_row([], |row| row.get(0))
                    .map_err(|e| format!("next_event_seq: {}", e))
            }
        }
    }

    pub fn fetch_events_after(&self, session_id: Option<&str>, after_seq: i64) -> Result<Vec<EventRecord>, String> {
        let conn = &self.conn;
        match session_id {
            Some(sid) => {
                let mut stmt = conn.prepare(
                    "SELECT event_id, session_id, seq, type, content, payload_json, created_at
                     FROM events WHERE session_id = ?1 AND seq > ?2 ORDER BY seq ASC"
                ).map_err(|e| format!("fetch_events_after: {}", e))?;
                let events = stmt.query_map(params![sid, after_seq], |row| {
                    Ok(EventRecord {
                        event_id: row.get(0)?,
                        session_id: row.get(1)?,
                        seq: row.get(2)?,
                        event_type: row.get(3)?,
                        content: row.get(4)?,
                        payload_json: row.get(5)?,
                        created_at: row.get(6)?,
                    })
                }).map_err(|e| format!("fetch_events_after: {}", e))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| format!("fetch_events_after: {}", e))?;
                Ok(events)
            }
            None => {
                let mut stmt = conn.prepare(
                    "SELECT event_id, session_id, seq, type, content, payload_json, created_at
                     FROM events WHERE session_id IS NULL AND seq > ?2 ORDER BY seq ASC"
                ).map_err(|e| format!("fetch_events_after: {}", e))?;
                let events = stmt.query_map(params![after_seq], |row| {
                    Ok(EventRecord {
                        event_id: row.get(0)?,
                        session_id: row.get(1)?,
                        seq: row.get(2)?,
                        event_type: row.get(3)?,
                        content: row.get(4)?,
                        payload_json: row.get(5)?,
                        created_at: row.get(6)?,
                    })
                }).map_err(|e| format!("fetch_events_after: {}", e))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| format!("fetch_events_after: {}", e))?;
                Ok(events)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct EventRecord {
    pub event_id: String,
    pub session_id: Option<String>,
    pub seq: i64,
    pub event_type: String,
    pub content: Option<String>,
    pub payload_json: Option<String>,
    pub created_at: String,
}
