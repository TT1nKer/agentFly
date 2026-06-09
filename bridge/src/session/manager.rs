use std::collections::HashMap;
use std::sync::Mutex;
use crate::session::model::{Session, SessionKind, SessionStatus};
use crate::db::BridgeDb;
use chrono::Utc;

pub struct SessionManager {
    sessions: Mutex<HashMap<String, Session>>,
    db: BridgeDb,
}

impl SessionManager {
    pub fn new(db_path: &str) -> Result<Self, String> {
        let db = BridgeDb::open(db_path)?;
        Ok(SessionManager {
            sessions: Mutex::new(HashMap::new()),
            db,
        })
    }

    pub fn create_session(
        &self,
        session_id: &str,
        kind: SessionKind,
        title: &str,
        workspace: &str,
    ) -> Result<Session, String> {
        let now = Utc::now().to_rfc3339();
        let tmux_name = format!("ac_{}", &session_id[..8.min(session_id.len())]);
        let status = SessionStatus::Created;

        let session = Session {
            session_id: session_id.to_string(),
            kind,
            title: title.to_string(),
            workspace: workspace.to_string(),
            tmux_name: Some(tmux_name.clone()),
            status: status.clone(),
            created_at: now.clone(),
            updated_at: now.clone(),
        };

        self.db.add_session(
            session_id,
            &format!("{:?}", session.kind).to_lowercase(),
            title,
            workspace,
            Some(&tmux_name),
            "created",
        )?;

        let mut sessions = self.sessions.lock().unwrap();
        sessions.insert(session_id.to_string(), session.clone());

        Ok(session)
    }

    pub fn stop_session(&self, session_id: &str) -> Result<(), String> {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get_mut(session_id) {
            session.status = SessionStatus::Stopped;
            session.updated_at = Utc::now().to_rfc3339();
        }
        Ok(())
    }

    pub fn get_session(&self, session_id: &str) -> Option<Session> {
        let sessions = self.sessions.lock().unwrap();
        sessions.get(session_id).cloned()
    }

    pub fn list_sessions(&self) -> Vec<Session> {
        let sessions = self.sessions.lock().unwrap();
        sessions.values().cloned().collect()
    }
}
