//! Network multiplayer shell (PeerJS lobby compatible UI only in v1)
pub struct NetworkSession {
    pub active: bool,
    pub log: Vec<String>,
}

impl NetworkSession {
    pub fn new() -> Self {
        Self { active: false, log: vec!["Network: connect via lobby (optional).".into()] }
    }
    pub fn append_log(&mut self, line: &str) {
        self.log.push(line.to_string());
    }
}
