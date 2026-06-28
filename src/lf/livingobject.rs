//! Living object — faithful subset of LF/livingobject.js
use crate::core_engine::sprite::SpriteInstance;
use crate::lf::data::{frame_hit_tag, FrameData, ObjectData};
use crate::lf::global;
use crate::lf::mechanics::Mech;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Default)]
pub struct Pos {
    pub x: f64,
    pub y: f64,  // height (negative = airborne in LF2)
    pub z: f64,  // depth
    pub vx: f64,
    pub vy: f64, // negative = up
    pub vz: f64,
    pub zz: f64, // render order helper
}

#[derive(Clone, Debug, Default)]
pub struct EffectState {
    pub num: i32,
    pub dvx: f64,
    pub dvy: f64,
    pub stuck: bool,
    pub oscillate: f64,
    pub blink: bool,
    pub super_armor: bool,
    pub timein: i32,
    pub timeout: i32,
}

#[derive(Clone, Debug)]
pub struct FrameState {
    pub pn: i32,
    pub n: i32,
    pub wait_left: i32,
    /// lock priority for transition (higher wins)
    pub lock: i32,
}

impl Default for FrameState {
    fn default() -> Self {
        Self { pn: 0, n: 0, wait_left: 0, lock: 0 }
    }
}

pub struct LivingObject {
    pub id: i32,
    pub uid: u32,
    pub team: i32,
    pub obj_type: String,
    pub data: ObjectData,
    pub ps: Pos,
    pub facing: i32, // 1 right, -1 left (dirh)
    pub hp: f64,
    pub hp_full: f64,
    pub mp: f64,
    pub mp_full: f64,
    pub fall: f64,
    pub bdefend: f64,
    pub arest: i32,
    pub vrest: HashMap<u32, i32>,
    pub frame: FrameState,
    pub sp: SpriteInstance,
    pub mech: Mech,
    pub effect: EffectState,
    pub holding_uid: Option<u32>,
    pub hold_type: String,
    pub held_by: Option<u32>,
    pub dead: bool,
    pub removed: bool,
    pub ai: bool,
    pub controller_index: Option<usize>,
    pub allow_switch_dir: bool,
    pub name: String,
    pub counter_disappear: i32,
    pub counter_dead_blink: i32,
    pub statemem_attlock: i32,
    pub statemem_frame_tu: bool,
    pub injury_total: f64,
    pub kills: i32,
    pub enter_frame_applied: bool,
    pub opoint_spawned: bool,
    pub frame_sound: String,
}

impl LivingObject {
    pub fn new(uid: u32, data: ObjectData, team: i32, x: f64, z: f64) -> Self {
        let sheets = data.bmp.sheets.clone();
        let name = data.bmp.name.clone();
        let mut lo = Self {
            id: data.id,
            uid,
            team,
            obj_type: data.obj_type.clone(),
            data,
            ps: Pos { x, y: 0.0, z, vx: 0.0, vy: 0.0, vz: 0.0, zz: 0.0 },
            facing: 1,
            hp: global::HP_FULL,
            hp_full: global::HP_FULL,
            mp: global::MP_START,
            mp_full: global::MP_FULL,
            fall: 0.0,
            bdefend: 0.0,
            arest: 0,
            vrest: HashMap::new(),
            frame: FrameState::default(),
            sp: SpriteInstance { sheets, ..Default::default() },
            mech: Mech::new(None),
            effect: EffectState { num: -99, ..Default::default() },
            holding_uid: None,
            hold_type: String::new(),
            held_by: None,
            dead: false,
            removed: false,
            ai: false,
            controller_index: None,
            allow_switch_dir: true,
            name,
            counter_disappear: -1,
            counter_dead_blink: -1,
            statemem_attlock: 0,
            statemem_frame_tu: false,
            injury_total: 0.0,
            kills: 0,
            enter_frame_applied: false,
            opoint_spawned: false,
            frame_sound: String::new(),
        };
        lo.trans_frame(0, 0);
        lo
    }

    pub fn frame_data(&self) -> Option<&FrameData> {
        self.data.frames.get(&self.frame.n)
    }

    pub fn state(&self) -> i32 {
        self.frame_data().map(|f| f.state).unwrap_or(0)
    }

    pub fn dirh(&self) -> i32 { self.facing }

    /// Transition to frame with lock priority (F.LF trans.frame)
    pub fn trans_frame(&mut self, mut frame: i32, lock: i32) -> bool {
        if lock < self.frame.lock {
            return false;
        }
        if frame == 999 {
            frame = 0;
        }
        if frame < 0 {
            return false;
        }
        let Some(fd) = self.data.frames.get(&frame).cloned() else {
            return false;
        };
        let prev = self.frame.n;
        self.frame.pn = prev;
        self.frame.n = frame;
        self.frame.wait_left = fd.wait;
        self.frame.lock = lock;
        self.sp.pic = fd.pic;
        self.enter_frame_applied = false;
        self.opoint_spawned = false;
        self.statemem_frame_tu = true;
        self.frame_sound = fd.sound.clone();
        // apply frame force once on entry (dvx/dvy/dvz)
        self.apply_frame_force(&fd);
        self.enter_frame_applied = true;
        true
    }

