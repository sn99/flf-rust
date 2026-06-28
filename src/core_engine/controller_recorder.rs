//! F.LF core/controller-recorder.js — record/replay key streams for demos & net debug
use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct RecEvent {
    pub tu: u32,
    pub action: String,
    pub down: bool,
}

#[derive(Default)]
pub struct ControllerRecorder {
    pub recording: bool,
    pub playing: bool,
    pub events: Vec<RecEvent>,
    play_idx: usize,
    queue: VecDeque<RecEvent>,
}

impl ControllerRecorder {
    pub fn start_record(&mut self) {
        self.recording = true;
        self.playing = false;
        self.events.clear();
    }
    pub fn stop_record(&mut self) {
        self.recording = false;
    }
    pub fn record(&mut self, tu: u32, action: &str, down: bool) {
        if self.recording {
            self.events.push(RecEvent {
                tu,
                action: action.into(),
                down,
            });
        }
    }
    pub fn start_playback(&mut self) {
        self.playing = true;
        self.recording = false;
        self.play_idx = 0;
        self.queue.clear();
    }
    /// Keys that should be down at this TU
    pub fn keys_at(&mut self, tu: u32) -> Vec<String> {
        if !self.playing {
            return vec![];
        }
        while self.play_idx < self.events.len() && self.events[self.play_idx].tu <= tu {
            let e = &self.events[self.play_idx];
            if e.down {
                self.queue.push_back(e.clone());
            } else {
                self.queue.retain(|q| q.action != e.action);
            }
            self.play_idx += 1;
        }
        self.queue.iter().map(|e| e.action.clone()).collect()
    }
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.events.iter().map(|e| {
            serde_json::json!({"tu": e.tu, "a": e.action, "d": e.down})
        }).collect::<Vec<_>>()).unwrap_or_else(|_| "[]".into())
    }
}
