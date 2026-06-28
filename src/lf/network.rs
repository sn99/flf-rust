//! Network — LF/network.js lockstep application layer
//! Transport: BroadcastChannel (multi-tab) + optional window.__flf_peer_* PeerJS glue
use serde_json::Value;
use std::collections::VecDeque;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::{BroadcastChannel, MessageEvent};

#[derive(Clone, Debug)]
pub struct NetInputFrame {
    pub tu: u32,
    pub peer: u8,
    pub keys: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct SyncSnapshot {
    pub hp: Vec<i32>,
    pub x: Vec<i32>,
}

pub struct NetworkSession {
    pub active: bool,
    pub role: String,
    pub log: Vec<String>,
    pub server: String,
    pub peers: u32,
    pub build_version: String,
    pub room_id: String,
    pub local_peer_id: String,
    pub input_queue: VecDeque<NetInputFrame>,
    pub last_sync_tu: u32,
    pub is_host: bool,
    pub verify_last: Option<String>,
    pub verify_error: bool,
    channel: Option<BroadcastChannel>,
    _on_msg: Option<Closure<dyn FnMut(MessageEvent)>>,
}

impl NetworkSession {
    pub fn new() -> Self {
        let pid = format!("rust-{}", (js_sys::Math::random() * 1_000_000.0) as u64);
        Self {
            active: false,
            role: String::new(),
            log: vec![
                "Network: BroadcastChannel lockstep (two tabs, same room id).".into(),
                "Optional PeerJS: define window.__flf_peer_connect / __flf_peer_send.".into(),
            ],
            server: "http://lobby.projectf.hk".into(),
            peers: 0,
            build_version: "0.9.9-rust".into(),
            room_id: String::new(),
            local_peer_id: pid,
            input_queue: VecDeque::new(),
            last_sync_tu: 0,
            is_host: true,
            verify_last: None,
            verify_error: false,
            channel: None,
            _on_msg: None,
        }
    }

    fn ensure_inbox() {
        if let Some(win) = web_sys::window() {
            if !js_sys::Reflect::has(&win, &"__flf_net_inbox".into()).unwrap_or(false) {
                let _ = js_sys::Reflect::set(&win, &"__flf_net_inbox".into(), &js_sys::Array::new());
            }
        }
    }

    pub fn set_room_hint(&mut self, addr: &str) {
        let a = addr.trim();
        if a.starts_with("room-") {
            self.room_id = a.to_string();
        }
    }

    pub fn connect(&mut self, server: &str, role: &str) {
        Self::ensure_inbox();
        self.set_room_hint(server);
        self.server = server.to_string();
        self.role = role.to_string();
        self.active = true;
        self.is_host = role != "passive" && role != "remote";
        if self.room_id.is_empty() || !self.room_id.starts_with("room-") {
            self.room_id = format!("room-{}", (js_sys::Date::now() as u64) % 1_000_000);
        }
        self.append_log(&format!("role={} room={} peer={}", role, self.room_id, self.local_peer_id));

        let ch_name = format!("flf-lockstep-{}", self.room_id);
        if let Ok(ch) = BroadcastChannel::new(&ch_name) {
            let on_msg = Closure::wrap(Box::new(move |ev: MessageEvent| {
                if let Ok(s) = js_sys::JSON::stringify(&ev.data()) {
                    let s = s.as_string().unwrap_or_default();
                    if let Some(win) = web_sys::window() {
                        Self::ensure_inbox();
                        if let Ok(inbox) = js_sys::Reflect::get(&win, &"__flf_net_inbox".into()) {
                            if let Some(arr) = inbox.dyn_ref::<js_sys::Array>() {
                                arr.push(&JsValue::from_str(&s));
                            }
                        }
                    }
                }
            }) as Box<dyn FnMut(_)>);
            let _ = ch.set_onmessage(Some(on_msg.as_ref().unchecked_ref()));
            self._on_msg = Some(on_msg);
            self.channel = Some(ch);
            self.peers = 2;
            self.append_log(&format!("BroadcastChannel {} ready — open 2nd tab, connect passive, address={}", ch_name, self.room_id));
        } else {
            self.peers = 1;
            self.append_log("BroadcastChannel unavailable.");
        }

        if let Some(win) = web_sys::window() {
            if let Ok(f) = js_sys::Reflect::get(&win, &"__flf_peer_connect".into()) {
                if f.is_function() {
                    let args = js_sys::Array::new();
                    args.push(&JsValue::from_str(server));
                    args.push(&JsValue::from_str(role));
                    args.push(&JsValue::from_str(&self.room_id));
                    let _ = js_sys::Reflect::apply(&f.into(), &win, &args);
                    self.append_log("PeerJS __flf_peer_connect invoked.");
                }
            }
        }
    }

