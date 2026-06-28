//! Network multiplayer — LF/network.js shell
//! Full PeerJS lockstep is not in WASM; provides session UI hooks + local message queue
//! so manager can simulate lobby handshake and queue input frames for a future transport.
use serde_json::Value;
use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct NetInputFrame {
    pub tu: u32,
    pub peer: u8,
    pub keys: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct NetworkSession {
    pub active: bool,
    pub role: String, // "active" | "passive" | ""
    pub log: Vec<String>,
    pub server: String,
    pub peers: u32,
    pub build_version: String,
    pub room_id: String,
    pub local_peer_id: String,
    /// queued inputs for lockstep (filled by host/controller; drained by match when peer transport exists)
    pub input_queue: VecDeque<NetInputFrame>,
    pub last_sync_tu: u32,
}

impl NetworkSession {
    pub fn new() -> Self {
        let pid = format!("rust-{}", js_sys::Math::random().to_bits() % 1_000_000);
        Self {
            active: false,
            role: String::new(),
            log: vec![
                "F.LF network: PeerJS lockstep not fully ported in Rust WASM.".into(),
                "Lobby UI + input queue ready; use /game/game.html for real multiplayer.".into(),
            ],
            server: "http://lobby.projectf.hk".into(),
            peers: 0,
            build_version: "0.9.9-rust".into(),
            room_id: String::new(),
            local_peer_id: pid,
            input_queue: VecDeque::new(),
            last_sync_tu: 0,
        }
    }

    pub fn connect(&mut self, server: &str, role: &str) {
        self.server = server.to_string();
        self.role = role.to_string();
        self.active = true;
        self.room_id = format!("room-{}", (js_sys::Date::now() as u64) % 1_000_000);
        self.append_log(&format!("Connecting to {} as {}…", server, role));
        self.append_log(&format!("local peer id {}", self.local_peer_id));
        self.append_log(&format!("session room {}", self.room_id));
        // Simulate lobby handshake progress (UI feedback only)
        self.peers = 1;
        self.append_log("Lobby handshake: waiting for peer (transport stub).");
        self.append_log("ERROR: WebRTC/Peer transport not implemented in WASM port.");
        self.append_log("Play networked games via classic F.LF: /game/game.html");
    }

    pub fn disconnect(&mut self) {
        self.active = false;
        self.peers = 0;
        self.input_queue.clear();
        self.append_log("Disconnected.");
    }

    pub fn push_local_input(&mut self, tu: u32, keys: Vec<String>) {
        if !self.active {
            return;
        }
        self.input_queue.push_back(NetInputFrame {
            tu,
            peer: 0,
            keys,
        });
        if self.input_queue.len() > 120 {
            self.input_queue.pop_front();
        }
        self.last_sync_tu = tu;
    }

    pub fn poll_inputs(&mut self) -> Vec<NetInputFrame> {
        self.input_queue.drain(..).collect()
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
            "room": self.room_id,
            "peer": self.local_peer_id,
            "player": [{"name":"player1"},{"name":"player2"}]
        })
    }
}
