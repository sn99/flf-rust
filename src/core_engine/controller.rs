use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::KeyboardEvent;
use crate::core_engine::util::window;

#[derive(Clone, Debug, Default)]
pub struct KeyState {
    pub pressed: bool,
    pub double: bool,
    pub last_down: f64,
}

/// Keyboard controller mapping LF2 actions to keys
pub struct Controller {
    pub config: HashMap<String, String>, // action -> key
    pub keys: HashMap<String, KeyState>,  // key -> state
    pub actions: Vec<String>,
    pub type_name: String,
    pub id: i32,
    double_interval_ms: f64,
}

impl Controller {
    pub fn new_keyboard(config: HashMap<String, String>) -> Self {
        let actions: Vec<String> = config.keys().cloned().collect();
        let mut keys = HashMap::new();
        for k in config.values() {
            keys.insert(k.clone(), KeyState::default());
        }
        Self {
            config,
            keys,
            actions,
            type_name: "keyboard".into(),
            id: 0,
            double_interval_ms: 200.0,
        }
    }

    pub fn clear_states(&mut self) {
        for st in self.keys.values_mut() {
            st.pressed = false;
            st.double = false;
        }
    }

    pub fn key_down(&mut self, key: &str, now: f64) {
        let key = key.to_lowercase();
        if let Some(st) = self.keys.get_mut(&key) {
            if now - st.last_down < self.double_interval_ms {
                st.double = true;
            }
            st.pressed = true;
            st.last_down = now;
        }
    }

    pub fn key_up(&mut self, key: &str) {
        let key = key.to_lowercase();
        if let Some(st) = self.keys.get_mut(&key) {
            st.pressed = false;
            st.double = false;
        }
    }

    /// Simulate AI keypress
    pub fn keypress(&mut self, action_or_dir: &str) {
        let key = self.config.get(action_or_dir).cloned()
            .or_else(|| Some(action_or_dir.to_string()));
        if let Some(k) = key {
            let now = js_sys::Date::now();
            self.key_down(&k, now);
        }
    }

    pub fn is_pressed(&self, action: &str) -> bool {
        if let Some(key) = self.config.get(action) {
            return self.keys.get(key).map(|s| s.pressed).unwrap_or(false);
        }
        false
    }

    pub fn is_double(&self, action: &str) -> bool {
        if let Some(key) = self.config.get(action) {
            return self.keys.get(key).map(|s| s.double).unwrap_or(false);
        }
        false
    }

    /// Bind global keyboard events into this controller (shared via Rc)
    pub fn bind_global(controllers: Rc<RefCell<Vec<Controller>>>) {
        let win = window();
        let c1 = controllers.clone();
        let on_down = Closure::wrap(Box::new(move |ev: KeyboardEvent| {
            let key = ev.key().to_lowercase();
            let now = js_sys::Date::now();
            for ctrl in c1.borrow_mut().iter_mut() {
                ctrl.key_down(&key, now);
            }
        }) as Box<dyn FnMut(_)>);
        let _ = win.add_event_listener_with_callback("keydown", on_down.as_ref().unchecked_ref());
        on_down.forget();

        let c2 = controllers.clone();
        let on_up = Closure::wrap(Box::new(move |ev: KeyboardEvent| {
            let key = ev.key().to_lowercase();
            for ctrl in c2.borrow_mut().iter_mut() {
                ctrl.key_up(&key);
            }
        }) as Box<dyn FnMut(_)>);
        let _ = win.add_event_listener_with_callback("keyup", on_up.as_ref().unchecked_ref());
        on_up.forget();
    }
}

pub fn default_p1_config() -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert("up".into(), "w".into());
    m.insert("down".into(), "x".into());
    m.insert("left".into(), "a".into());
    m.insert("right".into(), "d".into());
    m.insert("def".into(), "z".into());
    m.insert("jump".into(), "q".into());
    m.insert("att".into(), "s".into());
    m
}

pub fn default_p2_config() -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert("up".into(), "u".into());
    m.insert("down".into(), "m".into());
    m.insert("left".into(), "h".into());
    m.insert("right".into(), "k".into());
    m.insert("def".into(), ",".into());
    m.insert("jump".into(), "i".into());
    m.insert("att".into(), "j".into());
    m
}
