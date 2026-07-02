//! Game manager — closer to LF/manager.js UX
use crate::core_engine::controller::{self, Controller};
use crate::core_engine::sprite::{CanvasRenderer, DomSpriteLayer, RendererKind};
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
use wasm_bindgen::JsValue;
use web_sys::{HtmlElement, KeyboardEvent, MouseEvent};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Screen {
    FrontPage,
    CharacterSelect,
    ComputerCount,
    VsDialog,
    StageDialog,
    Settings,
    Network,
    Gameplay,
}

pub struct Manager {
    package: Package,
    version: String,
    screen: Screen,
    renderer: CanvasRenderer,
    dom_layer: Option<DomSpriteLayer>,
    renderer_kind: RendererKind,
    controllers: Rc<RefCell<Vec<Controller>>>,
    match_game: Option<Match>,
    sound: Soundpack,
    network: NetworkSession,
    slots: Vec<SelectSlot>,
    selected_bg: i32,
    num_computers: i32,
    computer_cursor: i32,
    vs_cursor: usize,
    /// -1 crazy, 0 hard, 1 normal, 2 easy (F.LF AI difficulty)
    difficulty: i8,
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
    /// Street campaign entry
    stage_mode: bool,
    stage_chapter: usize,
    stage_cursor: usize,
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

        let params = crate::core_engine::util::location_parameters();
        let renderer_kind = if params.get("renderer").map(|s| s.as_str()) == Some("dom") {
            RendererKind::Dom
        } else {
            RendererKind::Canvas
        };
        let dom_layer = DomSpriteLayer::attach(".gameplay", &asset_root).ok();
        if let Some(ref d) = dom_layer {
            d.set_visible(renderer_kind == RendererKind::Dom);
        }
        if renderer_kind == RendererKind::Dom {
            // keep canvas as bg fallback under DOM sprites
            if let Some(c) = util::qs(".canvas") {
                let _ = c.set_attribute("style", "opacity:1");
            }
        }

