//! Game manager — closer to LF/manager.js UX
use crate::core_engine::controller::{self, Controller};
use crate::core_engine::sprite::CanvasRenderer;
use crate::core_engine::support;
use crate::core_engine::util::{self, document, hide, show};
use crate::lf::global;
use crate::lf::match_game::{Match, PlayerSetup};
use crate::lf::network::NetworkSession;
use crate::lf::package::Package;
use crate::lf::soundpack::Soundpack;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, KeyboardEvent, MouseEvent};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Screen {
    FrontPage,
    CharacterSelect,
    ComputerCount,
    VsDialog,
    Settings,
    Network,
    Gameplay,
}

pub struct Manager {
    package: Package,
    version: String,
    screen: Screen,
    renderer: CanvasRenderer,
    controllers: Rc<RefCell<Vec<Controller>>>,
    match_game: Option<Match>,
    sound: Soundpack,
    network: NetworkSession,
    slots: Vec<SelectSlot>,
    selected_bg: i32,
    num_computers: i32,
    computer_cursor: i32,
    vs_cursor: usize,
    last_tick: f64,
    acc: f64,
    fps_display: f64,
    frames_counted: u32,
    fps_timer: f64,
    running: Rc<Cell<bool>>,
    /// pending key rebind (controller slot, action name)
    rebind_target: Option<(usize, String)>,
    /// F4 cycles: normal → wide → maximize-ish
    view_mode: u8,
}

#[derive(Clone)]
struct SelectSlot {
    player_name: String,
    char_index: i32, // -1 = random
    team: i32,
    joined: bool,
    is_com: bool,
}

impl Manager {
    pub fn new(package: Package, version: &str) -> Result<Self, wasm_bindgen::JsValue> {
        let asset_root = package.root.clone();
        let mut renderer = CanvasRenderer::from_selector(".canvas", &asset_root)
            .map_err(|e| wasm_bindgen::JsValue::from_str(&e))?;
        // viewer height 400; panels drawn in canvas top
        renderer.set_size(global::WINDOW_WIDTH as u32, global::VIEWER_HEIGHT as u32);

        let mut controllers = vec![
            Controller::new_keyboard(controller::default_p1_config()),
            Controller::new_keyboard(controller::default_p2_config()),
        ];
        if let Some(raw) = support::local_storage_get("F.LF/settings") {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
                if let Some(arr) = v["control"].as_array() {
                    for (i, c) in arr.iter().enumerate() {
                        if i >= controllers.len() {
                            break;
                        }
                        if let Some(cfg) = c["config"].as_object() {
                            let mut map = std::collections::HashMap::new();
                            for (k, val) in cfg {
                                if let Some(s) = val.as_str() {
                                    map.insert(k.clone(), s.to_string());
                                }
                            }
                            if !map.is_empty() {
                                controllers[i] = Controller::new_keyboard(map);
                            }
                        }
                    }
                }
            }
        }
        let controllers = Rc::new(RefCell::new(controllers));
        Controller::bind_global(controllers.clone());
        crate::core_engine::touch::mount_touch_hint();

        let slots = vec![
            SelectSlot {
                player_name: "player1".into(),
                char_index: 0,
                team: 1,
                joined: false,
                is_com: false,
            },
            SelectSlot {
                player_name: "player2".into(),
                char_index: 1,
                team: 2,
                joined: false,
                is_com: false,
            },
        ];

        let first_bg = package
            .data_list
            .background
            .first()
            .map(|b| b.id)
            .unwrap_or(0);

        if let Some(el) = util::div_class("window_caption_title") {
            el.set_inner_text(version);
        }
        // favicon
        if let Ok(Some(head)) = document().query_selector("head") {
            if let Ok(link) = document().create_element("link") {
                let _ = link.set_attribute("rel", "icon");
                let _ = link.set_attribute(
                    "href",
                    &format!("{}/sprite/icon.png", asset_root),
                );
                let _ = head.append_child(&link);
            }
        }

