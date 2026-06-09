use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayConfig {
    pub listen_addr: String,
    pub db_path: String,
}

impl Default for RelayConfig {
    fn default() -> Self {
        RelayConfig {
            listen_addr: "127.0.0.1:8080".to_string(),
            db_path: "relay.db".to_string(),
        }
    }
}
