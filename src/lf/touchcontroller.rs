//! LF/touchcontroller.js — on-screen gamepad
use crate::core_engine::controller::Controller;
use crate::core_engine::util::document;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{Element, Event, HtmlElement};

pub struct TouchController {
    pub controller: Controller,
}

impl TouchController {
    pub fn new_gamepad() -> Self {
        let mut cfg = HashMap::new();
        for k in ["up", "down", "left", "right", "def", "jump", "att"] {
            cfg.insert(k.to_string(), k.to_string());
        }
        Self {
            controller: Controller::new_keyboard(cfg),
        }
    }

    pub fn mount(shared: Rc<RefCell<Controller>>) {
        let Ok(Some(holder)) = document().query_selector(".touch_control_holder") else {
            return;
        };
        let Ok(holder) = holder.dyn_into::<HtmlElement>() else {
            return;
        };
        holder.set_inner_html(
            r#"<div id="flf-touch" style="position:fixed;left:0;right:0;bottom:0;height:130px;display:flex;justify-content:space-between;padding:8px;z-index:50;background:linear-gradient(transparent,rgba(0,0,0,.4));pointer-events:auto">
            <div style="display:grid;grid-template-columns:44px 44px 44px;gap:4px;align-content:end">
              <span></span><button type="button" data-k="up" style="width:44px;height:44px">▲</button><span></span>
              <button type="button" data-k="left" style="width:44px;height:44px">◀</button>
              <button type="button" data-k="down" style="width:44px;height:44px">▼</button>
              <button type="button" data-k="right" style="width:44px;height:44px">▶</button>
            </div>
            <div style="display:flex;gap:8px;align-items:flex-end">
              <button type="button" data-k="def" style="width:52px;height:52px;border-radius:50%">D</button>
              <button type="button" data-k="jump" style="width:52px;height:52px;border-radius:50%">J</button>
              <button type="button" data-k="att" style="width:60px;height:60px;border-radius:50%;background:#c33;color:#fff;border:0">A</button>
            </div></div>"#,
        );
        let Ok(Some(root)) = document().query_selector("#flf-touch") else {
            return;
        };
        let s1 = shared.clone();
        let down = Closure::wrap(Box::new(move |ev: Event| {
            let mut el = ev.target().and_then(|e| e.dyn_into::<Element>().ok());
            while let Some(e) = el {
                if let Some(k) = e.get_attribute("data-k") {
                    s1.borrow_mut().keypress(&k);
                    break;
                }
                el = e.parent_element();
            }
        }) as Box<dyn FnMut(_)>);
        let _ = root.add_event_listener_with_callback("pointerdown", down.as_ref().unchecked_ref());
        down.forget();
    }
}
