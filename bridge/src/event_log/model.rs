use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub event_id: String,
    pub session_id: Option<String>,
    pub seq: i64,
    pub event_type: EventType,
    pub content: Option<String>,
    pub payload_json: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum EventType {
    #[serde(rename = "session.created")]
    SessionCreated,
    #[serde(rename = "session.started")]
    SessionStarted,
    #[serde(rename = "session.stopped")]
    SessionStopped,
    #[serde(rename = "session.failed")]
    SessionFailed,
    #[serde(rename = "user.input")]
    UserInput,
    #[serde(rename = "agent.output")]
    AgentOutput,
    #[serde(rename = "agent.error")]
    AgentError,
    #[serde(rename = "device.paired")]
    DevicePaired,
    #[serde(rename = "device.revoked")]
    DeviceRevoked,
    #[serde(rename = "system.notice")]
    SystemNotice,
}
