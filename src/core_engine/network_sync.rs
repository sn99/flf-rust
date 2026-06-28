//! Synchronized frame pump akin to F.LF core/network setInterval(frame, ms)
//! Frame buffer: remote sends `{ f: { t, d } }` → push → consume when interval elapsed.

use std::collections::{HashMap, VecDeque};

#[derive(Clone, Debug)]
pub struct SyncFrame {
    pub time: u32,
    pub data: String,
}

#[derive(Default)]
pub struct NetSyncPump {
    pub time: u32,
    pub last_verify: HashMap<String, i32>,
    pub error: bool,
    pub buffer: VecDeque<SyncFrame>,
    pub target_interval_ms: u32,
    last_consume_ms: f64,
}

impl NetSyncPump {
    pub fn new(interval_ms: u32) -> Self {
        Self {
            target_interval_ms: interval_ms,
            ..Default::default()
        }
    }

    pub fn push_remote(&mut self, time: u32, data: String) {
        self.buffer.push_back(SyncFrame { time, data });
    }

    /// Try consume one buffered frame if enough wall time elapsed (F.LF frame()).
    pub fn try_consume(&mut self, now_ms: f64) -> Option<SyncFrame> {
        let front = self.buffer.front()?;
        let diff = now_ms - self.last_consume_ms;
        if diff <= self.target_interval_ms as f64 - 5.0 && self.last_consume_ms > 0.0 {
            return None;
        }
        if front.time != self.time {
            self.error = true;
        }
        let fr = self.buffer.pop_front()?;
        self.time = self.time.wrapping_add(1);
        self.last_consume_ms = now_ms;
        Some(fr)
    }

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
