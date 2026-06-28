//! Character — expanded port of LF/character.js
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
    /// weapon uid held
    pub hold_weapon: Option<u32>,
    pub dir_up: bool,
    pub dir_down: bool,
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
        }
    }

    pub fn dirv(&self) -> f64 {
        let mut v = 0.0;
        if self.dir_up { v -= 1.0; }
        if self.dir_down { v += 1.0; }
        v
    }

    pub fn tu(&mut self, ctrl: Option<&Controller>, bg_z: (f64, f64), bg_w: f64) {
        if self.base.removed {
            return;
        }
        self.combo.tick();
        let state = self.base.state();

        if let Some(ctrl) = ctrl {
            if self.base.held_by.is_none() && !self.base.effect.stuck {
                self.dir_up = ctrl.is_pressed("up");
                self.dir_down = ctrl.is_pressed("down");
                self.handle_input(ctrl, state);
            }
        }

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
                    // horizontal from keys applied in input
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
            14 => {
                if self.base.hp <= 0.0 && self.base.counter_dead_blink < 0 {
                    self.base.counter_dead_blink = 0;
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

        // wpoint weaponact while holding — frame weaponact applied in match
        self.base.physics_tu(bg_z, bg_w);
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
        let holding_light = self.hold_weapon.is_some() && self.base.hold_type == "lightweapon";

        if self.base.allow_switch_dir && matches!(state, 0 | 1 | 2 | 4 | 5 | 7) {
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
            (defend, "def"), (jump, "jump"), (att, "att"),
            (left, "left"), (right, "right"), (up, "up"), (down, "down"),
        ] {
            if pressed { self.combo.feed(name); }
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
                if att {
                    if holding_heavy {
                        self.base.trans_frame(50, 10); // throw heavy
                        return;
                    }
                    if holding_light {
                        let dx = left != right;
                        if dx {
                            self.base.trans_frame(45, 10); // throw
                        } else {
                            // weapon attack 20/25
                            let fr = if js_sys::Math::random() < 0.5 { 20 } else { 25 };
                            self.base.trans_frame(fr, 10);
                        }
                        return;
                    }
                    // normal punch 60/65 or super 70
                    let fr = if js_sys::Math::random() < 0.5 { 60 } else { 65 };
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
                    if state == 0 && !holding_heavy {
                        self.base.trans_frame(5, 5);
                    }
                    if holding_heavy {
                        if state == 0 {
                            self.base.trans_frame(12, 5);
                        }
                        let hs = bmp.heavy_walking_speed;
                        let hsz = bmp.heavy_walking_speedz;
                        self.base.ps.vx = 0.0;
                        self.base.ps.vz = 0.0;
                        if left { self.base.ps.vx = -hs; }
                        if right { self.base.ps.vx = hs; }
                        if up { self.base.ps.vz = -hsz; }
                        if down { self.base.ps.vz = hsz; }
                    } else {
                        self.base.ps.vx = 0.0;
                        self.base.ps.vz = 0.0;
                        if left { self.base.ps.vx = -bmp.walking_speed; }
                        if right { self.base.ps.vx = bmp.walking_speed; }
                        if up { self.base.ps.vz = -bmp.walking_speedz; }
                        if down { self.base.ps.vz = bmp.walking_speedz; }
                    }
                } else if state == 1 || (holding_heavy && state != 0) {
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
                    } else {
                        self.base.trans_frame(85, 10);
                    }
                    self.running = false;
                    return;
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
                // air steer
                if left { self.base.ps.vx = -bmp.jump_distance * 0.5; }
                if right { self.base.ps.vx = bmp.jump_distance * 0.5; }
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
            }
            9 => {
                // catching — att throws (frame 121+)
                if att {
                    self.base.trans_frame(121, 12);
                }
                if jump {
                    self.base.trans_frame(122, 12);
                }
            }
            10 => {
                // being caught — no input
            }
            3 | 11 | 12 | 15 | 16 | 8 => {}
            _ => {}
        }
    }

    /// Sync held weapon position from wpoint
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
}