        let mut mgr = Self {
            package,
            version: version.to_string(),
            screen: Screen::FrontPage,
            renderer,
            dom_layer,
            renderer_kind,
            controllers,
            match_game: None,
            sound: Soundpack::new(),
            network: NetworkSession::new(),
            slots,
            selected_bg: first_bg,
            num_computers: 0,
            computer_cursor: 0,
            vs_cursor: 0,
            difficulty: 0,
            last_tick: js_sys::Date::now(),
            acc: 0.0,
            fps_display: 0.0,
            frames_counted: 0,
            fps_timer: js_sys::Date::now(),
            running: Rc::new(Cell::new(true)),
            rebind_target: None,
            view_mode: 0,
            stage_mode: false,
            stage_chapter: 0,
            stage_cursor: 0,
        };
        // apply UI bg colors
        if let Some(fp) = util::div_class("frontpage") {
            let col = mgr.package.ui["frontpage"]["bg_color"].as_str().unwrap_or("#10206c");
            let _ = fp.style().set_property("background-color", col);
        }
        crate::core_engine::touch::mount_touch_hint();
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
            matches!(
                s,
                Screen::CharacterSelect
                    | Screen::ComputerCount
                    | Screen::VsDialog
                    | Screen::StageDialog
            ),
        );
        show_cls("settings", s == Screen::Settings);
        show_cls("network_game", s == Screen::Network);
        show_cls("gameplay", s == Screen::Gameplay);

        match s {
            Screen::FrontPage => self.render_frontpage_dom(),
            Screen::CharacterSelect => self.render_charselect_dom(),
            Screen::ComputerCount => self.render_computer_dom(),
            Screen::VsDialog => self.render_vs_dom(),
            Screen::StageDialog => self.render_stage_dom(),
            Screen::Settings => self.render_settings_dom(),
            Screen::Network => self.render_network_dom(),
            Screen::Gameplay => {}
        }
    }

    fn render_stage_dom(&self) {
        let root = &self.package.root;
        let stage_file = self.package.stage.as_ref();
        let chapters: Vec<_> = stage_file
            .map(|s| s.stages.iter().filter(|st| st.id < 50).collect())
            .unwrap_or_default();
        let survival = stage_file.and_then(|s| s.stages.iter().find(|st| st.id >= 50));
        let diff = match self.difficulty {
            d if d < 0 => "CRAZY!",
            0 => "Difficult",
            1 => "Normal",
            _ => "Easy",
        };
        let mut html = format!(
            r#"<div class="cs_stage dim"><img class="cs_bg" src="{root}/UI/character_selection.png" alt=""/>
            <div class="stage_panel" style="position:absolute;left:190px;top:100px;width:420px;padding:16px;background:rgba(0,20,60,0.94);border:2px solid #5af;color:#def;font:14px sans-serif;z-index:5;">
            <h2 style="margin:0 0 8px;color:#ff0;">Street Campaign</h2>
            <p style="margin:4px 0;">Difficulty: <b id="stage_diff_label">{diff}</b></p><div>"#
        );
        for (i, st) in chapters.iter().enumerate() {
            let sel = i == self.stage_chapter;
            html.push_str(&format!(
                r#"<button type="button" class="menu_hit stage_pick" data-stage="{i}" style="display:block;width:100%;text-align:left;margin:3px 0;padding:6px 8px;background:{bg};color:{fg};border:1px solid #456;cursor:pointer;">{name} — {n} phases</button>"#,
                bg = if sel { "#1a3a8a" } else { "transparent" },
                fg = if sel { "#fff" } else { "#8af" },
                name = st.name,
                n = st.phases.len(),
            ));
        }
        if let Some(sv) = survival {
            let i = chapters.len();
            let sel = self.stage_chapter == i;
            html.push_str(&format!(
                r#"<button type="button" class="menu_hit stage_pick" data-stage="{i}" style="display:block;width:100%;text-align:left;margin:3px 0;padding:6px 8px;background:{bg};color:{fg};border:1px solid #456;cursor:pointer;">{name}</button>"#,
                bg = if sel { "#1a3a8a" } else { "transparent" },
                fg = if sel { "#fff" } else { "#8af" },
                name = sv.name,
            ));
        }
        html.push_str(
            r#"</div><p style="margin:10px 0 4px;font-size:12px;color:#9ab;">Select chapter, then your fighter. Defeat each wave and walk right on <b>GO!</b></p>
            <button type="button" class="menu_hit" data-i="30" style="margin-top:8px;padding:8px 16px;background:#2a6;color:#fff;border:0;cursor:pointer;">Choose fighter</button>
            <button type="button" class="menu_hit" data-i="32" style="margin-top:8px;margin-left:8px;padding:8px 16px;background:#258;color:#fff;border:0;cursor:pointer;">Difficulty</button>
            <button type="button" class="menu_hit" data-i="31" style="margin-top:8px;margin-left:8px;padding:8px 16px;background:#444;color:#fff;border:0;cursor:pointer;">Back</button>
            </div></div>"#,
        );
        if let Some(el) = util::div_class("character_selection") {
            el.set_inner_html(&html);
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
              <button type="button" class="menu_hit stage_entry" data-i="3" style="position:absolute;left:12px;bottom:12px;padding:6px 10px;background:rgba(0,40,100,0.85);color:#9cf;border:1px solid #48f;cursor:pointer;font:12px sans-serif;" title="stage mode">Street Campaign</button>
              <button type="button" class="menu_hit demo_entry" data-i="4" style="position:absolute;right:12px;bottom:12px;padding:6px 10px;background:rgba(0,40,100,0.85);color:#9cf;border:1px solid #48f;cursor:pointer;font:12px sans-serif;" title="demo mode">Demo</button>
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
            let joined_cls = if slot.joined { " joined" } else { "" };
            html.push_str(&format!(
                r#"<div class="cs_box{joined_cls}" style="left:{px}px;top:{py}px;width:120px;height:116px;">
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
        let diff_label = match self.difficulty {
            d if d < 0 => "CRAZY!",
            0 => "difficult",
            1 => "normal",
            _ => "easy",
        };
        let labels = [
            "Vs mode start".to_string(),
            "Reset".into(),
            "Background → click list".into(),
            format!("Difficulty: {diff_label} (click)"),
            "Back".into(),
            "OK".into(),
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
        // F.LF keychanger: rows = name, type, then each action; columns = players
        let actions = ["up", "down", "left", "right", "def", "jump", "att"];
        let mut html = format!(
            r#"<div class="set_stage"><img src="{root}/UI/settings.png" class="set_bg" alt=""/>
            <div class="keychanger"><table class="keychanger_table">"#
        );
        // name row
        html.push_str("<tr><td class='kc_left'>name</td>");
        for (i, s) in self.slots.iter().enumerate() {
            html.push_str(&format!(
                r#"<td class="kc_cell kc_name" data-pname="{i}">{pn}</td>"#,
                pn = s.player_name
            ));
        }
        html.push_str("</tr><tr><td class='kc_left'>type</td>");
        for i in 0..ctrls.len().min(2) {
            html.push_str(&format!(
                r#"<td class="kc_cell">keyboard</td>"#
            ));
            let _ = i;
        }
        html.push_str("</tr>");
        for a in actions {
            html.push_str(&format!("<tr><td class='kc_left'>{a}</td>"));
            for pi in 0..2 {
                let k = ctrls
                    .get(pi)
                    .and_then(|c| c.config.get(a))
                    .cloned()
                    .unwrap_or_else(|| "-".into());
                html.push_str(&format!(
                    r#"<td class="kc_cell kc_key" data-rebind="{pi}:{a}">{k}</td>"#
                ));
            }
            html.push_str("</tr>");
        }
        html.push_str(
            r#"</table>
            <p class="cs_hint">Click a key cell, then press a key to rebind (F.LF keychanger). Click name to rename. Esc/OK saves.</p>
            <button type="button" class="ok_hit menu_hit" data-i="99">OK</button>
            </div></div>"#,
        );
        if let Some(el) = util::div_class("settings") {
            el.set_inner_html(&html);
        }
    }

    /// F.LF manager.alert
    pub fn alert(&self, mess: &str) {
        if let Some(box_) = util::qs(".alert_box") {
            let _ = box_.set_attribute("style", "display:block");
        }
        if let Some(msg) = util::qs(".alert_message") {
            if let Ok(el) = msg.dyn_into::<HtmlElement>() {
                el.set_inner_text(mess);
            }
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
    pub fn cycle_view_mode(&mut self) {
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
        // F.LF maximize_button toggles .maximized on container + extra_UI
        let maxed = self.view_mode >= 1;
        if let Some(c) = util::qs(".container") {
            let cl = c.class_list();
            let _ = if maxed {
                cl.add_1("maximized")
            } else {
                cl.remove_1("maximized")
            };
        }
        if let Some(c) = util::qs(".extra_UI") {
            let cl = c.class_list();
            let _ = if maxed {
                cl.add_1("maximized")
            } else {
                cl.remove_1("maximized")
            };
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

    /// F.LF start_demo — COM vs COM (Woody vs Firen-ish picks)
    fn start_demo(&mut self, _playable: bool) {
        let chars = self.package.characters();
        let id_a = chars
            .iter()
            .find(|c| c.id == 10 || c.name.eq_ignore_ascii_case("woody"))
            .map(|c| c.id)
            .or_else(|| chars.first().map(|c| c.id))
            .unwrap_or(1);
        let id_b = chars
            .iter()
            .find(|c| c.id == 8 || c.name.eq_ignore_ascii_case("firen"))
            .map(|c| c.id)
            .or_else(|| chars.get(1).map(|c| c.id))
            .unwrap_or(id_a);
        let players = vec![
            PlayerSetup {
                id: id_a,
                team: 1,
                controller_index: None,
                is_ai: true,
                name: "CRUSHER".into(),
            },
            PlayerSetup {
                id: id_b,
                team: 2,
                controller_index: None,
                is_ai: true,
                name: "dumbass".into(),
            },
        ];
        let bg = if self.package.backgrounds.contains_key(&6) {
            6
        } else {
            self.selected_bg
        };
        match Match::create(
            &self.package,
            players,
            bg,
            self.controllers.clone(),
            false,
        ) {
            Ok(mut m) => {
                m.demo_mode = true;
                for b in &mut m.ai_brains {
                    b.difficulty = 2;
                }
                self.match_game = Some(m);
                self.show_screen(Screen::Gameplay);
                self.alert("Demo mode — Esc returns to menu when available.");
            }
            Err(e) => {
                self.alert(&e);
                util::error(&e);
            }
        }
    }

    fn start_stage_match(&mut self) {
        let Some(stage_file) = self.package.stage.clone() else {
            self.alert("Stage data missing.");
            return;
        };
        let chapter = self.stage_chapter.min(stage_file.stages.len().saturating_sub(1));
        let bg = stage_file
            .chapter(chapter)
            .map(|c| c.background)
            .unwrap_or(self.selected_bg);
        let mut players = vec![];
        for (i, slot) in self.slots.iter().enumerate() {
            if !slot.joined {
                continue;
            }
            let id = self.resolve_char_id(slot);
            players.push(PlayerSetup {
                id,
                team: 1,
                controller_index: Some(i.min(1)),
                is_ai: false,
                name: slot.player_name.clone(),
            });
        }
        if players.is_empty() {
            let id = self
                .package
                .characters()
                .first()
                .map(|c| c.id)
                .unwrap_or(11);
            players.push(PlayerSetup {
                id,
                team: 1,
                controller_index: Some(0),
                is_ai: false,
                name: "Player1".into(),
            });
        }
        let humans = players.len();
        match Match::create(
            &self.package,
            players,
            bg,
            self.controllers.clone(),
            false,
        ) {
            Ok(mut m) => {
                m.attach_stage(stage_file, chapter, humans, self.difficulty, &self.package);
                for b in &mut m.ai_brains {
                    b.difficulty = self.difficulty;
                }
                self.match_game = Some(m);
                self.show_screen(Screen::Gameplay);
            }
            Err(e) => {
                self.alert(&e);
                util::error(&e);
            }
        }
    }

    fn start_match(&mut self) {
        if self.stage_mode {
            self.start_stage_match();
            return;
        }
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
            Ok(mut m) => {
                for b in &mut m.ai_brains {
                    b.difficulty = self.difficulty;
                }
                self.match_game = Some(m);
                self.show_screen(Screen::Gameplay);
            }
            Err(e) => {
                self.alert(&e);
                util::error(&e);
            }
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
                let mut data_stage = None::<String>;
                let mut walk_st = ev.target().and_then(|e| e.dyn_into::<HtmlElement>().ok());
                while let Some(e) = walk_st {
                    if data_stage.is_none() {
                        data_stage = e.get_attribute("data-stage");
                    }
                    walk_st = e.parent_element().and_then(|p| p.dyn_into().ok());
                }
                let mut g = mgr2.borrow_mut();
                if let Some(si) = data_stage.and_then(|s| s.parse::<usize>().ok()) {
                    g.stage_chapter = si;
                    g.stage_mode = true;
                    g.render_stage_dom();
                }
                // player name edit (settings keychanger)
                let mut walk_pn = ev.target().and_then(|e| e.dyn_into::<HtmlElement>().ok());
                while let Some(e) = walk_pn.clone() {
                    if let Some(pi) = e.get_attribute("data-pname").and_then(|s| s.parse::<usize>().ok()) {
                        if let Some(win) = web_sys::window() {
                            let cur = g.slots.get(pi).map(|s| s.player_name.clone()).unwrap_or_default();
                            if let Ok(Some(v)) = win.prompt_with_message_and_default("Player name:", &cur) {
                                if let Some(s) = g.slots.get_mut(pi) {
                                    s.player_name = v;
                                }
                                g.render_settings_dom();
                            }
                        }
                        break;
                    }
                    walk_pn = e.parent_element().and_then(|p| p.dyn_into().ok());
                }
                // alert OK
                let mut walk_al = ev.target().and_then(|e| e.dyn_into::<HtmlElement>().ok());
                while let Some(e) = walk_al {
                    if e.class_list().contains("alert_box_ok") {
                        if let Some(box_) = util::qs(".alert_box") {
                            let _ = box_.set_attribute("style", "display:none");
                        }
                        break;
                    }
                    walk_al = e.parent_element().and_then(|p| p.dyn_into().ok());
                }
                if let Some(i) = data_i.and_then(|s| s.parse::<i32>().ok()) {
                    match i {
                        0 => {
                            g.stage_mode = false;
                            g.show_screen(Screen::CharacterSelect);
                        }
                        1 => {
                            g.network.append_log("Network: F.Lobby 0.1 protocol + Peer/BC lockstep.");
                            g.show_screen(Screen::Network);
                        }
                        2 => g.show_screen(Screen::Settings),
                        3 => {
                            g.stage_mode = true;
                            g.stage_chapter = 0;
                            g.num_computers = 0;
                            if g.package.stage.is_none() {
                                g.alert("No stage data in package (data/stage.json).");
                            } else {
                                g.show_screen(Screen::StageDialog);
                            }
                        }
                        30 => {
                            g.stage_mode = true;
                            g.show_screen(Screen::CharacterSelect);
                        }
                        31 => {
                            g.stage_mode = false;
                            g.show_screen(Screen::FrontPage);
                        }
                        32 => {
                            g.difficulty = match g.difficulty {
                                d if d < 0 => 0,
                                0 => 1,
                                1 => 2,
                                _ => -1,
                            };
                            g.render_stage_dom();
                        }
                        4 => {
                            // F.LF start_demo — two AI fighters, no gameover panel pressure
                            g.start_demo(false);
                        }
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
                    let mut addr = document()
                        .query_selector(".server_address")
                        .ok()
                        .flatten()
                        .and_then(|e| e.dyn_into::<web_sys::HtmlInputElement>().ok())
                        .map(|i| i.value())
                        .unwrap_or_default();
                    if addr.is_empty() {
                        if let Some(sel) = document()
                            .query_selector(".server_select")
                            .ok()
                            .flatten()
                            .and_then(|e| e.dyn_into::<web_sys::HtmlSelectElement>().ok())
                        {
                            let v = sel.value();
                            if v != "third_party_server" {
                                addr = v;
                            }
                        }
                    }
                    let server = if addr.is_empty() {
                        "http://lobby.projectf.hk".into()
                    } else {
                        addr
                    };
                    g.network.set_room_hint(&server);
                    let role = if server.contains(":passive") || server.ends_with("/p") {
                        "passive"
                    } else {
                        "active"
                    };
                    let server = server.replace(":passive", "").replace("/p", "");
                    g.network.set_room_hint(&server);
                    // F.Lobby client (JS) + lockstep transport
                    g.network.connect(&server, role);
                    g.render_network_dom();
                }
                // server_select third party prompt
                let mut walk_sel = ev.target().and_then(|e| e.dyn_into::<HtmlElement>().ok());
                while let Some(e) = walk_sel {
                    if e.class_list().contains("server_select") {
                        if let Ok(sel) = e.dyn_into::<web_sys::HtmlSelectElement>() {
                            if sel.value() == "third_party_server" {
                                if let Some(win) = web_sys::window() {
                                    if let Ok(Some(v)) = win.prompt_with_message_and_default(
                                        "Enter F.Lobby server address:",
                                        "http://localhost:8001",
                                    ) {
                                        if let Some(inp) = document()
                                            .query_selector(".server_address")
                                            .ok()
                                            .flatten()
                                            .and_then(|e| e.dyn_into::<web_sys::HtmlInputElement>().ok())
                                        {
                                            inp.set_value(&v);
                                        }
                                    }
                                }
                            } else if !sel.value().is_empty() {
                                if let Some(inp) = document()
                                    .query_selector(".server_address")
                                    .ok()
                                    .flatten()
                                    .and_then(|e| e.dyn_into::<web_sys::HtmlInputElement>().ok())
                                {
                                    inp.set_value(&sel.value());
                                }
                            }
                        }
                        break;
                    }
                    walk_sel = e.parent_element().and_then(|p| p.dyn_into().ok());
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
                            if g.stage_mode {
                                g.show_screen(Screen::StageDialog);
                            } else {
                                g.show_screen(Screen::FrontPage);
                            }
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
                            g.slots[0].team = if g.stage_mode { 1 } else { g.slots[0].team };
                            g.render_charselect_dom();
                        } else if k == "j" {
                            if g.slots.len() > 1 {
                                g.slots[1].joined = true;
                                if g.stage_mode {
                                    g.slots[1].team = 1;
                                }
                            }
                            g.render_charselect_dom();
                        } else if k == "q" || k == "i" || k == "enter" {
                            if g.slots.iter().any(|s| s.joined) {
                                if g.stage_mode {
                                    g.start_stage_match();
                                } else {
                                    g.show_screen(Screen::ComputerCount);
                                }
                            }
                        } else if k == "f2" {
                            g.num_computers += 1;
                        }
                    }
                    Screen::StageDialog => {
                        if k == "escape" {
                            g.stage_mode = false;
                            g.show_screen(Screen::FrontPage);
                        } else if k == "arrowup" || k == "w" {
                            g.stage_chapter = g.stage_chapter.saturating_sub(1);
                            g.render_stage_dom();
                        } else if k == "arrowdown" || k == "x" {
                            let max = g
                                .package
                                .stage
                                .as_ref()
                                .map(|s| s.stages.len().saturating_sub(1))
                                .unwrap_or(0);
                            g.stage_chapter = (g.stage_chapter + 1).min(max);
                            g.render_stage_dom();
                        } else if k == "enter" || k == "s" || k == "j" {
                            g.show_screen(Screen::CharacterSelect);
                        } else if k == "d" {
                            g.difficulty = match g.difficulty {
                                d if d < 0 => 0,
                                0 => 1,
                                1 => 2,
                                _ => -1,
                            };
                            g.render_stage_dom();
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
                            // F.LF match F4 ends match; maximize via button / F10
                            if let Some(m) = g.match_game.as_mut() {
                                m.f4_end_match();
                            }
                        } else if k == "escape" {
                            g.match_game = None;
                            g.show_screen(Screen::FrontPage);
                        } else if key == "F1" || k == "p" {
                            if let Some(m) = g.match_game.as_mut() {
                                m.toggle_pause();
                                if let Some(el) = util::qs(".pause_message") {
                                    el.set_inner_html(if m.paused { "PAUSED" } else { "" });
                                }
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
                                if let Some(el) = util::qs(".pause_message") {
                                    el.set_inner_html(if m.paused { "PAUSED (F2 step)" } else { "" });
                                }
                            }
                        } else if key == "F3" {
                            if let Some(m) = g.match_game.as_mut() {
                                let mut s = String::from("STATS ");
                                for ch in &m.characters {
                                    s.push_str(&format!("|{} K:{} HP:{:.0} ", ch.base.name, ch.base.kills, ch.base.hp));
                                }
                                m.overlay_message(&s);
                            }
                        } else if key == "F5" {
                            // LF2 F5 reset positions-ish: restart match
                            g.start_match();
                        } else if key == "F6" {
                            if let Some(m) = g.match_game.as_mut() {
                                m.f6_mode = !m.f6_mode;
                                for ch in &mut m.characters {
                                    ch.infinite_mp = m.f6_mode;
                                    if m.f6_mode {
                                        ch.base.mp = ch.base.mp_full;
                                    }
                                }
                                m.overlay_message(if m.f6_mode { "F6 infinite MP ON" } else { "F6 infinite MP OFF" });
                            }
                        } else if key == "F7" {
                            if let Some(m) = g.match_game.as_mut() {
                                m.F7_refill();
                                m.overlay_message("F7 HP/MP refill");
                            }
                        } else if key == "F8" {
                            // F.LF F8 drop_weapons
                            if let Some(m) = g.match_game.as_mut() {
                                m.drop_weapons();
                            }
                        } else if key == "F9" {
                            if let Some(m) = g.match_game.as_mut() {
                                m.destroy_weapons();
                            }
                        } else if key == "F11" {
                            g.toggle_renderer_backend();
                        } else if key == "F10" {
                            g.cycle_view_mode();
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
        {
            let mgr2 = mgr.clone();
            let closure = Closure::wrap(Box::new(move |_ev: MouseEvent| {
                mgr2.borrow_mut().cycle_view_mode();
            }) as Box<dyn FnMut(_)>);
            if let Some(btn) = util::qs(".maximize_button") {
                let _ = btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref());
            }
            closure.forget();
        }
        // F.Lobby start callback from flobby.js → apply room/role on NetworkSession
        {
            let mgr2 = mgr.clone();
            let on_start = Closure::wrap(Box::new(move |payload: JsValue| {
                let s = payload.as_string().unwrap_or_default();
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&s) {
                    let role = v["param"]["role"].as_str().unwrap_or("active");
                    let room = v["param"]["room"]
                        .as_str()
                        .or_else(|| v["param"]["id1"].as_str())
                        .unwrap_or("");
                    let server = v["server"]["address"]
                        .as_str()
                        .or_else(|| v["server"].as_str())
                        .unwrap_or("");
                    let mut g = mgr2.borrow_mut();
                    g.network.apply_lobby_start(server, role, room);
                    g.render_network_dom();
                }
            }) as Box<dyn FnMut(JsValue)>);
            if let Some(win) = web_sys::window() {
                let _ = js_sys::Reflect::set(
                    &win,
                    &"__flf_on_lobby_start".into(),
                    on_start.as_ref().unchecked_ref(),
                );
            }
            on_start.forget();
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
            3 => {
                // cycle difficulty: crazy(-1) → hard(0) → normal(1) → easy(2)
                self.difficulty = match self.difficulty {
                    d if d < 0 => 0,
                    0 => 1,
                    1 => 2,
                    _ => -1,
                };
                self.render_vs_dom();
            }
            4 => self.show_screen(Screen::CharacterSelect),
            _ => self.start_match(),
        }
    }

    fn toggle_renderer_backend(&mut self) {
        self.renderer_kind = match self.renderer_kind {
            RendererKind::Canvas => RendererKind::Dom,
            RendererKind::Dom => RendererKind::Canvas,
        };
        if let Some(ref d) = self.dom_layer {
            d.set_visible(self.renderer_kind == RendererKind::Dom);
        }
        let msg = match self.renderer_kind {
            RendererKind::Canvas => "Renderer: canvas",
            RendererKind::Dom => "Renderer: DOM sprites",
        };
        if let Some(m) = self.match_game.as_mut() {
            m.overlay_message(msg);
        }
        self.network.append_log(msg);
    }

    fn frame(&mut self, ts: f64) {
        let dt = (ts - self.last_tick).max(0.0).min(100.0);
        self.last_tick = ts;
        self.acc += dt;
        let step = 1000.0 / global::FRAMERATE;
        while self.acc >= step {
            self.acc -= step;
            if self.screen == Screen::Gameplay {
                // queue local inputs for future lockstep transport
                if self.network.active {
                    let mut keys = vec![];
                    {
                        let ctrls = self.controllers.borrow();
                        if let Some(c0) = ctrls.first() {
                            for a in ["up", "down", "left", "right", "def", "jump", "att"] {
                                if c0.is_pressed(a) {
                                    keys.push(a.to_string());
                                }
                            }
                        }
                    }
                    let tu = self.match_game.as_ref().map(|m| m.time).unwrap_or(0);
                    self.network.push_local_input(tu, keys);
                    // remote peer -> P2 controller (lockstep application layer)
                    let remote = self.network.poll_remote();
                    if !remote.is_empty() {
                        let mut ctrls = self.controllers.borrow_mut();
                        if let Some(c1) = ctrls.get_mut(1) {
                            c1.clear_states();
                            if let Some(fr) = remote.last() {
                                for k in &fr.keys {
                                    c1.keypress(k);
                                }
                            }
                        }
                    }
                    if let Some(m) = self.match_game.as_ref() {
                        let hps: Vec<f64> = m.characters.iter().map(|c| c.base.hp).collect();
                        self.network.set_verify_hp(&hps);
                    }
                }
                if let Some(m) = self.match_game.as_mut() {
                    m.tu();
                    // Expose F.LF-compatible game_state + TU harness record
                    let gs = m.game_state();
                    if let Some(win) = web_sys::window() {
                        if let Ok(js) = js_sys::JSON::parse(&gs.to_string()) {
                            let _ = js_sys::Reflect::set(&win, &"__flf_game_state".into(), &js);
                        }
                        if let Ok(rec) = js_sys::Reflect::get(&win, &"__flf_tu_record".into()) {
                            if rec.is_function() {
                                if let Ok(js) = js_sys::JSON::parse(&gs.to_string()) {
                                    let args = js_sys::Array::of1(&js);
                                    let _ = js_sys::Reflect::apply(&rec.into(), &win, &args);
                                }
                            }
                        }
                    }
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
                // Optional DOM sprite overlay (F.LF sprite-dom); F11 toggles
                if self.renderer_kind == RendererKind::Dom {
                    if let Some(dom) = self.dom_layer.as_mut() {
                        dom.cam_x = m.camera_x;
                        dom.cam_y = 0.0;
                        dom.clear_frame();
                        for ch in &m.characters {
                            if ch.base.removed {
                                continue;
                            }
                            if let Some(fd) = ch.base.frame_data() {
                                let id = format!("c{}", ch.base.uid);
                                dom.draw_sprite_id(&id, &ch.base.sp, fd.centerx, fd.centery);
                            }
                        }
                        for w in &m.weapons {
                            if let Some(fd) = w.base.frame_data() {
                                let id = format!("w{}", w.base.uid);
                                dom.draw_sprite_id(&id, &w.base.sp, fd.centerx, fd.centery);
                            }
                        }
                        for s in &m.specials {
                            if let Some(fd) = s.base.frame_data() {
                                let id = format!("s{}", s.base.uid);
                                dom.draw_sprite_id(&id, &s.base.sp, fd.centerx, fd.centery);
                            }
                        }
                    }
                } else if let Some(dom) = self.dom_layer.as_ref() {
                    dom.set_visible(false);
                }
                if m.game_over {
                    if let Some(el) = util::qs(".summary_dialog") {
                        let tu = m.match_time_tu();
                        let secs = tu as f64 / global::FRAMERATE;
                        let mut html = format!(
                            "<div class='summary_time'>Time: {secs:.1}s ({tu} TU)</div>\
                             <table class='summary'><tr><th>Name</th><th>Kill</th><th>Attack</th><th>HP</th><th>Team</th></tr>"
                        );
                        for (name, kills, attack, hp, team) in m.summary_rows() {
                            html.push_str(&format!(
                                "<tr><td>{name}</td><td>{kills}</td><td>{attack:.0}</td><td>{hp:.0}</td><td>{team}</td></tr>"
                            ));
                        }
                        html.push_str("</table><p>Esc — menu</p>");
                        el.set_inner_html(&html);
                        let _ = el.set_attribute("style", "display:block");
                    }
                }
            }
        }
    }
}
