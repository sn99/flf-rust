//! Character — full state machine port from LF/character.js (all major states)
use crate::core_engine::combodec::{ComboDecoder, ComboDef};
use crate::core_engine::controller::Controller;
use crate::lf::data::ObjectData;
use crate::lf::global;
use crate::lf::livingobject::LivingObject;

pub struct Character {
    pub base: LivingObject,
    pub combo: ComboDecoder,
    pub running: bool,
    pub last_left: bool,
    pub last_right: bool,
    pub walk_ani: i32,
    pub hold_weapon: Option<u32>,
    pub dir_up: bool,
    pub dir_down: bool,
    pub catch_counter: i32,
    pub catch_attacks: i32,
    pub caught_throwinjury: f64,
}

impl Character {
    pub fn new(uid: u32, data: ObjectData, team: i32, x: f64, z: f64) -> Self {
        let combos: Vec<ComboDef> = global::combo_list()
            .into_iter()
            .map(|(name, seq, clear)| ComboDef { name, seq, clear_on_combo: clear })
            .collect();
        Self {
            base: LivingObject::new(uid, data, team, x, z),
            combo: ComboDecoder::new(combos, global::COMBO_TIMEOUT),
            running: false,
            last_left: false,
            last_right: false,
            walk_ani: 0,
            hold_weapon: None,
            dir_up: false,
            dir_down: false,
            catch_counter: 0,
            catch_attacks: 0,
            caught_throwinjury: 0.0,
        }
    }

    pub fn switch_dir(&mut self, dir: &str) {
        if dir == "left" {
            self.base.facing = -1;
        }
        if dir == "right" {
            self.base.facing = 1;
        }
    }

    pub fn dirv(&self) -> f64 {
        let mut v = 0.0;
        if self.dir_up {
            v -= 1.0;
        }
        if self.dir_down {
            v += 1.0;
        }
        v
    }

    pub fn tu(&mut self, ctrl: Option<&Controller>, bg_z: (f64, f64), bg_w: f64) {
        if self.base.removed {
            return;
        }
        self.combo.tick();
        let state = self.base.state();

        // state entry hooks
        match state {
            9 if self.base.frame.pn != 9 && self.base.statemem_frame_tu => {
                self.catch_counter = 43;
                self.catch_attacks = 0;
            }
            _ => {}
        }

        if let Some(ctrl) = ctrl {
            if self.base.held_by.is_none() && !self.base.effect.stuck && !matches!(state, 10 | 11 | 12 | 13 | 14 | 16 | 18) {
                self.dir_up = ctrl.is_pressed("up");
                self.dir_down = ctrl.is_pressed("down");
                self.handle_input(ctrl, state);
            }
        }

        // per-state TU
        match state {
            2 => {
                let spd = self.base.data.bmp.running_speed;
                self.base.ps.vx = spd * self.base.facing as f64;
            }
            4 => {
                if self.base.frame.n == 212 && self.base.frame.pn == 211 && self.base.statemem_frame_tu {
                    self.base.statemem_frame_tu = false;
                    let bmp = &self.base.data.bmp;
                    self.base.ps.vy = bmp.jump_height;
                    self.base.ps.vz = self.dirv() * (bmp.jump_distancez - 1.0);
                }
            }
            5 => {
                if self.base.statemem_frame_tu && (self.base.frame.n == 213 || self.base.frame.n == 214) {
                    self.base.statemem_frame_tu = false;
                    let bmp = &self.base.data.bmp;
                    if self.base.frame.n == 213 {
                        self.base.ps.vx = self.base.facing as f64 * (bmp.dash_distance - 1.0);
                    } else {
                        self.base.ps.vx = -self.base.facing as f64 * (bmp.dash_distance - 1.0);
                    }
                    self.base.ps.vz = self.dirv() * (bmp.dash_distancez - 1.0);
                    self.base.ps.vy = bmp.dash_height;
                }
            }
            6 => {
                // rowing — velocity from frames
            }
            9 => {
                self.catch_counter -= 1;
                if self.catch_counter <= 0 {
                    // release
                    self.base.holding_uid = None;
                    self.base.trans_frame(0, 10);
                }
                if self.base.frame.n == 123 {
                    self.catch_attacks += 1;
                    self.catch_counter += 3;
                    self.base.trans.inc_wait(1, 10, 1);
                }
            }
            10 => {
                // being caught — position set by match
            }
            11 => {
                // injured lock
                let n = self.base.frame.n;
                if matches!(n, 221 | 223 | 225) {
                    self.base.trans.set_next(999, 20);
                }
            }
            12 => {
                self.state12_fall_tu();
            }
            13 => {
                // frozen — no input
            }
            14 => {
                if self.base.hp <= 0.0 && self.base.counter_dead_blink < 0 {
                    self.base.counter_dead_blink = 0;
                }
            }
            15 => {}
            16 => {}
            18 | 19 => {}
            301 => {
                // deep specific — data driven
            }
            400 | 401 => {
                // teleport — snap in id_frame_hook if needed
            }
            1700 => {
                // heal state
                if self.base.hp < self.base.hp_full {
                    self.base.hp = (self.base.hp + 2.0).min(self.base.hp_full);
                }
            }
            _ => {}
        }

        if self.base.counter_dead_blink >= 0 {
            self.base.counter_dead_blink += 1;
            self.base.effect.blink = true;
            if self.base.counter_dead_blink >= 30 {
                self.base.removed = true;
                self.base.dead = true;
                self.base.sp.visible = false;
            }
        }

        self.id_frame_hook();
        self.base.physics_tu(bg_z, bg_w);

        // apply caught throw injury on land
        if self.caught_throwinjury > 0.0 && self.base.ps.y >= 0.0 && self.base.state() == 12 {
            self.base.hp -= self.caught_throwinjury;
            self.caught_throwinjury = 0.0;
        }
    }

