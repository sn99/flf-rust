//! Soundpack — audio sprite like LF/soundpack.js + LF2_19/sound/soundpack.json
use crate::core_engine::util::document;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use wasm_bindgen::JsCast;
use web_sys::HtmlAudioElement;

#[derive(Clone, Debug)]
struct SpriteClip {
    start: f64,
    end: f64,
}

pub struct Soundpack {
    pub enabled: bool,
    asset_root: String,
    /// "1/016" style → clip; also bare "016"
    clips: HashMap<String, SpriteClip>,
    /// preferred audio URL for the big sprite file
    sprite_url: Option<String>,
    buffer: HashSet<String>,
    time: u32,
    loaded_meta: bool,
}

impl Soundpack {
    pub fn new() -> Self {
        Self {
            enabled: true,
            asset_root: String::new(),
            clips: HashMap::new(),
            sprite_url: None,
            buffer: HashSet::new(),
            time: 0,
            loaded_meta: false,
        }
    }

    pub fn set_root(&mut self, root: &str) {
        self.asset_root = root.trim_end_matches('/').to_string();
    }

    /// Call once when package UI/sound metadata is known (sync JSON already fetched in package)
    pub fn load_meta_json(&mut self, v: &Value) {
        self.loaded_meta = true;
        let file = v["file"].as_str().unwrap_or("sound/soundpack");
        let exts = v["ext"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.as_str())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| vec!["ogg".into(), "mp3".into()]);
        // prefer ogg then mp3
        for ext in &exts {
            if ext == "ac3" {
                continue;
            }
            self.sprite_url = Some(format!("{}/{}.{}", self.asset_root, file, ext));
            break;
        }
        if let Some(sound) = v["sound"].as_object() {
            for (id, clip) in sound {
                let start = clip["start"].as_f64().unwrap_or(0.0);
                let end = clip["end"].as_f64().unwrap_or(start + 0.2);
                let sc = SpriteClip { start, end };
                self.clips.insert(id.clone(), sc.clone());
                // also index as "1/id" common LF2 path (pack 1)
                self.clips.insert(format!("1/{}", id), sc.clone());
                self.clips.insert(format!("1{}", id), sc);
            }
        }
    }

    pub fn play(&mut self, path: &str) {
        if !self.enabled || path.is_empty() {
            return;
        }
        if self.buffer.contains(path) {
            return;
        }
        self.buffer.insert(path.to_string());

        // Resolve clip id from "1/016" or "016"
        let id = if path.contains('/') {
            path.split('/').last().unwrap_or(path)
        } else {
            path
        };
        let clip = self
            .clips
            .get(path)
            .or_else(|| self.clips.get(id))
            .or_else(|| self.clips.get(&format!("1/{}", id)))
            .cloned();

        if let (Some(url), Some(clip)) = (self.sprite_url.clone(), clip) {
            self.play_sprite(&url, clip.start, clip.end);
            return;
        }

        // fallback direct file
        for ext in ["ogg", "mp3", "wav"] {
            let url = format!("{}/sound/{}.{}", self.asset_root, id, ext);
            if self.try_play_full(&url) {
                return;
            }
        }
    }

    fn play_sprite(&self, url: &str, start: f64, end: f64) {
        let Ok(el) = document().create_element("audio") else {
            return;
        };
        let Ok(audio) = el.dyn_into::<HtmlAudioElement>() else {
            return;
        };
        audio.set_src(url);
        audio.set_preload("auto");
        let _ = audio.set_attribute("data-end", &end.to_string());
        // seek and play; stop via timeout approximated by duration
        let dur = (end - start).max(0.05);
        audio.set_current_time(start);
        let _ = audio.play();
        // schedule pause
        let audio2 = audio.clone();
        let end_t = end;
        let cb = wasm_bindgen::closure::Closure::once(move || {
            if audio2.current_time() >= end_t - 0.02 {
                let _ = audio2.pause();
            } else {
                // one more tick
                let _ = audio2.pause();
            }
        });
        let _ = web_sys::window().map(|w| {
            w.set_timeout_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(),
                (dur * 1000.0) as i32,
            )
        });
        cb.forget();
        let _ = dur;
    }

    fn try_play_full(&self, url: &str) -> bool {
        let Ok(el) = document().create_element("audio") else {
            return false;
        };
        let Ok(audio) = el.dyn_into::<HtmlAudioElement>() else {
            return false;
        };
        audio.set_src(url);
        audio.set_volume(0.45);
        audio.play().is_ok()
    }

    pub fn tu(&mut self) {
        self.time += 1;
        if self.time % 5 == 0 {
            self.buffer.clear();
        }
    }
}
