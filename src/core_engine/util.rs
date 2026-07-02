use wasm_bindgen::JsCast;
use web_sys::{Document, Element, HtmlElement, Window};

pub fn window() -> Window {
    web_sys::window().expect("no window")
}

pub fn document() -> Document {
    window().document().expect("no document")
}

pub fn qs(sel: &str) -> Option<Element> {
    document().query_selector(sel).ok().flatten()
}

pub fn div_class(class: &str) -> Option<HtmlElement> {
    qs(&format!(".{}", class))
        .and_then(|e| e.dyn_into::<HtmlElement>().ok())
}

pub fn show(el: &HtmlElement) {
    let _ = el.style().set_property("display", "block");
}

pub fn hide(el: &HtmlElement) {
    let _ = el.style().set_property("display", "none");
}

pub fn normalize_path(p: &str) -> String {
    let mut s = p.replace('\\', "/");
    if !s.is_empty() && !s.ends_with('/') {
        s.push('/');
    }
    s
}

pub fn location_parameters() -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    if let Ok(search) = window().location().search() {
        let q = search.trim_start_matches('?');
        for part in q.split('&') {
            if part.is_empty() { continue; }
            let mut it = part.splitn(2, '=');
            let k = it.next().unwrap_or("").to_string();
            let v = it.next().unwrap_or("").to_string();
            map.insert(k, v);
        }
    }
    map
}

pub fn set_text(el: &HtmlElement, text: &str) {
    el.set_inner_text(text);
}

pub fn log(msg: &str) {
    web_sys::console::log_1(&msg.into());
}

pub fn error(msg: &str) {
    web_sys::console::error_1(&msg.into());
}

/// F.LF core/util.make_array
pub fn make_array<T: Clone>(v: Option<&[T]>) -> Vec<T> {
    v.map(|s| s.to_vec()).unwrap_or_default()
}

/// F.LF core/util.push_unique
pub fn push_unique<T: PartialEq>(vec: &mut Vec<T>, item: T) {
    if !vec.contains(&item) {
        vec.push(item);
    }
}

/// F.LF core/util.extend_object — shallow merge into base map
pub fn extend_object(
    base: &mut serde_json::Map<String, serde_json::Value>,
    extra: &serde_json::Map<String, serde_json::Value>,
) {
    for (k, v) in extra {
        base.insert(k.clone(), v.clone());
    }
}

/// F.LF core/util.search_array
pub fn search_array<'a, T, F: Fn(&T) -> bool>(arr: &'a [T], pred: F) -> Option<&'a T> {
    arr.iter().find(|x| pred(x))
}

/// F.LF game/game.js util_div
pub fn util_div(class: &str) -> Option<HtmlElement> {
    div_class(class)
}