    pub fn disconnect(&mut self) {
        self.active = false;
        self.peers = 0;
        self.input_queue.clear();
        if let Some(ch) = self.channel.take() {
            let _ = ch.close();
        }
        self._on_msg = None;
        self.append_log("Disconnected.");
    }

    pub fn push_local_input(&mut self, tu: u32, keys: Vec<String>) {
        if !self.active {
            return;
        }
        self.input_queue.push_back(NetInputFrame {
            tu,
            peer: 0,
            keys: keys.clone(),
        });
        while self.input_queue.len() > 120 {
            self.input_queue.pop_front();
        }
        self.last_sync_tu = tu;
        let payload = serde_json::json!({
            "tu": tu,
            "control": [keys],
            "peer": self.local_peer_id,
            "verify": self.verify_last,
        });
        let s = payload.to_string();
        if let Some(ch) = &self.channel {
            if let Ok(js) = js_sys::JSON::parse(&s) {
                let _ = ch.post_message(&js);
            }
        }
        if let Some(win) = web_sys::window() {
            if let Ok(send) = js_sys::Reflect::get(&win, &"__flf_peer_send".into()) {
                if send.is_function() {
                    let args = js_sys::Array::of1(&JsValue::from_str(&s));
                    let _ = js_sys::Reflect::apply(&send.into(), &win, &args);
                }
            }
        }
    }

    pub fn poll_remote(&mut self) -> Vec<NetInputFrame> {
        Self::ensure_inbox();
        let mut out = vec![];
        if let Some(win) = web_sys::window() {
            if let Ok(inbox) = js_sys::Reflect::get(&win, &"__flf_net_inbox".into()) {
                if let Some(arr) = inbox.dyn_ref::<js_sys::Array>() {
                    let n = arr.length();
                    for i in 0..n {
                        if let Some(s) = arr.get(i).as_string() {
                            if let Ok(v) = serde_json::from_str::<Value>(&s) {
                                if let Some(ver) = v.get("verify").and_then(|x| x.as_str()) {
                                    self.compare_verify(Some(ver));
                                }
                                let tu = v["tu"].as_u64().unwrap_or(0) as u32;
                                let keys: Vec<String> = v["control"]
                                    .as_array()
                                    .and_then(|a| a.first())
                                    .and_then(|k| k.as_array())
                                    .map(|a| {
                                        a.iter()
                                            .filter_map(|x| x.as_str().map(|s| s.to_string()))
                                            .collect()
                                    })
                                    .unwrap_or_default();
                                // ignore our own echo
                                let peer = v["peer"].as_str().unwrap_or("");
                                if peer != self.local_peer_id {
                                    out.push(NetInputFrame { tu, peer: 1, keys });
                                }
                            }
                        }
                    }
                    let _ = js_sys::Reflect::set(&win, &"__flf_net_inbox".into(), &js_sys::Array::new());
                }
            }
        }
        out
    }

    pub fn poll_inputs(&mut self) -> Vec<NetInputFrame> {
        self.input_queue.drain(..).collect()
    }

    pub fn set_verify_hp(&mut self, hps: &[f64]) {
        let s: Vec<i32> = hps.iter().map(|h| *h as i32).collect();
        self.verify_last = Some(format!("{:?}", s));
    }

    pub fn compare_verify(&mut self, remote: Option<&str>) {
        if let (Some(a), Some(b)) = (self.verify_last.as_deref(), remote) {
            if a != b && !self.verify_error {
                self.verify_error = true;
                self.append_log("SYNC ERROR: verify mismatch (desync).");
            }
        }
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
