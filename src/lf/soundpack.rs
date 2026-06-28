//! Soundpack — LF/soundpack.js via HTMLAudioElement (sprite IDs as file paths)
use crate::core_engine::util::document;
use std::collections::HashSet;
use wasm_bindgen::JsCast;
use web_sys::HtmlAudioElement;

pub struct Soundpack {
    pub enabled: bool,
    asset_root: String,
    /// paths played this TU (dedupe)
    buffer: HashSet<String>,
    time: u32,
}

impl Soundpack {
    pub fn new() -> Self {
        Self {
            enabled: true,
            asset_root: String::new(),
            buffer: HashSet::new(),
            time: 0,
        }
    }

    pub fn set_root(&mut self, root: &str) {
        self.asset_root = root.trim_end_matches('/').to_string();
    }

    /// Play LF2 sound path like `1/001` or `data/xxx` — map to sound files when possible
    pub fn play(&mut self, path: &str) {
        if !self.enabled || path.is_empty() {
            return;
        }
        if self.buffer.contains(path) {
            return;
        }
        self.buffer.insert(path.to_string());

        // Prefer full path under package; also try sound/ prefixes
        let candidates = [
            format!("{}/{}", self.asset_root, path),
            format!("{}/sound/{}", self.asset_root, path),
            format!("{}/{}", self.asset_root, path.trim_start_matches('/')),
        ];
        // Many LF2 sounds are in a sprite pack — play short beep via data URI fallback silent
        // Try .wav variants for known patterns
        for base in &candidates {
            for ext in ["", ".wav", ".ogg", ".mp3"] {
                let url = if ext.is_empty() && !base.contains('.') {
                    continue;
                } else if ext.is_empty() {
                    base.clone()
                } else if base.contains('.') {
                    base.clone()
                } else {
                    format!("{}{}", base, ext)
                };
                if self.try_play(&url) {
                    return;
                }
            }
        }
        // Frame sounds often "1/056" — cannot play without sprite sheet timing; no-op ok
    }

    fn try_play(&self, url: &str) -> bool {
        let Ok(audio) = document().create_element("audio") else {
            return false;
        };
        let Ok(audio) = audio.dyn_into::<HtmlAudioElement>() else {
            return false;
        };
        audio.set_src(url);
        audio.set_volume(0.5);
        match audio.play() {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub fn tu(&mut self) {
        self.time += 1;
        if self.time % 5 == 0 {
            self.buffer.clear();
        }
    }
}
