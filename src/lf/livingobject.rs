//! Living object — faithful subset of LF/livingobject.js
use crate::core_engine::sprite::SpriteInstance;
use crate::lf::data::{frame_hit_tag, FrameData, ObjectData};
use crate::lf::global;
use crate::lf::mechanics::Mech;
use crate::lf::transistor::FrameTransistor;
use std::collections::HashMap;
use serde_json::Value;

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
    pub trans: FrameTransistor,
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
    /// per-id properties from package (LF2 properties.js)
    pub properties: Value,
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
            trans: FrameTransistor::default(),
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
            properties: Value::Null,
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
        // Use transistor authority
        self.trans.frame(frame, lock);
        // Apply immediately if wait was set to 0
        if self.trans.wait == 0 {
            return self.apply_transit();
        }
        // Also set next for when wait expires from set_wait with wait>0 via frame()
        // frame() sets wait 0 so apply now:
        self.apply_transit()
    }

    /// Apply pending transistor next frame (enter frame)
    pub fn apply_transit(&mut self) -> bool {
        let mut frame = self.trans.next;
        if self.trans.switch_dir_after {
            self.facing = -self.facing;
            self.trans.switch_dir_after = false;
        }
        if frame == 999 {
            frame = 0;
        }
        if frame == 1000 {
            self.removed = true;
            self.dead = true;
            return true;
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
        self.frame.lock = self.trans.lock;
        self.trans.wait = fd.wait;
        self.trans.next = if fd.next == 999 { 0 } else { fd.next };
        self.sp.pic = fd.pic;
        self.enter_frame_applied = false;
        self.opoint_spawned = false;
        self.statemem_frame_tu = true;
        self.frame_sound = fd.sound.clone();
        self.apply_frame_force(&fd);
        self.enter_frame_applied = true;
        // natural lock decay setup
        self.trans.lockout = fd.wait.max(1);
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

    /// LF livingobject.proper(id?, prop)
    pub fn proper(&self, prop: &str) -> Option<Value> {
        let id = self.id.to_string();
        self.properties
            .get(&id)
            .and_then(|o| o.get(prop))
            .cloned()
    }

    pub fn proper_bool(&self, prop: &str) -> bool {
        self.proper(prop)
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
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

    /// Faithful subset of character.js `hit` fall() / falldown() / posteffect()
    /// Returns (drop_weapon, effect_sound_hint) for match to act on.
    pub fn injure(
        &mut self,
        injury: f64,
        fall_add: f64,
        dvx: f64,
        dvy: f64,
        attacker_x: f64,
        effect_num: i32,
        itr_kind: i32,
    ) -> (bool, i32) {
        if injury < 0.0 {
            self.hp = (self.hp - injury).min(self.hp_full);
            return (false, -1);
        }
        if self.effect.super_armor && itr_kind != 8 {
            self.hp = (self.hp - injury * 0.35).max(0.0);
            self.injury_total += injury * 0.35;
            return (false, effect_num);
        }

        let mut inj = injury;
        // flute force kinds 10/11 double injury while falling
        if (itr_kind == 10 || itr_kind == 11) && self.state() == 12 {
            inj *= 2.0;
        }

        self.hp -= inj;
        self.injury_total += inj;
        if self.hp < 0.0 {
            self.hp = 0.0;
        }

        // effect-driven outcomes (posteffect) take priority over generic fall
        let mut drop_w = false;
        match effect_num {
            2 | 20 | 21 | 22 | 23 => {
                // fire
                drop_w = true;
                self.ps.vx = dvx;
                if dvy != 0.0 {
                    self.ps.vy = dvy;
                }
                self.trans_frame(203, 36);
                self.effect.blink = true;
                self.effect.timeout = 20;
                return (drop_w, effect_num);
            }
            3 | 30 => {
                // ice
                drop_w = true;
                self.ps.vx = dvx;
                if self.state() != 13 {
                    self.trans_frame(200, 38);
                } else {
                    self.trans_frame(182, 21);
                }
                self.effect.stuck = true;
                self.effect.timeout = 40;
                return (drop_w, effect_num);
            }
            4 => {
                drop_w = true;
            }
            _ => {}
        }

        // itr kind 16 on victim → ice frame 200
        if itr_kind == 16 {
            self.trans_frame(200, 38);
            self.ps.vx = dvx;
            return (true, 3);
        }

        self.fall += if fall_add != 0.0 {
            fall_add
        } else {
            global::DEFAULT_FALL
        };
        self.ps.vx = dvx;
        let ef_dvy = if dvy != 0.0 {
            dvy
        } else {
            global::DEFAULT_FALL_DVY
        };

        let fall = self.fall;
        let airborne = self.ps.y < 0.0 || self.ps.vy < 0.0;
        let do_falldown = self.state() == 13
            || airborne
            || self.hp <= 0.0
            || fall > global::FALL_KO;

        if do_falldown {
            self.fall = 0.0;
            self.ps.vy = ef_dvy;
            let front = (attacker_x > self.ps.x) == (self.facing > 0);
            if front {
                self.trans_frame(180, 21);
            } else {
                self.trans_frame(186, 21);
            }
            drop_w = true;
        } else if self.bdefend > 0.0 && self.state() == 7 {
            self.trans_frame(111, 10);
        } else if fall > 0.0 && fall <= 20.0 {
            self.trans_frame(220, 20);
        } else if fall > 20.0 && fall <= 30.0 {
            self.trans_frame(222, 20);
        } else if fall > 30.0 && fall <= 40.0 {
            self.trans_frame(224, 20);
        } else if fall > 40.0 && fall <= 60.0 {
            self.trans_frame(226, 20);
        } else {
            // light injury / dance of pain entry
            if self.data.frames.contains_key(&220) {
                self.trans_frame(220, 12);
            }
        }

        // fall frames that drop weapon in F.LF posteffect 0/1
        if matches!(self.frame.n, 180 | 186) || do_falldown {
            drop_w = true;
        }

        self.effect.blink = true;
        self.effect.timeout = 8;
        (drop_w, effect_num)
    }

    pub fn itr_vrest_test(&self, att_uid: u32) -> bool {
        !self.vrest.contains_key(&att_uid)
    }

    pub fn itr_vrest_update(&mut self, att_uid: u32, vrest: i32) {
        let v = if vrest > 0 {
            vrest
        } else {
            global::DEFAULT_VREST
        };
        self.vrest.insert(att_uid, v);
    }

    pub fn itr_arest_update(&mut self, arest: i32) {
        let a = if arest > 0 {
            arest
        } else {
            global::DEFAULT_AREST
        };
        if a > self.arest {
            self.arest = a;
        }
    }

    /// LF livingobject.whirlwind_force — centripetal toward volume center
    pub fn whirlwind_force(&mut self, cx: f64, cz: f64) {
        let mass = self.mech.mass.max(1.0);
        self.ps.vy -= 2.0 / mass;
        let sx = if self.ps.x - cx > 0.0 { 1.0 } else { -1.0 };
        let sz = if self.ps.z - cz > 0.0 { 1.0 } else { -1.0 };
        self.ps.vx -= sx * 2.0 / mass;
        self.ps.vz -= sz * 0.5 / mass;
    }

    /// LF livingobject.flute_force — hover bands + super armor
    pub fn flute_force(&mut self) {
        let mass = self.mech.mass.max(1.0);
        let low_level = -140.0;
        let mid_level = -160.0;
        let high_level = -180.0;
        self.effect.super_armor = true;
        self.ps.vx = 0.0;
        self.ps.vz = 0.0;
        if self.ps.y > low_level {
            self.ps.vy = if self.ps.vy <= 0.0 {
                -7.5
            } else {
                -self.ps.vy / 2.0
            };
        } else if self.ps.y <= low_level && self.ps.y > mid_level {
            self.ps.vy -= mass / 2.0;
        } else if self.ps.y <= mid_level && self.ps.y > high_level {
            self.ps.vy += mass / 2.0;
        }
        self.effect.blink = true;
        self.effect.timeout = self.effect.timeout.max(8);
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
        // slow mp regen (LF2 ~1/3 per TU on ground; slower in air)
        if self.mp < self.mp_full {
            let rate = if self.ps.y >= 0.0 { 1.0 / 3.0 } else { 1.0 / 8.0 };
            self.mp = (self.mp + rate).min(self.mp_full);
        }
        // passive hp drip only while dead-blink not active and not removed
        if self.hp > 0.0 && self.hp < self.hp_full && self.counter_dead_blink < 0 && self.ps.y >= 0.0 {
            // extremely slow passive — disabled by default; heal state handles real regen
        }
        // effect timein: delay before stuck applies (Davis hit_stop pattern)
        if self.effect.timein > 0 {
            self.effect.timein -= 1;
            if self.effect.timein == 0 && self.effect.timeout > 0 {
                self.effect.stuck = true;
            }
        }
        if self.effect.timeout > 0 {
            self.effect.timeout -= 1;
            if self.effect.timeout == 0 {
                self.effect.blink = false;
                self.effect.stuck = false;
                self.effect.super_armor = false;
                self.effect.num = -99;
            }
        }
        if self.statemem_attlock > 0 {
            self.statemem_attlock -= 1;
        }
    }

    /// Physics + frame wait (base TU without character input)
    pub fn effect_stuck(&mut self, timein: i32, timeout: i32) {
        self.effect.stuck = true;
        self.effect.timein = timein;
        self.effect.timeout = timeout;
    }

    pub fn effect_create(&mut self, num: i32, duration: i32, dvx: f64, dvy: f64) {
        self.effect.num = num;
        self.effect.timeout = duration;
        self.effect.dvx = dvx;
        self.effect.dvy = dvy;
        if num == 0 {
            self.effect.super_armor = true;
        }
    }

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

        // wait / next via transistor (natural authority 0)
        self.frame.wait_left = self.trans.wait;
        if let Some(n) = self.trans.tick_wait() {
            // natural transition uses lock 0 unless locked
            if self.trans.lock == 0 || self.trans.lockout == 0 {
                self.trans.next = n;
                self.trans.lock = 0;
                self.apply_transit();
            } else {
                // still locked — use stored next from frame data
                let next = self.frame_data().map(|f| f.next).unwrap_or(0);
                self.trans.set_next(next, 0);
                if self.trans.wait == 0 {
                    self.apply_transit();
                }
            }
        } else {
            // sync wait_left
            self.frame.wait_left = self.trans.wait;
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