    /// F.LF livingobject.frame_force — exact rules
    pub fn frame_force(&mut self) {
        let Some(fd) = self.frame_data().cloned() else { return };
        self.apply_frame_force(&fd);
    }

    fn apply_frame_force(&mut self, fd: &FrameData) {
        if fd.dvx != 0.0 && (fd.dvx as i32) != global::UNSPECIFIED {
            if fd.dvx == 550.0 {
                self.ps.vx = 0.0;
            } else {
                let avx = self.ps.vx.abs();
                // accelerate ok if airborne or slower than target
                if self.ps.y < 0.0 || avx < fd.dvx {
                    self.ps.vx = self.facing as f64 * fd.dvx;
                }
                // decelerate gradual when dvx negative in data (LF2 uses negative dvx as friction cue)
                if fd.dvx < 0.0 {
                    self.ps.vx -= self.facing as f64;
                }
            }
        }
        if fd.dvz != 0.0 && (fd.dvz as i32) != global::UNSPECIFIED {
            if fd.dvz == 550.0 {
                self.ps.vz = 0.0;
            } else {
                // dirv applied by caller for characters; weapons use facing only on z rarely
                self.ps.vz = fd.dvz; // character sets via dirv * dvz in state code
            }
        }
        if fd.dvy != 0.0 && (fd.dvy as i32) != global::UNSPECIFIED {
            if fd.dvy == 550.0 {
                self.ps.vy = 0.0;
            } else {
                self.ps.vy += fd.dvy;
            }
        }
    }

    pub fn set_mass(&mut self, mass: f64) {
        self.mech.mass = mass;
    }

    pub fn take_sound(&mut self) -> Option<String> {
        if self.frame_sound.is_empty() {
            None
        } else {
            let s = self.frame_sound.clone();
            self.frame_sound.clear();
            Some(s)
        }
    }

    pub fn try_hit_tag(&mut self, tag: &str) -> bool {
        let Some(fd) = self.frame_data().cloned() else { return false };
        let next = frame_hit_tag(&fd, tag);
        if next == 0 || next == global::UNSPECIFIED {
            return false;
        }
        // mp cost on specials
        if fd.mp != 0 && !matches!(tag, "hit_a" | "hit_d" | "hit_j") {
            let dmp = (fd.mp % 1000).abs();
            if self.mp < dmp as f64 {
                return false;
            }
            self.mp -= dmp as f64;
        }
        self.trans_frame(next, 10)
    }

    pub fn injure(&mut self, injury: f64, fall_add: f64, dvx: f64, dvy: f64, attacker_facing: i32) {
        self.hp -= injury;
        self.injury_total += injury;
        self.fall += if fall_add != 0.0 { fall_add } else { global::DEFAULT_FALL };
        self.ps.vx = dvx;
        if dvy != 0.0 {
            self.ps.vy = dvy;
        } else {
            self.ps.vy = global::DEFAULT_FALL_DVY;
        }
        if self.hp < 0.0 {
            self.hp = 0.0;
        }
        if self.hp <= 0.0 {
            self.trans_frame(180, 20); // fall
            if !self.data.frames.contains_key(&180) {
                self.trans_frame(181, 20);
            }
        } else if self.fall >= global::FALL_KO {
            self.trans_frame(180, 15);
        } else if self.bdefend > 0.0 && self.state() == 7 {
            // defended — minor
            self.trans_frame(111, 10);
        } else {
            // injury frames 220-229
            let inj = if self.data.frames.contains_key(&220) { 220 } else { 0 };
            if inj != 0 {
                self.trans_frame(inj, 12);
            }
        }
        let _ = attacker_facing;
        self.effect.blink = true;
        self.effect.timeout = 8;
    }

    pub fn recover_tu(&mut self) {
        if self.fall > 0.0 {
            self.fall += global::RECOVER_FALL;
            if self.fall < 0.0 {
                self.fall = 0.0;
            }
        }
        if self.bdefend > 0.0 {
            self.bdefend += global::RECOVER_BDEFEND;
            if self.bdefend < 0.0 {
                self.bdefend = 0.0;
            }
        }
        if self.arest > 0 {
            self.arest -= 1;
        }
        let keys: Vec<u32> = self.vrest.keys().cloned().collect();
        for k in keys {
            if let Some(v) = self.vrest.get_mut(&k) {
                *v -= 1;
                if *v <= 0 {
                    self.vrest.remove(&k);
                }
            }
        }
        // slow mp regen
        if self.mp < self.mp_full {
            self.mp = (self.mp + 1.0 / 3.0).min(self.mp_full);
        }
        // effect timeout
        if self.effect.timeout > 0 {
            self.effect.timeout -= 1;
            if self.effect.timeout == 0 {
                self.effect.blink = false;
                self.effect.stuck = false;
            }
        }
        if self.statemem_attlock > 0 {
            self.statemem_attlock -= 1;
        }
    }

