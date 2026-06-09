use crate::db::BridgeDb;
use crate::event_log::model::{Event, EventType};
use chrono::Utc;
use rand::Rng;

pub struct EventLog {
    pub db: BridgeDb,
}

impl EventLog {
    pub fn new_db(db: BridgeDb) -> Self {
        EventLog { db }
    }

    pub fn record(
        &self,
        session_id: Option<&str>,
        event_type: EventType,
        content: Option<&str>,
        payload_json: Option<&str>,
    ) -> Result<Event, String> {
        let event_id = generate_event_id();
        let seq = self.db.next_event_seq(session_id)?;
        let created_at = Utc::now().to_rfc3339();

        self.db.add_event(&event_id, session_id, seq,
            &serde_json::to_string(&event_type).unwrap_or_default().trim_matches('"'),
            content, payload_json)?;

        Ok(Event {
            event_id,
            session_id: session_id.map(|s| s.to_string()),
            seq,
            event_type,
            content: content.map(|s| s.to_string()),
            payload_json: payload_json.map(|s| s.to_string()),
            created_at,
        })
    }

    pub fn fetch_after(&self, session_id: Option<&str>, after_seq: i64) -> Result<Vec<Event>, String> {
        let records = self.db.fetch_events_after(session_id, after_seq)?;
        let events = records.into_iter().map(|r| {
            let event_type: EventType = serde_json::from_str(&format!("\"{}\"", r.event_type)).unwrap_or(EventType::SystemNotice);
            Event {
                event_id: r.event_id,
                session_id: r.session_id,
                seq: r.seq,
                event_type,
                content: r.content,
                payload_json: r.payload_json,
                created_at: r.created_at,
            }
        }).collect();
        Ok(events)
    }
}

fn generate_event_id() -> String {
    let mut rng = rand::thread_rng();
    let bytes: [u8; 12] = rng.gen();
    format!("evt_{}", hex::encode(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::BridgeDb;

    fn setup() -> EventLog {
        let db = BridgeDb::open(":memory:").unwrap();
        EventLog::new_db(db)
    }

    #[test]
    fn test_event_written() {
        let log = setup();

        let event = log.record(
            Some("sess_test"),
            EventType::UserInput,
            Some("hello"),
            None,
        ).unwrap();

        assert_eq!(event.seq, 1);
        assert_eq!(event.event_type, EventType::UserInput);
        assert_eq!(event.content, Some("hello".to_string()));

        let events = log.fetch_after(Some("sess_test"), 0).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].content, Some("hello".to_string()));
    }

    #[test]
    fn test_event_seq_increasing() {
        let log = setup();
        let sid = "sess_seq";

        let e1 = log.record(Some(sid), EventType::SessionCreated, None, None).unwrap();
        let e2 = log.record(Some(sid), EventType::UserInput, Some("hi"), None).unwrap();
        let e3 = log.record(Some(sid), EventType::AgentOutput, Some("pong"), None).unwrap();

        assert_eq!(e1.seq, 1);
        assert_eq!(e2.seq, 2);
        assert_eq!(e3.seq, 3);
    }

    #[test]
    fn test_restart_keeps_events() {
        let tmp = std::env::temp_dir().join("test_bridge_restart.db");
        let _ = std::fs::remove_file(&tmp);

        let db = BridgeDb::open(tmp.to_str().unwrap()).unwrap();
        let log = EventLog::new_db(db);
        let sid = "sess_persist";
        log.record(Some(sid), EventType::SessionCreated, None, None).unwrap();
        log.record(Some(sid), EventType::UserInput, Some("data"), None).unwrap();
        drop(log);

        let db = BridgeDb::open(tmp.to_str().unwrap()).unwrap();
        let log = EventLog::new_db(db);
        let events = log.fetch_after(Some(sid), 0).unwrap();
        assert_eq!(events.len(), 2, "File DB should persist events across reopens");

        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_fetch_after_seq() {
        let log = setup();
        let sid = "sess_fetch";

        log.record(Some(sid), EventType::SessionCreated, None, None).unwrap();
        log.record(Some(sid), EventType::UserInput, Some("one"), None).unwrap();
        log.record(Some(sid), EventType::UserInput, Some("two"), None).unwrap();
        log.record(Some(sid), EventType::UserInput, Some("three"), None).unwrap();

        let events = log.fetch_after(Some(sid), 1).unwrap();
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].seq, 2);

        let events = log.fetch_after(Some(sid), 3).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].seq, 4);
    }
}
