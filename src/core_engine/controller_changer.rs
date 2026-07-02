//! Port of core/controller-changer.js — key rebind UI helpers

use crate::core_engine::controller::Controller;
use crate::core_engine::util;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

/// Render a keychanger table into `host` for each controller (F.LF keychanger).
pub fn render_keychanger(host: &HtmlElement, controllers: &[Controller], player_names: &[String]) {
    let mut html = String::from(r#"<div class="keychanger"><table class="keychanger_table"><tr>"#);
    for (i, _c) in controllers.iter().enumerate() {
        let name = player_names
            .get(i)
            .map(|s| s.as_str())
            .unwrap_or("player");
        html.push_str(&format!(
            "<th colspan='2' data-slot='{i}' class='kc_name'>{name}</th>"
        ));
    }
    html.push_str("</tr>");
    let actions = ["up", "down", "left", "right", "def", "jump", "att"];
    for action in actions {
        html.push_str("<tr>");
        html.push_str(&format!("<td class='kc_action'>{action}</td>"));
        for (i, c) in controllers.iter().enumerate() {
            let key = c.config_key(action).unwrap_or("");
            html.push_str(&format!(
                "<td class='kc_key' data-slot='{i}' data-action='{action}' tabindex='0'>{key}</td>"
            ));
        }
        html.push_str("</tr>");
    }
    html.push_str("</table></div>");
    html.push_str(
        r#"<p class="cs_hint">Click a key cell, then press a key to rebind. Esc/OK saves.</p>"#,
    );
    host.set_inner_html(&html);
}

/// Attach click handlers that invoke `on_select(slot, action)`.
pub fn bind_keychanger_clicks(
    host: &HtmlElement,
    on_select: std::rc::Rc<std::cell::RefCell<dyn FnMut(usize, String)>>,
) {
    if let Ok(nodes) = host.query_selector_all(".kc_key") {
        for i in 0..nodes.length() {
            if let Some(n) = nodes.item(i) {
                if let Ok(el) = n.dyn_into::<HtmlElement>() {
                    let slot = el
                        .get_attribute("data-slot")
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0);
                    let action = el
                        .get_attribute("data-action")
                        .unwrap_or_else(|| "att".into());
                    let cb = on_select.clone();
                    let act = action.clone();
                    let closure = wasm_bindgen::closure::Closure::wrap(Box::new(
                        move |_e: web_sys::MouseEvent| {
                            cb.borrow_mut()(slot, act.clone());
                        },
                    )
                        as Box<dyn FnMut(_)>);
                    let _ = el.add_event_listener_with_callback(
                        "click",
                        closure.as_ref().unchecked_ref(),
                    );
                    closure.forget();
                }
            }
        }
    }
}

/// Highlight active rebind cell.
pub fn highlight_rebind(host: &HtmlElement, slot: usize, action: &str) {
    if let Ok(nodes) = host.query_selector_all(".kc_key") {
        for i in 0..nodes.length() {
            if let Some(n) = nodes.item(i) {
                if let Ok(el) = n.dyn_into::<HtmlElement>() {
                    let s = el
                        .get_attribute("data-slot")
                        .and_then(|x| x.parse::<usize>().ok())
                        .unwrap_or(usize::MAX);
                    let a = el.get_attribute("data-action").unwrap_or_default();
                    let active = s == slot && a == action;
                    let _ = el
                        .style()
                        .set_property("background-color", if active { "#FAA" } else { "#EEE" });
                }
            }
        }
    }
}

pub fn hide(host: &HtmlElement) {
    util::hide(host);
}

pub fn show(host: &HtmlElement) {
    util::show(host);
}
