//! Game manager — menus, character select, match lifecycle (LF/manager.js)
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
    /// character select state
    slots: Vec<SelectSlot>,
    selected_bg: i32,
    num_computers: i32,
    menu_index: usize,
    last_tick: f64,
    acc: f64,
    fps_display: f64,
    frames_counted: u32,
    fps_timer: f64,
    running: Rc<Cell<bool>>,
}

#[derive(Clone)]
struct SelectSlot {
    player_name: String,
    char_index: usize,
    team: i32,
    joined: bool,
    is_com: bool,
}

impl Manager {
    pub fn new(package: Package, version: &str) -> Result<Self, wasm_bindgen::JsValue> {
        let asset_root = package.root.clone();
        let mut renderer = CanvasRenderer::from_selector(".canvas", &asset_root)
            .map_err(|e| wasm_bindgen::JsValue::from_str(&e))?;
        renderer.set_size(global::WINDOW_WIDTH as u32, global::VIEWER_HEIGHT as u32);

        let mut controllers = vec![
            Controller::new_keyboard(controller::default_p1_config()),
            Controller::new_keyboard(controller::default_p2_config()),
        ];
        // load settings
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

        let chars = package.characters();
        let slots = vec![
            SelectSlot {
                player_name: "player1".into(),
                char_index: 0,
                team: 1,
                joined: true,
                is_com: false,
            },
            SelectSlot {
                player_name: "player2".into(),
                char_index: chars.len().saturating_sub(1).min(1),
                team: 2,
                joined: true,
                is_com: false,
            },
        ];

        let first_bg = package
            .data_list
            .background
            .first()
            .map(|b| b.id)
            .unwrap_or(0);

        // title
        if let Some(el) = util::div_class("window_caption_title") {
            el.set_inner_text(version);
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
            menu_index: 0,
            last_tick: js_sys::Date::now(),
            acc: 0.0,
            fps_display: 0.0,
            frames_counted: 0,
            fps_timer: js_sys::Date::now(),
            running: Rc::new(Cell::new(true)),
        };
        mgr.show_screen(Screen::FrontPage);
        mgr.bind_ui();
        Ok(mgr)
    }

    fn bind_ui(&mut self) {
        // maximize
        if let Some(btn) = util::div_class("maximize_button") {
            let container = util::qs(".LFroot .container");
            let closure = Closure::wrap(Box::new(move |_ev: MouseEvent| {
                if let Some(c) = util::qs(".LFroot .container") {
                    let cl = c.class_list();
                    if cl.contains("maximized") {
                        let _ = cl.remove_1("maximized");
                    } else {
                        let _ = cl.add_1("maximized");
                    }
                }
            }) as Box<dyn FnMut(_)>);
            let _ = btn
                .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref());
            closure.forget();
            let _ = container;
        }

