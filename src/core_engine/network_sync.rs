//! Synchronized frame pump akin to F.LF core/network setInterval(frame, ms)
//! Host calls `begin_frame` / `end_frame` with optional remote control blob.

use std::collections::HashMap;

#[derive(Default)]
pub struct NetSyncPump {
    pub time: u32,
    pub last_verify: HashMap<String, i32>,
    pub error: bool,
}

impl NetSyncPump {
    pub fn tick(&mut self) -> u32 {
        self.time = self.time.wrapping_add(1);
        self.time
    }
    pub fn set_verify(&mut self, key: &str, val: i32) {
        self.last_verify.insert(key.to_string(), val);
    }
    pub fn compare(&mut self, other: &HashMap<String, i32>) -> bool {
        for (k, v) in &self.last_verify {
            if other.get(k) != Some(v) {
                self.error = true;
                return false;
            }
        }
        true
    }
}
