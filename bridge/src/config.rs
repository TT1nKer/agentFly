use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeConfig {
    pub relay_url: String,
    pub bridge_id: String,
    pub bridge_name: String,
    pub db_path: String,
    pub workspace_root: String,
}

impl Default for BridgeConfig {
    fn default() -> Self {
        BridgeConfig {
            relay_url: "ws://localhost:8080/ws".to_string(),
            bridge_id: "bridge_001".to_string(),
            bridge_name: "my-macbook".to_string(),
            db_path: "bridge.db".to_string(),
            workspace_root: "./workspace".to_string(),
        }
    }
}
