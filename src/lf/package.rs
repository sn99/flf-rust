//! Content package loader (LF2_19)
use crate::core_engine::resourcemap::ResourceMap;
use crate::core_engine::util::log;
use crate::lf::data::{parse_object_data, DataList, ObjectData, ObjectEntry};
use serde_json::Value;
use std::collections::HashMap;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

pub struct Package {
    pub root: String,
    pub data_list: DataList,
    pub objects: HashMap<i32, ObjectData>,
    pub backgrounds: HashMap<i32, Value>,
    pub properties: Value,
    pub ui: Value,
    pub resourcemap: ResourceMap,
    pub manifest: Value,
}

impl Package {
    pub async fn load(asset_root: &str) -> Result<Self, String> {
        let root = asset_root.trim_end_matches('/').to_string();
        let manifest: Value = fetch_json(&format!("{}/manifest.json", root)).await?;
        let data_path = manifest["data"].as_str().unwrap_or("data/data.json");
        let data_list: DataList = serde_json::from_value(
            fetch_json(&format!("{}/{}", root, data_path)).await?
        ).map_err(|e| e.to_string())?;

        let mut resourcemap = ResourceMap::from_json(&Value::Object(Default::default()));
        if let Some(rm) = manifest["resourcemap"].as_str() {
            if let Ok(v) = fetch_json(&format!("{}/{}", root, rm)).await {
                resourcemap = ResourceMap::from_json(&v);
            }
        }

        let mut properties = Value::Object(Default::default());
        if let Some(pe) = &data_list.properties {
            let file = pe.get("file").and_then(|f| f.as_str()).unwrap_or("");
            if !file.is_empty() {
                properties = fetch_json(&format!("{}/{}", root, file)).await.unwrap_or(properties);
            } else if pe.is_object() {
                properties = pe.clone();
            }
        }

        let mut ui = Value::Object(Default::default());
        // UI data is often embedded in data list UI entry
        if let Some(ui_entry) = &data_list.UI {
            if let Some(file) = ui_entry.get("file").and_then(|f| f.as_str()) {
                ui = fetch_json(&format!("{}/{}", root, file)).await.unwrap_or(ui);
            } else {
                ui = ui_entry.clone();
            }
        }
        // also try UI/UI.json
        if ui.as_object().map(|o| o.is_empty()).unwrap_or(true) {
            if let Ok(v) = fetch_json(&format!("{}/UI/UI.json", root)).await {
                ui = v;
            }
        }

        let mut pkg = Self {
            root: root.clone(),
            data_list,
            objects: HashMap::new(),
            backgrounds: HashMap::new(),
            properties,
            ui,
            resourcemap,
            manifest,
        };

        // Preload all non-lazy objects (weapons, effects, drinks, etc.)
        let ids: Vec<(i32, String, String)> = pkg.data_list.object.iter().map(|o| {
            (o.id, o.obj_type.clone(), o.file.clone())
        }).collect();
        for (id, typ, file) in ids {
            if file.is_empty() { continue; }
            // lazy: character and specialattack still load now for simplicity (full fidelity offline)
            if let Ok(v) = fetch_json(&format!("{}/{}", root, file)).await {
                let od = parse_object_data(id, &typ, &v);
                pkg.objects.insert(id, od);
            } else {
                log(&format!("failed to load object {}", file));
            }
        }

        // backgrounds
        for bg in &pkg.data_list.background.clone() {
            if bg.file.is_empty() { continue; }
            if let Ok(v) = fetch_json(&format!("{}/{}", root, bg.file)).await {
                pkg.backgrounds.insert(bg.id, v);
            }
        }

        log(&format!(
            "package loaded: {} objects, {} backgrounds",
            pkg.objects.len(),
            pkg.backgrounds.len()
        ));
        Ok(pkg)
    }

    pub fn object_entry(&self, id: i32) -> Option<&ObjectEntry> {
        self.data_list.object.iter().find(|o| o.id == id)
    }

    pub fn characters(&self) -> Vec<&ObjectEntry> {
        self.data_list.object.iter().filter(|o| o.obj_type == "character").collect()
    }

    pub fn resolve(&self, path: &str) -> String {
        self.resourcemap.resolve(path)
    }
}

async fn fetch_json(url: &str) -> Result<Value, String> {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);
    let request = Request::new_with_str_and_init(url, &opts).map_err(|e| format!("{:?}", e))?;
    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("fetch {}: {:?}", url, e))?;
    let resp: Response = resp_value.dyn_into().unwrap();
    if !resp.ok() {
        return Err(format!("HTTP {} for {}", resp.status(), url));
    }
    let json = JsFuture::from(resp.json().map_err(|e| format!("{:?}", e))?)
        .await
        .map_err(|e| format!("{:?}", e))?;
    serde_wasm_bindgen::from_value(json).map_err(|e| e.to_string())
}
