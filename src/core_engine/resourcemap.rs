use std::collections::HashMap;
use serde_json::Value;

/// Maps resource paths (from original resourcemap.js)
pub struct ResourceMap {
    map: HashMap<String, String>,
}

impl ResourceMap {
    pub fn from_json(v: &Value) -> Self {
        let mut map = HashMap::new();
        if let Some(obj) = v.as_object() {
            for (k, val) in obj {
                if let Some(s) = val.as_str() {
                    map.insert(k.clone(), s.to_string());
                }
            }
        }
        Self { map }
    }

    pub fn resolve(&self, path: &str) -> String {
        self.map.get(path).cloned().unwrap_or_else(|| path.to_string())
    }
}
