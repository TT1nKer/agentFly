use std::collections::VecDeque;
use std::sync::Mutex;

pub struct MessageStore {
    messages: Mutex<VecDeque<(String, String, String)>>,
    max_size: usize,
}

impl MessageStore {
    pub fn new(max_size: usize) -> Self {
        MessageStore {
            messages: Mutex::new(VecDeque::new()),
            max_size,
        }
    }

    pub fn store(&self, from: &str, to: &str, payload: &str) {
        let mut messages = self.messages.lock().unwrap();
        messages.push_back((from.to_string(), to.to_string(), payload.to_string()));
        while messages.len() > self.max_size {
            messages.pop_front();
        }
    }

    pub fn fetch_for_device(&self, device_id: &str) -> Vec<(String, String)> {
        let messages = self.messages.lock().unwrap();
        messages
            .iter()
            .filter(|(_, to, _)| to == device_id)
            .map(|(from, _, payload)| (from.clone(), payload.clone()))
            .collect()
    }
}
