//! Character state machine — port of LF/character.js states 0–16
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
        }
    }

    pub fn tu(&mut self, ctrl: Option<&Controller>, bg_z: (f64, f64), bg_w: f64) {
        if self.base.removed {
            return;
        }
        self.combo.tick();
        let state = self.base.state();

        if let Some(ctrl) = ctrl {
            if self.base.held_by.is_none() && !self.base.effect.stuck {
                self.handle_input(ctrl, state);
            }
        } else if self.base.ai {
            // AI applied by match via synthetic controller usually
        }

        // state-specific TU
        match state {
            1 => self.state_walking_tu(),
            2 => self.state_running_tu(),
            4 => self.state_jump_tu(),
            5 => self.state_dash_entry_done(),
            14 => {
                // lying — if wait done and hp>0 get up
                if self.base.frame.wait_left == 0 && self.base.hp > 0.0 {
                    if self.base.data.frames.contains_key(&218) {
                        // will transit via next
                    }
                }
                if self.base.hp <= 0.0 && self.base.counter_dead_blink < 0 {
                    self.base.counter_dead_blink = 0;
                }
            }
            _ => {}
        }

        // dead blink remove
        if self.base.counter_dead_blink >= 0 {
            self.base.counter_dead_blink += 1;
            self.base.effect.blink = true;
            if self.base.counter_dead_blink >= 30 {
                self.base.removed = true;
                self.base.dead = true;
                self.base.sp.visible = false;
            }
        }

        self.base.physics_tu(bg_z, bg_w);
    }

    fn state_walking_tu(&mut self) {
        let bmp = &self.base.data.bmp;
        let speed = bmp.walking_speed;
        let speedz = bmp.walking_speedz;
        // velocity set in input; animate walk frames 5-8
        self.walk_ani += 1;
        if self.walk_ani >= bmp.walking_frame_rate.max(1) {
            self.walk_ani = 0;
        }
        let _ = (speed, speedz);
    }

    fn state_running_tu(&mut self) {
        let bmp = &self.base.data.bmp;
        self.base.ps.vx = bmp.running_speed * self.base.facing as f64;
    }

    fn state_jump_tu(&mut self) {
        // impulse applied on frame 212 entry in handle / state entry
        if self.base.frame.n == 212 && self.base.frame.pn == 211 && self.base.statemem_frame_tu {
            self.base.statemem_frame_tu = false;
            let bmp = &self.base.data.bmp;
            // only if vy not already set strongly
            if self.base.ps.vy >= 0.0 || self.base.ps.vy > bmp.jump_height {
                self.base.ps.vy = bmp.jump_height;
            }
        }
    }

    fn state_dash_entry_done(&mut self) {
        // dash velocity on entry 213/214
        if self.base.statemem_frame_tu && (self.base.frame.n == 213 || self.base.frame.n == 214) {
            self.base.statemem_frame_tu = false;
            let bmp = &self.base.data.bmp;
            if self.base.frame.n == 213 {
                self.base.ps.vx = self.base.facing as f64 * (bmp.dash_distance - 1.0);
            } else {
                self.base.ps.vx = -self.base.facing as f64 * (bmp.dash_distance - 1.0);
            }
            self.base.ps.vy = bmp.dash_height;
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

        // direction
        if self.base.allow_switch_dir && matches!(state, 0 | 1 | 2 | 4 | 5 | 7) {
            if left && !right {
                self.base.facing = -1;
            } else if right && !left {
                self.base.facing = 1;
            }
        }

        // double-tap run (states 0,1)
        if matches!(state, 0 | 1) {
            if left && !self.last_left && ctrl.is_double("left") {
                self.running = true;
                self.base.facing = -1;
                self.base.trans_frame(9, 5);
            }
            if right && !self.last_right && ctrl.is_double("right") {
                self.running = true;
                self.base.facing = 1;
                self.base.trans_frame(9, 5);
            }
        }
        self.last_left = left;
        self.last_right = right;

        // combo feed
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
                    self.combo.clear();
                    self.running = false;
                    return;
                }
            }
        }

        match state {
            0 | 1 => {
                // standing / walking
                if att {
                    if self.base.try_hit_tag("hit_a") || self.base.trans_frame(60, 10) {
                        self.running = false;
                        return;
                    }
                }
                if defend {
                    if self.base.try_hit_tag("hit_d") || self.base.trans_frame(110, 10) {
                        self.running = false;
                        return;
                    }
                }
                if jump {
                    if self.base.try_hit_tag("hit_j") {
                        self.running = false;
                        return;
                    }
                    // jump start 210->211->212
                    self.base.trans_frame(210, 10);
                    self.running = false;
                    return;
                }
                let moving = left || right || up || down;
                if moving {
                    if state == 0 {
                        self.base.trans_frame(5, 2);
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
                } else if state == 1 {
                    self.base.trans_frame(0, 2);
                    self.base.ps.vx = 0.0;
                    self.base.ps.vz = 0.0;
                }
            }
            2 => {
                // running
                if !left && !right {
                    self.running = false;
                    self.base.trans_frame(218, 5); // stop run often 218
                    if !self.base.data.frames.contains_key(&218) {
                        self.base.trans_frame(0, 5);
                    }
                    return;
                }
                if jump {
                    // dash from run
                    self.base.trans_frame(213, 10);
                    self.running = false;
                    return;
                }
                if att {
                    self.base.trans_frame(85, 10); // run attack
                    self.running = false;
                    return;
                }
                if defend {
                    self.base.trans_frame(102, 10); // rowing sometimes
                }
                self.base.ps.vx = bmp.running_speed * self.base.facing as f64;
                if up {
                    self.base.ps.vz = -bmp.running_speedz;
                } else if down {
                    self.base.ps.vz = bmp.running_speedz;
                } else {
                    self.base.ps.vz = 0.0;
                }
            }
            4 => {
                // jump — attack in air
                if (att || ctrl.is_pressed("att")) && self.base.frame.n == 212 && self.base.statemem_attlock == 0 {
                    self.base.trans_frame(80, 10);
                    self.base.statemem_attlock = 2;
                }
            }
            5 => {
                if att {
                    self.base.trans_frame(90, 10); // dash attack area
                }
            }
            7 => {
                // defending — hold def
                if !defend {
                    self.base.trans_frame(0, 5);
                }
                if att && self.base.try_hit_tag("hit_a") {
                    return;
                }
            }
            3 | 11 | 12 | 15 | 16 => {
                // attacks / injury / fall / crouch — limited input
            }
            _ => {}
        }
    }
}
