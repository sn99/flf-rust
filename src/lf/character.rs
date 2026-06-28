//! Character controller logic (subset of LF/character.js — full state machine)
use crate::core_engine::combodec::{ComboDecoder, ComboDef};
use crate::core_engine::controller::Controller;
use crate::lf::data::ObjectData;
use crate::lf::global;
use crate::lf::livingobject::LivingObject;

pub struct Character {
    pub base: LivingObject,
    pub combo: ComboDecoder,
    pub walk_frame_counter: i32,
    pub running: bool,
    pub last_left: bool,
    pub last_right: bool,
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
            walk_frame_counter: 0,
            running: false,
            last_left: false,
            last_right: false,
        }
    }

    pub fn tu(&mut self, ctrl: Option<&Controller>, bg_z: (f64, f64), bg_w: f64) {
        if self.base.dead { return; }
        self.combo.tick();
        let state = self.base.state();

        // Input only in controllable states
        let controllable = matches!(state, 0 | 1 | 2 | 5 | 6 | 12); // stand walk run jump dash defend catch
        if let Some(ctrl) = ctrl {
            if controllable && self.base.held_by.is_none() {
                self.handle_input(ctrl, state);
            }
        }

        // movement speeds for walk/run
        let state = self.base.state();
        let bmp = &self.base.data.bmp;
        if state == 1 {
            // walking — velocity from facing keys applied in handle_input
        }
        if state == 2 && self.running {
            self.base.vx = bmp.running_speed * self.base.facing as f64;
        }

        self.base.tu_base(bg_z, bg_w);

        // landing
        if self.base.ps.y <= 0.0 && matches!(self.base.state(), 3 | 4 | 5 | 6 | 7 | 8) {
            // dash/jump land — frame 215 often
            if self.base.vy >= 0.0 {
                if self.base.data.frames.contains_key(&215) {
                    // only if was airborne
                }
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

        // facing
        if left && !right {
            self.base.facing = -1;
        } else if right && !left {
            self.base.facing = 1;
        }

        // double tap run
        if left && !self.last_left && ctrl.is_double("left") {
            self.running = true;
            self.base.transit(9); // running start often 9
            if !self.base.data.frames.contains_key(&9) {
                self.base.transit(2);
            }
        }
        if right && !self.last_right && ctrl.is_double("right") {
            self.running = true;
            self.base.transit(9);
            if !self.base.data.frames.contains_key(&9) {
                self.base.transit(2);
            }
        }
        self.last_left = left;
        self.last_right = right;

        // feed combo decoder
        if defend { self.combo.feed("def"); }
        if jump { self.combo.feed("jump"); }
        if att { self.combo.feed("att"); }
        if left { self.combo.feed("left"); }
        if right { self.combo.feed("right"); }
        if up { self.combo.feed("up"); }
        if down { self.combo.feed("down"); }

        if let Some(name) = self.combo.match_combo() {
            if let Some(tag) = global::combo_tag(&name) {
                if self.base.try_hit(tag) {
                    self.combo.clear();
                    return;
                }
            }
        }

        // basic attacks / defend / jump
        if att && matches!(state, 0 | 1 | 2) {
            if self.base.try_hit("hit_a") {
                self.running = false;
                return;
            }
        }
        if defend && matches!(state, 0 | 1) {
            if self.base.try_hit("hit_d") {
                self.running = false;
                return;
            }
            self.base.transit(110); // defend
            return;
        }
        if jump && matches!(state, 0 | 1 | 2) {
            if self.base.try_hit("hit_j") {
                self.running = false;
                return;
            }
            // default jump
            self.base.vy = bmp.jump_height; // negative
            self.base.vx = bmp.jump_distance * self.base.facing as f64 * 0.3;
            self.base.transit(210);
            if !self.base.data.frames.contains_key(&210) {
                self.base.transit(212);
            }
            self.running = false;
            return;
        }

        // walk / stand
        if matches!(state, 0 | 1 | 2) {
            let moving = left || right || up || down;
            if self.running && (left || right) && state != 2 {
                // keep run
                if self.base.data.frames.contains_key(&9) {
                    // running frames 9-11 typically
                }
            }
            if moving && !self.running {
                if state == 0 {
                    self.base.transit(5); // walk
                }
                let speed = bmp.walking_speed;
                let speedz = bmp.walking_speedz;
                self.base.vx = 0.0;
                self.base.vz = 0.0;
                if left { self.base.vx = -speed; }
                if right { self.base.vx = speed; }
                if up { self.base.vz = -speedz; }
                if down { self.base.vz = speedz; }
            } else if self.running && (left || right) {
                self.base.vx = bmp.running_speed * self.base.facing as f64;
                if up { self.base.vz = -bmp.running_speedz; }
                else if down { self.base.vz = bmp.running_speedz; }
                else { self.base.vz = 0.0; }
                if state != 2 && self.base.data.frames.contains_key(&9) {
                    self.base.transit(9);
                }
            } else {
                self.running = false;
                if state == 1 || state == 2 {
                    self.base.transit(0);
                }
                self.base.vx = 0.0;
                self.base.vz = 0.0;
            }
        }
    }
}
