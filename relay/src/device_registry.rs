use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub device_id: String,
    pub device_type: String,
    pub name: String,
    pub status: String,
}

pub struct DeviceRegistry {
    devices: Mutex<HashMap<String, DeviceInfo>>,
}

impl DeviceRegistry {
    pub fn new() -> Self {
        DeviceRegistry {
            devices: Mutex::new(HashMap::new()),
        }
    }

    pub fn register(&self, device: DeviceInfo) {
        let mut devices = self.devices.lock().unwrap();
        devices.insert(device.device_id.clone(), device);
    }

    pub fn get(&self, device_id: &str) -> Option<DeviceInfo> {
        let devices = self.devices.lock().unwrap();
        devices.get(device_id).cloned()
    }

    pub fn remove(&self, device_id: &str) {
        let mut devices = self.devices.lock().unwrap();
        devices.remove(device_id);
    }
}
