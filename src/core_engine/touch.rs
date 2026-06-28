//! Touch gamepad overlay (LF/touchcontroller.js subset)
use crate::core_engine::util::document;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

pub fn mount_touch_hint() {
    if let Ok(Some(holder)) = document().query_selector(".touch_control_holder") {
        if let Ok(el) = holder.dyn_into::<HtmlElement>() {
            el.set_inner_html(
                r#"<div class="touch_hint" style="position:fixed;bottom:8px;left:8px;color:#8af;font:12px sans-serif;opacity:0.7;pointer-events:none;">
                Touch: use on-screen keys when enabled (keyboard preferred on desktop)
                </div>"#,
            );
        }
    }
}
