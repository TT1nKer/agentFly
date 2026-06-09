use std::collections::HashMap;
use std::sync::Mutex;

pub struct MessageRouter {
    pub routes: Mutex<HashMap<String, Vec<String>>>,
}

impl MessageRouter {
    pub fn new() -> Self {
        MessageRouter {
            routes: Mutex::new(HashMap::new()),
        }
    }

    pub fn register_route(&self, from: &str, to: &str) {
        let mut routes = self.routes.lock().unwrap();
        routes.entry(from.to_string()).or_default().push(to.to_string());
    }

    pub fn get_targets(&self, from: &str) -> Vec<String> {
        let routes = self.routes.lock().unwrap();
        routes.get(from).cloned().unwrap_or_default()
    }
}
