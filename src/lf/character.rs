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
    /// Rudolf transform bookkeeping (LF character.js transform_character)
    pub is_rudolf_transform: bool,
    pub transform_target_uid: Option<u32>,
    pub transform_target_id: i32,
    pub transform_caught_id: i32,
    pub pending_transform: bool,
    pub pending_revert_transform: bool,
    /// spawn broken effect id on ice exit (state 13)
    pub pending_broken_effect: i32,
    /// cpoint injury applied once per catch TU sync
    pub catch_injury_pending: bool,
    /// set by match when super-punch scope (frame 72/73) sees itr kind 6 victim
    pub want_super_punch: bool,
    /// drink weapon sip TU counter
    pub drink_sips: i32,
    /// held weapon object id (for properties.js keys)
    pub hold_weapon_oid: i32,
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
            is_rudolf_transform: false,
            transform_target_uid: None,
            transform_target_id: 0,
            transform_caught_id: 0,
            pending_transform: false,
            pending_revert_transform: false,
            pending_broken_effect: 0,
            catch_injury_pending: false,
            want_super_punch: false,
            drink_sips: 0,
            hold_weapon_oid: 0,
        }
    }

    /// properties.js entry for held weapon id
    pub fn weapon_proper_bool(&self, prop: &str) -> bool {
        if self.hold_weapon_oid == 0 {
            return false;
        }
        let id = self.hold_weapon_oid.to_string();
        self.base
            .properties
            .get(&id)
            .and_then(|o| o.get(prop))
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
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
                // rowing — push with rowing_distance
                let rd = self.base.data.bmp.rowing_distance;
                let rh = self.base.data.bmp.rowing_height;
                if self.base.statemem_frame_tu {
                    self.base.statemem_frame_tu = false;
                    self.base.ps.vx = self.base.facing as f64 * rd;
                    if rh != 0.0 {
                        self.base.ps.vy = rh;
                    }
                }
            }
            9 => {
                self.catch_counter -= 1;
                if self.catch_counter <= 0 {
                    // release unless finishing 5th punch on 122
                    if !(self.base.frame.n == 122 && self.catch_attacks == 4) {
                        if matches!(self.base.frame.n, 121 | 122) {
                            self.base.holding_uid = None;
                            self.base.trans_frame(0, 15);
                        }
                    }
                }
                match self.base.frame.n {
                    123 => {
                        self.catch_attacks += 1;
                        self.catch_counter += 3;
                        self.base.trans.inc_wait(1, 10, 1);
                    }
                    233 | 234 => {
                        self.base.trans.inc_wait(-1, 10, 1);
                    }
                    240 => {
                        let _ = crate::lf::character_ids::id_update(self, "rudolf_transform", None);
                    }
                    _ => {}
                }
                // cover zz from cpoint
                if let Some(fd) = self.base.frame_data() {
                    if let Some(cp) = &fd.cpoint {
                        let cover = if cp.cover != 0 { cp.cover } else { 0 };
                        self.base.ps.zz = if cover == 0 || cover == 10 { 1.0 } else { -1.0 };
                    }
                }
            }
            10 => {
                // being caught — lift against gravity on 135
                if self.base.frame.n == 135 {
                    self.base.ps.vy = 0.0;
                }
                self.base.trans.set_wait(99, 10, 99);
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
                // frozen — no input; ice shatter on exit
                let leaving = self
                    .base
                    .frame_data()
                    .and_then(|fd| self.base.data.frames.get(&fd.next).map(|f| f.state))
                    .unwrap_or(13);
                if leaving != 13 && self.base.trans.wait <= 1 {
                    self.base.request_broken(212, 8);
                }
            }
            14 => {
                // lying entry: clear fall meters
                if self.base.frame.pn != 14 {
                    self.base.fall = 0.0;
                    self.base.bdefend = 0.0;
                }
                if self.base.hp <= 0.0 && self.base.counter_dead_blink < 0 {
                    self.base.counter_dead_blink = 0;
                }
                // leaving lie → invuln blink (state_exit 14 in character.js)
                // applied when next state != 14 via frame transition watch:
                let next_st = self
                    .base
                    .frame_data()
                    .and_then(|fd| self.base.data.frames.get(&fd.next).map(|f| f.state))
                    .unwrap_or(14);
                if next_st != 14 && self.base.trans.wait <= 1 {
                    self.base.effect.blink = true;
                    self.base.effect.super_armor = true;
                    self.base.effect.timeout = 30;
                }
            }
            15 => {
                // stop_running / crouch / weapon throws
                let n = self.base.frame.n;
                let pn = self.base.frame.pn;
                if n == 19 && self.hold_weapon.is_some() && self.base.hold_type == "heavyweapon" {
                    self.base.trans.set_next(12, 10);
                }
                if n == 215 {
                    self.base.trans.inc_wait(-1, 10, 1);
                }
                if n == 219 {
                    if !crate::lf::character_ids::id_update(self, "state15_crouch", None) {
                        match pn {
                            105 => {
                                // unit friction after rowing
                                self.base.ps.vx *= 0.5;
                                self.base.ps.vz *= 0.5;
                            }
                            216 | 90 | 91 | 92 => {
                                self.base.trans.inc_wait(-1, 10, 1);
                            }
                            _ => {}
                        }
                    }
                }
                if n == 54 {
                    if let Some(fd) = self.base.frame_data() {
                        if fd.next == 999 && self.base.ps.y < 0.0 {
                            self.base.trans.set_next(212, 10);
                        }
                    }
                }
                if n == 257 {
                    let _ = crate::lf::character_ids::id_update(self, "state1280_disappear", None);
                }
            }
            16 => {
                // injured 2 / dance of pain — vulnerable to catch (kind 1) and super catch (kind 3)
                // lock until frame advances; allow slight blink
                self.base.effect.blink = self.base.trans.wait % 3 == 0;
                self.base.allow_switch_dir = false;
            }
            18 | 19 => {
                // firen-specific handled in id_tu; generic burn drift
                // land while burning → fall chain like state 12
                if self.base.ps.y >= 0.0 && self.base.ps.vy >= 0.0 && self.base.frame.pn != 14 {
                    // fall_onto_ground for burn delegates to fall frames
                    if self.base.frame.n < 180 || self.base.frame.n > 191 {
                        self.base.trans_frame(185, 15);
                    }
                }
            }
            301 => {
                // deep specific — walking_speedz on TU in id_tu
                if self.base.frame.n != 290 {
                    // frame_force disabled for non-290 (state3_frame_force pattern)
                }
            }
            400 | 401 => {
                // teleport — match applies targets on frame entry
            }
            501 => {
                // generic special / rudolf transform finish
                self.base.allow_switch_dir = false;
                if self.base.frame.n == 298 {
                    let _ = crate::lf::character_ids::id_update(self, "rudolf_transform", None);
                }
            }
            1700 => {
                // heal state — set heal aura duration
                if self.base.frame.pn != 1700 {
                    self.base.effect.timeout = 30;
                    self.base.effect.super_armor = true;
                }
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
                    let w = global::fall_wait180(self.base.ps.vy);
                    self.base.trans.set_wait(w, 15, 1);
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
                // DJA revert transform for Rudolf copies
                if name == "DJA" && self.is_rudolf_transform {
                    let _ = crate::lf::character_ids::id_update(self, "revert_transform", None);
                    self.combo.clear();
                    self.running = false;
                    return;
                }
                // Deep faces along Fj direction
                if tag == "hit_Fj" && self.base.id == 1 {
                    if name.contains('>') || name == "D>J" || name == "D>AJ" {
                        self.switch_dir("right");
                    } else {
                        self.switch_dir("left");
                    }
                }
                if crate::lf::character_ids::id_update(self, "generic_combo", Some(tag)) {
                    // blocked (e.g. Louis hit_ja)
                    self.combo.clear();
                    return;
                }
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
                        // properties.js on weapon id: stand_throw / just_throw / attackable
                        if self.weapon_proper_bool("just_throw") {
                            self.base.trans_frame(45, 10);
                            return;
                        }
                        let dx = left != right;
                        if dx && self.weapon_proper_bool("stand_throw") {
                            self.base.trans_frame(45, 10);
                            return;
                        }
                        if self.weapon_proper_bool("attackable")
                            || !self.weapon_proper_bool("stand_throw")
                        {
                            if left != right {
                                self.base.trans_frame(45, 10);
                            } else {
                                let fr = if js_sys::Math::random() < 0.5 { 20 } else { 25 };
                                self.base.trans_frame(fr, 10);
                            }
                            return;
                        }
                        self.base.trans_frame(45, 10);
                        return;
                    }
                    // super punch: F.LF checks frames 72/73 itr volume for victims with itr kind 6
                    // Match sets `want_super_punch` when scope hits; fallback heuristic near-random
                    let fr = if self.want_super_punch {
                        self.want_super_punch = false;
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
                        if !self.base.proper_bool("heavy_weapon_jump") {
                            return;
                        }
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
                    // back attack only if property allows when facing wrong way — simplify always allow unless false
                    if self.base.proper_bool("dash_backattack") || true {
                        self.base.trans_frame(90, 10);
                    }
                }
            }
            7 => {
                if !defend {
                    self.base.trans_frame(0, 5);
                }
                // dircontrol on defend
            }
            8 => {
                // broken defend — stun; clear bdefend gradually
                self.base.bdefend = (self.base.bdefend - 2.0).max(0.0);
                self.base.allow_switch_dir = false;
            }
            9 => {
                // cpoint taction / aaction / jaction
                if let Some(fd) = self.base.frame_data().cloned() {
                    if let Some(cp) = fd.cpoint.clone() {
                        if att {
                            let dx = left != right;
                            let dy = up != down;
                            if (dx || dy) && cp.taction != 0 {
                                let tac = cp.taction;
                                if tac < 0 {
                                    let nd = if self.base.facing > 0 { "left" } else { "right" };
                                    self.switch_dir(nd);
                                    self.base.trans_frame(-tac, 10);
                                } else {
                                    self.base.trans_frame(tac, 10);
                                }
                                self.catch_counter += 10;
                            } else if cp.aaction != 0 {
                                self.base.trans_frame(cp.aaction, 10);
                            } else {
                                self.base.trans_frame(121, 12);
                            }
                        }
                        if jump && self.base.frame.n == 121 && cp.jaction != 0 {
                            self.base.trans_frame(cp.jaction, 10);
                        } else if jump {
                            self.base.trans_frame(122, 12);
                        }
                        if cp.dircontrol == 1 {
                            if left {
                                self.switch_dir("left");
                            }
                            if right {
                                self.switch_dir("right");
                            }
                        }
                    } else {
                        if att {
                            self.base.trans_frame(121, 12);
                        }
                        if jump {
                            self.base.trans_frame(122, 12);
                        }
                    }
                }
                if left {
                    self.switch_dir("left");
                }
                if right {
                    self.switch_dir("right");
                }
            }
            12 => {
                // fall recovery jump on 182/188
                let n = self.base.frame.n;
                if jump && matches!(n, 182 | 188) && self.base.fall < global::FALL_KO && self.base.hp > 0.0 {
                    if n == 182 {
                        self.base.trans_frame(100, 10);
                    } else {
                        self.base.trans_frame(108, 10);
                    }
                    if self.base.ps.vx != 0.0 {
                        self.base.ps.vx = 5.0 * self.base.ps.vx.signum();
                    }
                    if self.base.ps.vy == 0.0 {
                        self.base.ps.vy = 5.0;
                    }
                    if self.base.ps.vz != 0.0 {
                        self.base.ps.vz = 2.0 * self.base.ps.vz.signum();
                    }
                }
            }
            3 => {
                // attack state — id_update state3_frame on frame change
                let _ = crate::lf::character_ids::id_update(self, "state3_frame", None);
            }
            15 => {
                // crouch after jump: def→rowing, jump→dash variants
                if self.base.frame.n == 215 {
                    if defend {
                        self.base.trans_frame(102, 10);
                    }
                    if jump {
                        let mut dx = 0i32;
                        if left {
                            dx -= 1;
                        }
                        if right {
                            dx += 1;
                        }
                        if dx != 0 {
                            self.base.trans_frame(213, 10);
                            self.switch_dir(if dx == 1 { "right" } else { "left" });
                        } else if self.base.ps.vx == 0.0 {
                            self.base.trans.inc_wait(2, 10, 99);
                            self.base.trans.set_next(210, 10);
                        } else if (self.base.ps.vx > 0.0) == (self.base.facing > 0) {
                            self.base.trans_frame(213, 10);
                        } else {
                            self.base.trans_frame(214, 10);
                        }
                    }
                }
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
        crate::lf::character_ids::id_tu(self);
    }

    /// Teleport after match provides targets
    pub fn apply_teleport_targets(
        &mut self,
        nearest_enemy: Option<(f64, f64)>,
        furthest_ally: Option<(f64, f64)>,
    ) {
        crate::lf::character_ids::apply_teleport(self, nearest_enemy, furthest_ally);
    }
}

