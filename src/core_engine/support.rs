use crate::core_engine::util::window;

pub fn has_local_storage() -> bool {
    window().local_storage().ok().flatten().is_some()
}

pub fn local_storage_get(key: &str) -> Option<String> {
    window()
        .local_storage()
        .ok()
        .flatten()
        .and_then(|s| s.get_item(key).ok().flatten())
}

pub fn local_storage_set(key: &str, value: &str) {
    if let Ok(Some(s)) = window().local_storage() {
        let _ = s.set_item(key, value);
    }
}

pub fn css2d_transform_supported() -> bool {
    true // modern browsers
}