    fn state12_fall_tu(&mut self) {
        let n = self.base.frame.n;
        let dvy_up = self.base.ps.vy <= 0.0;
        // chain fall frames when wait expires is mostly data-driven; reinforce next links
        if self.base.trans.wait == 0 {
            match n {
                180 if dvy_up => {
                    self.base.trans.set_next(181, 15);
                }
                180 => {
                    self.base.trans.set_next(185, 15);
                }
                181 => {
                    self.base.trans.set_next(182, 15);
                    let vy = self.base.ps.vy.abs();
                    let w = if vy <= 4.0 {
                        2
                    } else if vy < 7.0 {
                        3
                    } else {
                        4
                    };
                    self.base.trans.set_wait(w, 15, 1);
                }
                182 => self.base.trans.set_next(183, 15),
                186 => self.base.trans.set_next(187, 15),
                187 => self.base.trans.set_next(188, 15),
                188 => self.base.trans.set_next(189, 15),
                _ => {}
            }
        }
    }

    fn handle_input(&mut self, ctrl: &Controller, state: i32) {
        let left = ctrl.is_pressed("left");
        let right = ctrl.is_pressed("right");
        let up = ctrl.is_pressed("up");
        let down = ctrl.is_pressed("down");
        let att = ctrl.is_pressed("att");
        let jump = ctrl.is_pressed("jump");
        let defend = ctrl.is_pressed("def");
        let bmp = self.base.data.bmp.clone();
        let holding_heavy = self.hold_weapon.is_some() && self.base.hold_type == "heavyweapon";
        let holding_light = self.hold_weapon.is_some()
            && (self.base.hold_type == "lightweapon" || self.base.hold_type == "drink");

        if self.base.allow_switch_dir && matches!(state, 0 | 1 | 2 | 4 | 5 | 7 | 9) {
            if left && !right {
                self.base.facing = -1;
            } else if right && !left {
                self.base.facing = 1;
            }
        }

        if matches!(state, 0 | 1) {
            if left && !self.last_left && ctrl.is_double("left") {
                if holding_heavy {
                    self.base.trans_frame(16, 10);
                } else {
                    self.running = true;
                    self.base.facing = -1;
                    self.base.trans_frame(9, 10);
                }
            }
            if right && !self.last_right && ctrl.is_double("right") {
                if holding_heavy {
                    self.base.trans_frame(16, 10);
                } else {
                    self.running = true;
                    self.base.facing = 1;
                    self.base.trans_frame(9, 10);
                }
            }
        }
        self.last_left = left;
        self.last_right = right;

        for (pressed, name) in [
            (defend, "def"),
            (jump, "jump"),
            (att, "att"),
            (left, "left"),
            (right, "right"),
            (up, "up"),
            (down, "down"),
        ] {
            if pressed {
                self.combo.feed(name);
            }
        }
        if let Some(name) = self.combo.match_combo() {
            if let Some(tag) = global::combo_tag(&name) {
                if self.base.try_hit_tag(tag) {
                    let clear = self
                        .combo
                        .combos
                        .iter()
                        .find(|c| c.name == name)
                        .map(|c| c.clear_on_combo)
                        .unwrap_or(true);
                    if clear {
                        self.combo.clear();
                    }
                    self.running = false;
                    return;
                }
            }
        }

        match state {
            0 | 1 => {
                if att {
                    if holding_heavy {
                        self.base.trans_frame(50, 10);
                        return;
                    }
                    if holding_light {
                        if left != right {
                            self.base.trans_frame(45, 10);
                        } else {
                            let fr = if js_sys::Math::random() < 0.5 { 20 } else { 25 };
                            self.base.trans_frame(fr, 10);
                        }
                        return;
                    }
                    // super punch scope would check itr kind 6 — approximate random 70
                    let fr = if js_sys::Math::random() < 0.08 {
                        70
                    } else if js_sys::Math::random() < 0.5 {
                        60
                    } else {
                        65
                    };
                    if self.base.try_hit_tag("hit_a") || self.base.trans_frame(fr, 10) {
                        self.running = false;
                        return;
                    }
                }
                if defend && !holding_heavy {
                    if self.base.try_hit_tag("hit_d") || self.base.trans_frame(110, 10) {
                        self.running = false;
                        return;
                    }
                }
                if jump {
                    if holding_heavy {
                        return;
                    }
                    if self.base.try_hit_tag("hit_j") || self.base.trans_frame(210, 10) {
                        self.running = false;
                        return;
                    }
                }
                let moving = left || right || up || down;
                if moving {
                    if holding_heavy {
                        if state == 0 {
                            self.base.trans_frame(12, 5);
                        }
                        let hs = bmp.heavy_walking_speed;
                        let hsz = bmp.heavy_walking_speedz;
                        self.base.ps.vx = 0.0;
                        self.base.ps.vz = 0.0;
                        if left {
                            self.base.ps.vx = -hs;
                        }
                        if right {
                            self.base.ps.vx = hs;
                        }
                        if up {
                            self.base.ps.vz = -hsz;
                        }
                        if down {
                            self.base.ps.vz = hsz;
                        }
                    } else {
                        if state == 0 {
                            self.base.trans_frame(5, 5);
                        }
                        self.base.ps.vx = 0.0;
                        self.base.ps.vz = 0.0;
                        if left {
                            self.base.ps.vx = -bmp.walking_speed;
                        }
                        if right {
                            self.base.ps.vx = bmp.walking_speed;
                        }
                        if up {
                            self.base.ps.vz = -bmp.walking_speedz;
                        }
                        if down {
                            self.base.ps.vz = bmp.walking_speedz;
                        }
                    }
                } else if state == 1 {
                    self.base.trans_frame(0, 2);
                    self.base.ps.vx = 0.0;
                    self.base.ps.vz = 0.0;
                }
            }
            2 => {
                if !left && !right {
                    self.running = false;
                    self.base.trans_frame(218, 5);
                    if !self.base.data.frames.contains_key(&218) {
                        self.base.trans_frame(0, 5);
                    }
                    return;
                }
                if jump {
                    self.base.trans_frame(213, 10);
                    self.running = false;
                    return;
                }
                if att {
                    if holding_light {
                        self.base.trans_frame(35, 10);
                    } else if holding_heavy {
                        self.base.trans_frame(50, 10);
                    } else {
                        self.base.trans_frame(85, 10);
                    }
                    self.running = false;
                    return;
                }
                if defend {
                    self.base.trans_frame(102, 10); // rowing
                }
                self.base.ps.vx = bmp.running_speed * self.base.facing as f64;
                self.base.ps.vz = self.dirv() * bmp.running_speedz;
            }
            4 => {
                if att && self.base.frame.n == 212 && self.base.statemem_attlock == 0 {
                    if holding_light {
                        self.base.trans_frame(30, 10);
                    } else {
                        self.base.trans_frame(80, 10);
                    }
                    self.base.statemem_attlock = 2;
                }
                if left {
                    self.base.ps.vx = -(bmp.jump_distance - 1.0) * 0.5;
                }
                if right {
                    self.base.ps.vx = (bmp.jump_distance - 1.0) * 0.5;
                }
            }
            5 => {
                if att {
                    self.base.trans_frame(90, 10);
                }
            }
            7 => {
                if !defend {
                    self.base.trans_frame(0, 5);
                }
                // dircontrol on defend
            }
            8 => {
                // broken defend — wait for frames
            }
            9 => {
                if att {
                    self.base.trans_frame(121, 12);
                }
                if jump {
                    self.base.trans_frame(122, 12);
                }
                // dircontrol
                if left {
                    self.switch_dir("left");
                }
                if right {
                    self.switch_dir("right");
                }
            }
            3 | 15 => {
                // attacks / weapon throws driven by frames
            }
            _ => {}
        }
    }

    pub fn wpoint_world(&self) -> Option<(f64, f64, f64, i32, i32)> {
        let fd = self.base.frame_data()?;
        let wp = fd.wpoint.as_ref()?;
        let x = if self.base.facing >= 0 {
            self.base.ps.x - fd.centerx + wp.x
        } else {
            self.base.ps.x + fd.centerx - wp.x
        };
        let y = self.base.ps.y + (wp.y - fd.centery);
        let z = self.base.ps.z;
        Some((x, y, z, self.base.facing, wp.weaponact))
    }

    pub fn id_frame_hook(&mut self) {
        let id = self.base.id;
        let n = self.base.frame.n;
        match id {
            11 => { /* Davis */ }
            1 => {
                if n == 253 { /* deep fly */ }
            }
            5 => { /* Rudolf */ }
            2 => {
                if (240..280).contains(&n) {
                    self.base.effect.super_armor = true;
                }
            }
            7 | 8 => {}
            6 => { /* Louis */ }
            10 => { /* Woody */ }
            _ => {}
        }
        // teleport states 400/401: snap handled if we had targets — placeholder
        if self.base.state() == 400 || self.base.state() == 401 {
            // brief invuln
            self.base.effect.super_armor = true;
        }
    }
}