        // keyboard for menus
        let win = util::window();
        // We handle menu keys in the game loop via controllers / key events stored globally
        let _ = win;
    }

    fn show_screen(&mut self, s: Screen) {
        self.screen = s;
        let show_cls = |cls: &str, on: bool| {
            if let Some(el) = util::div_class(cls) {
                if on {
                    show(&el);
                } else {
                    hide(&el);
                }
            }
        };
        show_cls("frontpage", s == Screen::FrontPage);
        show_cls("character_selection", s == Screen::CharacterSelect);
        show_cls("settings", s == Screen::Settings);
        show_cls("network_game", s == Screen::Network);
        show_cls("gameplay", s == Screen::Gameplay);

        // build dynamic UI
        match s {
            Screen::FrontPage => self.render_frontpage_dom(),
            Screen::CharacterSelect => self.render_charselect_dom(),
            Screen::Settings => self.render_settings_dom(),
            Screen::Network => self.render_network_dom(),
            Screen::Gameplay => {}
        }
    }

    fn render_frontpage_dom(&self) {
        if let Some(el) = util::div_class("frontpage_content") {
            el.set_inner_html(
                r#"<div class="menu_box">
                <div class="menu_item" data-i="0">game start (開始遊戲)</div>
                <div class="menu_item" data-i="1">network game (連線遊戲)</div>
                <div class="menu_item" data-i="2">control settings (控制設定)</div>
                </div>"#,
            );
        }
    }

    fn render_charselect_dom(&self) {
        let chars = self.package.characters();
        let mut html = String::from("<div class='char_select_inner'><h3>Character Selection</h3><div class='slots'>");
        for (i, slot) in self.slots.iter().enumerate() {
            let name = chars
                .get(slot.char_index)
                .map(|c| c.name.as_str())
                .unwrap_or("?");
            let pic = chars
                .get(slot.char_index)
                .map(|c| c.pic.as_str())
                .unwrap_or("");
            html.push_str(&format!(
                "<div class='slot' data-slot='{}'><div>{} {}</div><img src='{}/{}' width='50' height='50'/><div>{} team{}</div><div class='hint'>A/D or H/K change · S/J attack join · Q/I start</div></div>",
                i,
                slot.player_name,
                if slot.is_com { "(COM)" } else { "" },
                self.package.root,
                pic,
                name,
                slot.team
            ));
        }
        html.push_str("</div>");
        // backgrounds
        html.push_str("<div class='bg_list'><b>Background</b><br/>");
        for bg in &self.package.data_list.background {
            let sel = if bg.id == self.selected_bg { " *" } else { "" };
            html.push_str(&format!("<span class='bg_item' data-bg='{}'>{}{}</span> ", bg.id, bg.name, sel));
        }
        html.push_str("</div>");
        html.push_str("<div class='cs_help'>Enter/Attack: start match · Esc: back · F2: add COM · Left/Right: character</div></div>");
        if let Some(el) = util::div_class("character_selection") {
            el.set_inner_html(&html);
        }
    }

    fn render_settings_dom(&self) {
        let ctrls = self.controllers.borrow();
        let mut html = String::from("<div class='settings_inner'><h3>Control Settings</h3>");
        for (i, c) in ctrls.iter().enumerate() {
            html.push_str(&format!("<div class='pctrl'><b>Player {}</b><ul>", i + 1));
            for (act, key) in &c.config {
                html.push_str(&format!("<li>{} → <kbd>{}</kbd></li>", act, key));
            }
            html.push_str("</ul></div>");
        }
        html.push_str("<p>Defaults: P1 WASD+Q/Z/S · P2 UHJK+I/,/J — Esc back</p></div>");
        if let Some(el) = util::div_class("settings") {
            el.set_inner_html(&html);
        }
    }

    fn render_network_dom(&self) {
        // static HTML already in page; append log
        if let Some(ta) = document().query_selector(".network_log").ok().flatten() {
            if let Ok(ta) = ta.dyn_into::<web_sys::HtmlTextAreaElement>() {
                ta.set_value(&self.network.log.join("\n"));
            }
        }
    }

    fn start_match(&mut self) {
        let chars = self.package.characters();
        let mut players = vec![];
        for (i, slot) in self.slots.iter().enumerate() {
            if !slot.joined {
                continue;
            }
            let id = chars.get(slot.char_index).map(|c| c.id).unwrap_or(1);
            players.push(PlayerSetup {
                id,
                team: slot.team,
                controller_index: if slot.is_com { None } else { Some(i.min(1)) },
                is_ai: slot.is_com,
            });
        }
        // add computers
        for c in 0..self.num_computers {
            let idx = (c as usize + 2) % chars.len().max(1);
            let id = chars.get(idx).map(|ch| ch.id).unwrap_or(30);
            players.push(PlayerSetup {
                id,
                team: 3 + c,
                controller_index: None,
                is_ai: true,
            });
        }
        if players.len() < 2 {
            // ensure at least bandit COM
            if let Some(bandit) = chars.iter().find(|c| c.id == 30) {
                players.push(PlayerSetup {
                    id: bandit.id,
                    team: 9,
                    controller_index: None,
                    is_ai: true,
                });
            }
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

    pub fn run_loop(self) {
        let mgr = Rc::new(RefCell::new(self));

        // click handlers on frontpage via event delegation
        {
            let mgr2 = mgr.clone();
            let closure = Closure::wrap(Box::new(move |ev: MouseEvent| {
                let target = ev.target().and_then(|t| t.dyn_into::<HtmlElement>().ok());
                let Some(t) = target else { return };
                let cls = t.class_name();
                let mut g = mgr2.borrow_mut();
                if cls.contains("menu_item") || t.get_attribute("data-i").is_some() {
                    let i = t
                        .get_attribute("data-i")
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0);
                    match i {
                        0 => g.show_screen(Screen::CharacterSelect),
                        1 => {
                            g.network.append_log("Opening network menu...");
                            g.show_screen(Screen::Network);
                        }
                        2 => g.show_screen(Screen::Settings),
                        _ => {}
                    }
                }
                if cls.contains("network_game_cancel") || t.class_list().contains("network_game_cancel") {
                    g.show_screen(Screen::FrontPage);
                }
                if let Some(bg) = t.get_attribute("data-bg") {
                    if let Ok(id) = bg.parse() {
                        g.selected_bg = id;
                        g.render_charselect_dom();
                    }
                }
            }) as Box<dyn FnMut(_)>);
            if let Some(root) = util::qs(".LFroot") {
                let _ = root.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref());
            }
            closure.forget();
        }

        // keydown for UI
        {
            let mgr2 = mgr.clone();
            let closure = Closure::wrap(Box::new(move |ev: KeyboardEvent| {
                let key = ev.key();
                let mut g = mgr2.borrow_mut();
                match g.screen {
                    Screen::FrontPage => {
                        if key == "Enter" || key == "s" || key == "S" {
                            g.show_screen(Screen::CharacterSelect);
                        }
                    }
                    Screen::CharacterSelect => {
                        let k = key.to_lowercase();
                        if k == "escape" {
                            g.show_screen(Screen::FrontPage);
                        } else if k == "enter" {
                            g.start_match();
                        } else if k == "f2" {
                            g.num_computers += 1;
                            g.render_charselect_dom();
                        } else if k == "a" || k == "h" {
                            let slot = 0;
                            if slot < g.slots.len() {
                                let n = g.package.characters().len().max(1);
                                g.slots[slot].char_index = (g.slots[slot].char_index + n - 1) % n;
                                g.render_charselect_dom();
                            }
                        } else if k == "d" || k == "k" {
                            let slot = if k == "k" { 1 } else { 0 };
                            if slot < g.slots.len() {
                                let n = g.package.characters().len().max(1);
                                g.slots[slot].char_index = (g.slots[slot].char_index + 1) % n;
                                g.render_charselect_dom();
                            }
                        } else if k == "s" {
                            // p1 att — cycle already handled; also start if held? LF2 uses att to join
                            g.slots[0].joined = true;
                        } else if k == "j" {
                            if g.slots.len() > 1 {
                                g.slots[1].joined = true;
                            }
                        } else if k == "q" || k == "i" {
                            g.start_match();
                        }
                    }
                    Screen::Settings | Screen::Network => {
                        if key == "Escape" {
                            g.show_screen(Screen::FrontPage);
                        }
                    }
                    Screen::Gameplay => {
                        if key == "Escape" || key == "F4" {
                            g.match_game = None;
                            g.show_screen(Screen::FrontPage);
                        } else if key == "F1" || key.to_lowercase() == "p" {
                            if let Some(m) = g.match_game.as_mut() {
                                m.toggle_pause();
                            }
                        }
                    }
                }
            }) as Box<dyn FnMut(_)>);
            let _ = util::window()
                .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref());
            closure.forget();
        }

        // rAF loop
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
        // prime frontpage
        mgr.borrow_mut().render_frontpage_dom();
        arm(mgr);
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

        // fps
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
                // clear sky
                self.renderer.clear("#87ceeb");
                m.render(&mut self.renderer);
            }
        }
    }
}