    /// Physics + frame wait (base TU without character input)
    pub fn physics_tu(&mut self, zbound: (f64, f64), bg_width: f64) {
        if self.removed || self.effect.stuck {
            return;
        }
        self.recover_tu();
        if !self.effect.stuck {
            self.frame_force();
        }

        // gravity when airborne
        if self.ps.y < 0.0 || self.ps.vy < 0.0 {
            self.ps.vy += global::GRAVITY;
        }

        // integrate — LF2: y negative is up
        self.ps.x += self.ps.vx;
        self.ps.z += self.ps.vz;
        self.ps.y += self.ps.vy; // vy negative => y decreases (higher)

        // land
        if self.ps.y >= 0.0 && self.ps.vy >= 0.0 {
            let was_air = self.frame.pn != 0 || self.ps.vy != 0.0;
            self.ps.y = 0.0;
            if self.ps.vy > 0.0 {
                // fell onto ground friction
                let fr = global::friction_fell(self.ps.vx.abs());
                if self.ps.vx.abs() > fr {
                    self.ps.vx -= self.ps.vx.signum() * fr;
                } else {
                    self.ps.vx = 0.0;
                }
                let frz = global::friction_fell(self.ps.vz.abs());
                if self.ps.vz.abs() > frz {
                    self.ps.vz -= self.ps.vz.signum() * frz;
                } else {
                    self.ps.vz = 0.0;
                }
                self.ps.vy = 0.0;
                // landing frames
                let st = self.state();
                if st == 4 || self.frame.n == 212 {
                    self.trans_frame(215, 15); // crouch
                } else if was_air && (st == 12 || (self.frame.n >= 180 && self.frame.n < 190)) {
                    // fall land — bounce if fast
                    let spd = (self.ps.vx * self.ps.vx + self.ps.vy * self.ps.vy).sqrt();
                    if spd > 13.4 || self.ps.vy.abs() > 11.0 {
                        // bounce up
                        let absorb = global::bounce_absorb(self.ps.vx);
                        if self.ps.vx.abs() > absorb {
                            self.ps.vx -= self.ps.vx.signum() * absorb;
                        } else {
                            self.ps.vx = 0.0;
                        }
                        self.ps.vy = -4.25; // bounceup.y
                        self.trans_frame(182, 10);
                    } else if self.data.frames.contains_key(&219) {
                        self.trans_frame(219, 15); // lying
                    } else if self.data.frames.contains_key(&185) {
                        self.trans_frame(185, 15);
                    }
                    self.frame_sound = "1/016".into(); // fall sound
                } else if was_air && matches!(st, 5 | 3) {
                    self.trans_frame(215, 15);
                }
            }
        }

        // bounds
        let (z0, z1) = zbound;
        if self.ps.z < z0 { self.ps.z = z0; }
        if self.ps.z > z1 { self.ps.z = z1; }
        if self.ps.x < 40.0 { self.ps.x = 40.0; }
        if self.ps.x > bg_width - 40.0 { self.ps.x = bg_width - 40.0; }

        // ground friction for standing
        if self.ps.y >= 0.0 && matches!(self.state(), 0 | 1 | 7 | 14 | 15) {
            self.ps.vx *= 0.0;
            self.ps.vz *= 0.0;
        }

        // wait / next
        if self.frame.wait_left > 0 {
            self.frame.wait_left -= 1;
        } else {
            let next = self.frame_data().map(|f| f.next).unwrap_or(0);
            if next == 1000 {
                self.removed = true;
                self.dead = true;
            } else if next == 999 {
                self.trans_frame(0, 0);
                self.frame.lock = 0;
            } else if next >= 0 {
                let lock = self.frame.lock;
                self.trans_frame(next, lock);
                // reduce lock after natural transition
                if self.frame.lock > 0 {
                    self.frame.lock = (self.frame.lock - 1).max(0);
                }
            }
        }

        // sync sprite world pos (render uses x,z and y lift)
        self.sp.x = self.ps.x;
        self.sp.y = self.ps.y;
        self.sp.z = self.ps.z;
        self.sp.facing = self.facing;
        self.sp.mirror = self.facing < 0;
        if let Some(fd) = self.frame_data() {
            self.sp.pic = fd.pic;
        }
        // blink
        if self.effect.blink && (self.effect.timeout / 2) % 2 == 0 {
            self.sp.visible = false;
        } else {
            self.sp.visible = !self.removed;
        }
        self.ps.zz = self.ps.z;
    }
}