        let mut mgr = Self {
            package,
            version: version.to_string(),
            screen: Screen::FrontPage,
            renderer,
            controllers,
            match_game: None,
            sound: Soundpack::new(),
            network: NetworkSession::new(),
            slots,
            selected_bg: first_bg,
            num_computers: 0,
            computer_cursor: 0,
            vs_cursor: 0,
            last_tick: js_sys::Date::now(),
            acc: 0.0,
            fps_display: 0.0,
            frames_counted: 0,
            fps_timer: js_sys::Date::now(),
            running: Rc::new(Cell::new(true)),
            rebind_target: None,
            view_mode: 0,
        };
        // apply UI bg colors
        if let Some(fp) = util::div_class("frontpage") {
            let col = mgr.package.ui["frontpage"]["bg_color"].as_str().unwrap_or("#10206c");
            let _ = fp.style().set_property("background-color", col);
        }
        crate::core_engine::touch::mount_touch_hint();
        let params = crate::core_engine::util::location_parameters();
        if params.contains_key("demo") {
            // auto start with COMs
            mgr.num_computers = 2;
            for s in &mut mgr.slots {
                s.joined = true;
            }
            mgr.start_match();
        } else {
            mgr.show_screen(Screen::FrontPage);
        }
        Ok(mgr)
    }

    fn show_screen(&mut self, s: Screen) {
        self.screen = s;
        let show_cls = |cls: &str, on: bool| {
            if let Some(el) = util::div_class(cls) {
                if on { show(&el); } else { hide(&el); }
            }
        };
        show_cls("frontpage", s == Screen::FrontPage);
        show_cls(
            "character_selection",
            matches!(s, Screen::CharacterSelect | Screen::ComputerCount | Screen::VsDialog),
        );
        show_cls("settings", s == Screen::Settings);
        show_cls("network_game", s == Screen::Network);
        show_cls("gameplay", s == Screen::Gameplay);

        match s {
            Screen::FrontPage => self.render_frontpage_dom(),
            Screen::CharacterSelect => self.render_charselect_dom(),
            Screen::ComputerCount => self.render_computer_dom(),
            Screen::VsDialog => self.render_vs_dom(),
            Screen::Settings => self.render_settings_dom(),
            Screen::Network => self.render_network_dom(),
            Screen::Gameplay => {}
        }
    }

    fn render_frontpage_dom(&self) {
        let ui = &self.package.ui;
        let dialog = &ui["frontpage_dialog"];
        let pic = dialog["pic"].as_str().unwrap_or("UI/frontpage_dialog.png");
        let bg = dialog["bg"].as_str().unwrap_or("UI/frontpage_dialog_bg.png");
        let x = dialog["x"].as_f64().unwrap_or(263.0);
        let y = dialog["y"].as_f64().unwrap_or(240.0);
        let w = dialog["width"].as_f64().unwrap_or(282.0);
        let h = dialog["height"].as_f64().unwrap_or(118.0);
        let root = &self.package.root;
        let html = format!(
            r#"<div class="fp_stage">
              <img class="fp_bg" src="{root}/UI/frontpage.png" alt=""/>
              <div class="fp_dialog" style="left:{x}px;top:{y}px;width:{w}px;height:{h}px;">
                <img class="fp_dialog_bg" src="{root}/{bg}" alt=""/>
                <img class="fp_dialog_pic" src="{root}/{pic}" alt=""/>
                <button type="button" class="menu_hit" data-i="0" style="top:16px;height:24px;" title="game start"></button>
                <button type="button" class="menu_hit" data-i="1" style="top:45px;height:26px;" title="network"></button>
                <button type="button" class="menu_hit" data-i="2" style="top:76px;height:27px;" title="settings"></button>
              </div>
            </div>"#
        );
        if let Some(el) = util::div_class("frontpage_content") {
            el.set_inner_html(&html);
        }
    }

    fn char_count(&self) -> usize {
        self.package.characters().len()
    }

    fn render_charselect_dom(&self) {
        let ui = &self.package.ui["character_selection"];
        let root = &self.package.root;
        let bg = ui["pic"].as_str().unwrap_or("UI/character_selection.png");
        let chars = self.package.characters();
        let posx = ui["posx"].as_array();
        let mut html = format!(
            r#"<div class="cs_stage"><img class="cs_bg" src="{root}/{bg}" alt=""/>"#
        );
        for (i, slot) in self.slots.iter().enumerate() {
            let px = posx
                .and_then(|a| a.get(i))
                .and_then(|v| v.as_f64())
                .unwrap_or(147.0 + i as f64 * 153.0);
            let py = 93.0;
            let (name, pic) = if !slot.joined {
                (
                    "Press attack".to_string(),
                    format!("{root}/UI/press_attack_to_join.png"),
                )
            } else if slot.char_index < 0 {
                ("Random".into(), format!("{root}/UI/random.png"))
            } else {
                let idx = slot.char_index as usize % chars.len().max(1);
                let c = &chars[idx];
                (c.name.clone(), format!("{root}/{}", c.pic))
            };
            let team_col = ["#afdcff", "#1946ff", "#ffffff", "#ff9b9b"]
                .get((slot.team as usize).saturating_sub(1) % 4)
                .unwrap_or(&"#fff");
            html.push_str(&format!(
                r#"<div class="cs_box" style="left:{px}px;top:{py}px;width:120px;height:116px;">
                  <img src="{pic}" width="100" height="100" alt=""/>
                  <div class="cs_name" style="color:{team_col}">{name}</div>
                  <div class="cs_meta">{pn} T{team}</div>
                </div>"#,
                pn = slot.player_name,
                team = slot.team,
            ));
        }
        html.push_str(&format!(
            r#"<div class="cs_hint">P1: A/D char · S join/att · Q ready &nbsp; P2: H/K · J · I &nbsp; Esc back · then computers</div>
            <div class="cs_bgrow">"#
        ));
        for bg in &self.package.data_list.background {
            let sel = if bg.id == self.selected_bg { " cs_bgsel" } else { "" };
            html.push_str(&format!(
                r#"<span class="bg_item{sel}" data-bg="{}">{}</span> "#,
                bg.id, bg.name
            ));
        }
        html.push_str("</div></div>");
        if let Some(el) = util::div_class("character_selection") {
            el.set_inner_html(&html);
            let col = ui["bg_color"].as_str().unwrap_or("#000");
            let _ = el.style().set_property("background-color", col);
        }
    }

    fn render_computer_dom(&self) {
        let root = &self.package.root;
        let mut html = format!(
            r#"<div class="cs_stage dim"><img class="cs_bg" src="{root}/UI/character_selection.png"/>
            <div class="modal_dlg" style="left:218px;top:180px;width:365px;height:111px;">
              <img src="{root}/UI/how_many_computer_players.png" width="365" height="111"/>
              <div class="com_row">"#
        );
        for n in 0..8 {
            let on = n == self.computer_cursor;
            let col = if on { "#fff" } else { "#5068c0" };
            html.push_str(&format!(
                r#"<span class="com_n" data-com="{n}" style="color:{col};border-color:{col}">{n}</span>"#
            ));
        }
        html.push_str(
            r#"</div><div class="cs_hint">←/→ or A/D choose · Attack/Enter confirm · Esc cancel</div></div></div>"#,
        );
        if let Some(el) = util::div_class("character_selection") {
            el.set_inner_html(&html);
        }
    }

    fn render_vs_dom(&self) {
        let root = &self.package.root;
        let labels = [
            "Vs mode start",
            "Reset",
            "Background → click list",
            "Difficulty (unused)",
            "Back",
            "OK",
        ];
        let mut html = format!(
            r#"<div class="cs_stage dim"><img class="cs_bg" src="{root}/UI/character_selection.png"/>
            <div class="modal_dlg vs" style="left:200px;top:100px;width:304px;">
              <img src="{root}/UI/dialog1.png" width="304" height="165" style="position:absolute;left:0;top:0;"/>
              <img src="{root}/UI/vs_mode_dialog.png" width="304" height="165" style="position:absolute;left:0;top:0;opacity:0.95;"/>
              <div class="vs_items">"#
        );
        for (i, lab) in labels.iter().enumerate() {
            let sel = if i == self.vs_cursor { " vs_sel" } else { "" };
            html.push_str(&format!(
                r#"<div class="vs_item{sel}" data-vs="{i}">{lab}</div>"#
            ));
        }
        html.push_str("</div></div></div>");
        if let Some(el) = util::div_class("character_selection") {
            el.set_inner_html(&html);
        }
    }

    fn render_settings_dom(&self) {
        let root = &self.package.root;
        let ctrls = self.controllers.borrow();
        let mut html = format!(
            r#"<div class="set_stage"><img src="{root}/UI/settings.png" class="set_bg" alt=""/>
            <div class="set_table"><table><tr><th>action</th><th>P1</th><th>P2</th></tr>"#
        );
        let actions = ["up", "down", "left", "right", "def", "jump", "att"];
        for a in actions {
            let k0 = ctrls[0].config.get(a).cloned().unwrap_or_default();
            let k1 = ctrls
                .get(1)
                .and_then(|c| c.config.get(a))
                .cloned()
                .unwrap_or_default();
            html.push_str(&format!(
                "<tr><td>{a}</td><td><kbd>{k0}</kbd></td><td><kbd>{k1}</kbd></td></tr>"
            ));
        }
        html.push_str(
            r#"</table>
            <p class="cs_hint">Click a key cell, then press a key to rebind. Esc/OK saves.</p>
            <div class="keybind_row">P1 att defaults: j · P2 att: numpad 1 — click <kbd data-rebind="0:att">rebind P1 att</kbd>
            <kbd data-rebind="1:att">rebind P2 att</kbd>
            <kbd data-rebind="0:jump">P1 jump</kbd>
            <kbd data-rebind="0:def">P1 def</kbd>
            <kbd data-rebind="1:jump">P2 jump</kbd>
            <kbd data-rebind="1:def">P2 def</kbd></div>
            <button type="button" class="ok_hit menu_hit" data-i="99">OK</button>
            </div></div>"#,
        );
        if let Some(el) = util::div_class("settings") {
            el.set_inner_html(&html);
        }
    }

    /// Pending key rebind: (controller_index, action)
    pub fn set_rebind_target(&mut self, slot: usize, action: &str) {
        self.rebind_target = Some((slot, action.to_string()));
        self.network.append_log(&format!(
            "Press a key for P{} {}…",
            slot + 1,
            action
        ));
    }

    pub fn apply_rebind_key(&mut self, key: &str) -> bool {
        let Some((slot, action)) = self.rebind_target.take() else {
            return false;
        };
        let mut ctrls = self.controllers.borrow_mut();
        if let Some(c) = ctrls.get_mut(slot) {
            let k = key.to_lowercase();
            c.rebind(&action, &k);
        }
        drop(ctrls);
        self.save_settings();
        if self.screen == Screen::Settings {
            self.render_settings_dom();
        }
        true
    }

    /// F4: cycle canvas width normal → wide → tall maximize (LF manager maximize/wide)
    fn cycle_view_mode(&mut self) {
        self.view_mode = (self.view_mode + 1) % 3;
        let (w, h) = match self.view_mode {
            0 => (global::WINDOW_WIDTH as u32, global::VIEWER_HEIGHT as u32),
            1 => (global::WINDOW_WIDE_WIDTH as u32, global::VIEWER_HEIGHT as u32),
            _ => (
                global::WINDOW_WIDE_WIDTH as u32,
                (global::VIEWER_HEIGHT + 80.0) as u32,
            ),
        };
        self.renderer.set_size(w, h);
        if let Some(root) = util::qs(".LFroot") {
            let _ = root.set_attribute(
                "data-view",
                match self.view_mode {
                    0 => "normal",
                    1 => "wide",
                    _ => "max",
                },
            );
            if let Some(el) = root.dyn_ref::<HtmlElement>() {
                let _ = el.style().set_property("max-width", &format!("{}px", w + 20));
            }
        }
    }

    fn render_network_dom(&self) {
        if let Some(ta) = document().query_selector(".network_log").ok().flatten() {
            if let Ok(ta) = ta.dyn_into::<web_sys::HtmlTextAreaElement>() {
                ta.set_value(&self.network.log.join("\n"));
            }
        }
    }

    fn cycle_char(&mut self, slot: usize, dir: i32) {
        let n = self.char_count() as i32;
        if n == 0 || slot >= self.slots.len() {
            return;
        }
        let mut idx = self.slots[slot].char_index + dir;
        // allow -1 random
        if idx < -1 {
            idx = n - 1;
        }
        if idx >= n {
            idx = -1;
        }
        self.slots[slot].char_index = idx;
        self.slots[slot].joined = true;
        self.render_charselect_dom();
    }

    fn resolve_char_id(&self, slot: &SelectSlot) -> i32 {
        let chars = self.package.characters();
        if chars.is_empty() {
            return 1;
        }
        if slot.char_index < 0 {
            let r = (js_sys::Math::random() * chars.len() as f64) as usize;
            return chars[r.min(chars.len() - 1)].id;
        }
        let idx = slot.char_index as usize % chars.len();
        chars[idx].id
    }

    fn start_match(&mut self) {
        let mut players = vec![];
        for (i, slot) in self.slots.iter().enumerate() {
            if !slot.joined && !slot.is_com {
                continue;
            }
            let id = self.resolve_char_id(slot);
            players.push(PlayerSetup {
                id,
                team: slot.team,
                controller_index: if slot.is_com { None } else { Some(i.min(1)) },
                is_ai: slot.is_com,
                name: slot.player_name.clone(),
            });
        }
        let chars = self.package.characters();
        for c in 0..self.num_computers {
            let idx = ((c as usize) * 3 + 2) % chars.len().max(1);
            let id = chars.get(idx).map(|ch| ch.id).unwrap_or(30);
            players.push(PlayerSetup {
                id,
                team: 3 + (c % 4),
                controller_index: None,
                is_ai: true,
                name: format!("COM{}", c + 1),
            });
        }
        // ensure 2 fighters
        while players.len() < 2 {
            let id = chars.iter().find(|c| c.id == 30).map(|c| c.id).unwrap_or(1);
            players.push(PlayerSetup {
                id,
                team: 9,
                controller_index: None,
                is_ai: true,
                name: "COM".into(),
            });
        }
        match Match::create(
            &self.package,
            players,
            self.selected_bg,
            self.controllers.clone(),
            true,
        ) {
            Ok(m) => {
                self.match_game = Some(m);
                self.show_screen(Screen::Gameplay);
            }
            Err(e) => util::error(&e),
        }
    }


    fn save_settings(&self) {
        let ctrls = self.controllers.borrow();
        let mut control = vec![];
        for c in ctrls.iter() {
            let mut config = serde_json::Map::new();
            for (k, v) in &c.config {
                config.insert(k.clone(), serde_json::Value::String(v.clone()));
            }
            control.push(serde_json::json!({"type": "keyboard", "config": config}));
        }
        let obj = serde_json::json!({
            "version": 1.00002,
            "control": control,
            "player": [{"name":"player1"},{"name":"player2"}],
            "server": {"Project F Official Lobby": "http://lobby.projectf.hk"},
            "support_sound": true
        });
        crate::core_engine::support::local_storage_set("F.LF/settings", &obj.to_string());
    }

    pub fn run_loop(self) {
        let mgr = Rc::new(RefCell::new(self));

        {
            let mgr2 = mgr.clone();
            let closure = Closure::wrap(Box::new(move |ev: MouseEvent| {
                let target = ev
                    .target()
                    .and_then(|t| t.dyn_into::<HtmlElement>().ok());
                let Some(t) = target else { return };
                let cancel = t.class_list().contains("network_game_cancel");
                // walk up for data attributes
                let mut el = Some(t);
                let mut data_i = None;
                let mut data_bg = None;
                let mut data_com = None;
                let mut data_vs = None;
                while let Some(e) = el {
                    if data_i.is_none() {
                        data_i = e.get_attribute("data-i");
                    }
                    if data_bg.is_none() {
                        data_bg = e.get_attribute("data-bg");
                    }
                    if data_com.is_none() {
                        data_com = e.get_attribute("data-com");
                    }
                    if data_vs.is_none() {
                        data_vs = e.get_attribute("data-vs");
                    }
                    el = e.parent_element().and_then(|p| p.dyn_into().ok());
                }
                let mut g = mgr2.borrow_mut();
                if let Some(i) = data_i.and_then(|s| s.parse::<i32>().ok()) {
                    match i {
                        0 => g.show_screen(Screen::CharacterSelect),
                        1 => {
                            g.network.append_log("Network lobby: use original F.LF for full multiplayer.");
                            g.network.append_log("UI shell retained for parity.");
                            g.show_screen(Screen::Network);
                        }
                        2 => g.show_screen(Screen::Settings),
                        99 => { g.save_settings(); g.show_screen(Screen::FrontPage); },
                        _ => {}
                    }
                }
                if cancel {
                    g.network.disconnect();
                    g.show_screen(Screen::FrontPage);
                }
                // connect button
                let mut el = ev.target().and_then(|e| e.dyn_into::<HtmlElement>().ok());
                let mut is_connect = false;
                let mut walk = el.clone();
                while let Some(e) = walk {
                    if e.class_list().contains("server_connect") {
                        is_connect = true;
                        break;
                    }
                    walk = e.parent_element().and_then(|p| p.dyn_into().ok());
                }
                if is_connect {
                    let addr = document()
                        .query_selector(".server_address")
                        .ok()
                        .flatten()
                        .and_then(|e| e.dyn_into::<web_sys::HtmlInputElement>().ok())
                        .map(|i| i.value())
                        .unwrap_or_default();
                    let server = if addr.is_empty() {
                        "http://lobby.projectf.hk".into()
                    } else {
                        addr
                    };
                    g.network.connect(&server, "active");
                    g.render_network_dom();
                }
                let _ = el;
                if let Some(bg) = data_bg.and_then(|s| s.parse().ok()) {
                    g.selected_bg = bg;
                    if g.screen == Screen::CharacterSelect {
                        g.render_charselect_dom();
                    }
                }
                if let Some(n) = data_com.and_then(|s| s.parse().ok()) {
                    g.computer_cursor = n;
                    g.num_computers = n;
                    g.render_computer_dom();
                }
                if let Some(v) = data_vs.and_then(|s| s.parse().ok()) {
                    g.vs_cursor = v;
                    g.on_vs_action(v);
                }
                // key rebind chips: data-rebind="0:att"
                let mut walk2 = ev.target().and_then(|e| e.dyn_into::<HtmlElement>().ok());
                while let Some(e) = walk2 {
                    if let Some(rb) = e.get_attribute("data-rebind") {
                        let parts: Vec<_> = rb.split(':').collect();
                        if parts.len() == 2 {
                            if let Ok(slot) = parts[0].parse::<usize>() {
                                g.set_rebind_target(slot, parts[1]);
                            }
                        }
                        break;
                    }
                    walk2 = e.parent_element().and_then(|p| p.dyn_into().ok());
                }
            }) as Box<dyn FnMut(_)>);
            if let Some(root) = util::qs(".LFroot") {
                let _ = root.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref());
            }
            closure.forget();
        }

        {
            let mgr2 = mgr.clone();
            let closure = Closure::wrap(Box::new(move |ev: KeyboardEvent| {
                let key = ev.key();
                let k = key.to_lowercase();
                let mut g = mgr2.borrow_mut();
                if g.rebind_target.is_some() && k != "escape" {
                    g.apply_rebind_key(&key);
                    return;
                }
                match g.screen {
                    Screen::FrontPage => {
                        if k == "enter" || k == "s" {
                            g.show_screen(Screen::CharacterSelect);
                        }
                    }
                    Screen::CharacterSelect => {
                        if k == "escape" {
                            g.show_screen(Screen::FrontPage);
                        } else if k == "a" {
                            g.cycle_char(0, -1);
                        } else if k == "d" {
                            g.cycle_char(0, 1);
                        } else if k == "h" {
                            g.cycle_char(1, -1);
                        } else if k == "k" {
                            g.cycle_char(1, 1);
                        } else if k == "s" {
                            g.slots[0].joined = true;
                            g.render_charselect_dom();
                        } else if k == "j" {
                            if g.slots.len() > 1 {
                                g.slots[1].joined = true;
                            }
                            g.render_charselect_dom();
                        } else if k == "q" || k == "i" || k == "enter" {
                            // need at least one joined
                            if g.slots.iter().any(|s| s.joined) {
                                g.show_screen(Screen::ComputerCount);
                            }
                        } else if k == "f2" {
                            g.num_computers += 1;
                        }
                    }
                    Screen::ComputerCount => {
                        if k == "escape" {
                            g.show_screen(Screen::CharacterSelect);
                        } else if k == "a" || k == "arrowleft" {
                            g.computer_cursor = (g.computer_cursor - 1).rem_euclid(8);
                            g.render_computer_dom();
                        } else if k == "d" || k == "arrowright" {
                            g.computer_cursor = (g.computer_cursor + 1) % 8;
                            g.render_computer_dom();
                        } else if k == "s" || k == "enter" || k == "j" {
                            g.num_computers = g.computer_cursor;
                            g.show_screen(Screen::VsDialog);
                        }
                    }
                    Screen::VsDialog => {
                        if k == "escape" {
                            g.show_screen(Screen::CharacterSelect);
                        } else if k == "arrowup" || k == "w" {
                            g.vs_cursor = g.vs_cursor.saturating_sub(1);
                            g.render_vs_dom();
                        } else if k == "arrowdown" || k == "x" {
                            g.vs_cursor = (g.vs_cursor + 1).min(5);
                            g.render_vs_dom();
                        } else if k == "enter" || k == "s" || k == "j" {
                            let v = g.vs_cursor;
                            g.on_vs_action(v);
                        }
                    }
                    Screen::Settings => {
                        if k == "escape" {
                            g.save_settings();
                            g.show_screen(Screen::FrontPage);
                        }
                    }
                    Screen::Network => {
                        if k == "escape" {
                            g.show_screen(Screen::FrontPage);
                        }
                    }
                    Screen::Gameplay => {
                        if key == "F4" {
                            g.cycle_view_mode();
                        } else if k == "escape" {
                            g.match_game = None;
                            g.show_screen(Screen::FrontPage);
                        } else if key == "F1" || k == "p" {
                            if let Some(m) = g.match_game.as_mut() {
                                m.toggle_pause();
                            }
                        } else if key == "F2" {
                            // F.LF F2: pause then single-step one TU
                            if let Some(m) = g.match_game.as_mut() {
                                if m.paused {
                                    m.paused = false;
                                    m.tu();
                                    m.paused = true;
                                } else {
                                    m.paused = true;
                                }
                            }
                        } else if key == "F5" {
                            // LF2 F5 reset positions-ish: restart match
                            g.start_match();
                        } else if key == "F6" {
                            // infinite mp cheat lite
                            if let Some(m) = g.match_game.as_mut() {
                                for ch in &mut m.characters {
                                    ch.base.mp = ch.base.mp_full;
                                }
                            }
                        } else if key == "F7" {
                            if let Some(m) = g.match_game.as_mut() {
                                for ch in &mut m.characters {
                                    ch.base.hp = ch.base.hp_full;
                                    ch.base.dead = false;
                                    ch.base.removed = false;
                                }
                            }
                        }
                    }
                }
            }) as Box<dyn FnMut(_)>);
            let _ = util::window()
                .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref());
            closure.forget();
        }

        fn arm(mgr: Rc<RefCell<Manager>>) {
            let mgr2 = mgr.clone();
            let cb = Closure::once_into_js(move |ts: f64| {
                {
                    let mut g = mgr2.borrow_mut();
                    g.frame(ts);
                }
                arm(mgr2);
            });
            let _ = util::window().request_animation_frame(cb.as_ref().unchecked_ref());
        }
        mgr.borrow_mut().render_frontpage_dom();
        arm(mgr);
    }

    fn on_vs_action(&mut self, v: usize) {
        match v {
            0 | 5 => self.start_match(),
            1 => {
                for s in &mut self.slots {
                    s.joined = false;
                    s.char_index = 0;
                }
                self.show_screen(Screen::CharacterSelect);
            }
            4 => self.show_screen(Screen::CharacterSelect),
            _ => self.start_match(),
        }
    }

    fn frame(&mut self, ts: f64) {
        let dt = (ts - self.last_tick).max(0.0).min(100.0);
        self.last_tick = ts;
        self.acc += dt;
        let step = 1000.0 / global::FRAMERATE;
        while self.acc >= step {
            self.acc -= step;
            if self.screen == Screen::Gameplay {
                if let Some(m) = self.match_game.as_mut() {
                    m.tu();
                }
            }
            self.frames_counted += 1;
        }
        if ts - self.fps_timer > 500.0 {
            self.fps_display = self.frames_counted as f64 / ((ts - self.fps_timer) / 1000.0);
            self.frames_counted = 0;
            self.fps_timer = ts;
            if let Some(inp) = document().query_selector(".fps").ok().flatten() {
                if let Ok(inp) = inp.dyn_into::<web_sys::HtmlInputElement>() {
                    inp.set_value(&format!("{:.0}fps", self.fps_display));
                }
            }
        }
        if self.screen == Screen::Gameplay {
            if let Some(m) = self.match_game.as_mut() {
                self.renderer.clear("#000");
                m.render(&mut self.renderer);
            }
        }
    }
}
