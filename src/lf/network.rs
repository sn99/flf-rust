//! Network multiplayer — LF/network.js shell (full peer lockstep not ported; UI + session hooks)
use serde_json::Value;

#[derive(Clone, Debug)]
pub struct NetworkSession {
    pub active: bool,
    pub role: String, // "active" | "passive" | ""
    pub log: Vec<String>,
    pub server: String,
    pub peers: u32,
    pub build_version: String,
}

impl NetworkSession {
    pub fn new() -> Self {
        Self {
            active: false,
            role: String::new(),
            log: vec![
                "F.LF network: full PeerJS lockstep is not in the Rust port yet.".into(),
                "Use hosted F.LF at /game/game.html for multiplayer lobby.".into(),
            ],
            server: "http://lobby.projectf.hk".into(),
            peers: 0,
            build_version: "0.9.9-rust".into(),
        }
    }

    pub fn connect(&mut self, server: &str, role: &str) {
        self.server = server.to_string();
        self.role = role.to_string();
        self.active = true;
        self.append_log(&format!("Connecting to {} as {}…", server, role));
        self.append_log("ERROR: WebRTC/Peer transport not implemented in WASM port.");
        self.append_log("Play networked games via classic F.LF build on this site: /game/game.html");
    }

    pub fn disconnect(&mut self) {
        self.active = false;
        self.peers = 0;
        self.append_log("Disconnected.");
    }

    pub fn append_log(&mut self, line: &str) {
        self.log.push(line.to_string());
        if self.log.len() > 200 {
            self.log.drain(0..50);
        }
    }

    pub fn transfer_session_info(&self) -> Value {
        serde_json::json!({
            "buildversion": self.build_version,
            "player": [{"name":"player1"},{"name":"player2"}]
        })
    }
}
