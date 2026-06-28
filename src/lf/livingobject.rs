//! Base living object (characters, weapons, special attacks)
use crate::core_engine::animator::Animator;
use crate::core_engine::sprite::SpriteInstance;
use crate::lf::data::{frame_hit_tag, FrameData, ObjectData};
use crate::lf::global;
use crate::lf::mechanics::{self, Mech};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Default)]
pub struct Pos {
    pub x: f64,
    pub y: f64, // height above ground
    pub z: f64, // depth
}

#[derive(Clone, Debug)]
pub struct EffectState {
    pub super_armor: bool,
    pub freeze: i32,
    pub blind: i32,
}

impl Default for EffectState {
    fn default() -> Self {
        Self { super_armor: false, freeze: 0, blind: 0 }
    }
}

pub struct LivingObject {
    pub id: i32,
    pub uid: u32,
    pub team: i32,
    pub obj_type: String,
    pub data: ObjectData,
    pub ps: Pos,
    pub vx: f64,
    pub vy: f64,
    pub vz: f64,
    pub facing: i32, // 1 right, -1 left
    pub hp: f64,
    pub hp_full: f64,
    pub mp: f64,
    pub mp_full: f64,
    pub fall: f64,
    pub bdefend: f64,
    pub arest: i32,
    pub vrest: HashMap<u32, i32>,
    pub frame_id: i32,
    pub animator: Animator,
    pub sp: SpriteInstance,
    pub mech: Mech,
    pub effect: EffectState,
    pub holding_uid: Option<u32>,
    pub held_by: Option<u32>,
    pub dead: bool,
    pub ai: bool,
    pub controller_index: Option<usize>,
    pub blink: i32,
    pub name: String,
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
            ps: Pos { x, y: 0.0, z },
            vx: 0.0,
            vy: 0.0,
            vz: 0.0,
            facing: 1,
            hp: global::HP_FULL,
            hp_full: global::HP_FULL,
            mp: global::MP_START,
            mp_full: global::MP_FULL,
            fall: 0.0,
            bdefend: 0.0,
            arest: 0,
            vrest: HashMap::new(),
            frame_id: 0,
            animator: Animator::default(),
            sp: SpriteInstance { sheets, ..Default::default() },
            mech: Mech::new(None),
            effect: EffectState::default(),
            holding_uid: None,
            held_by: None,
            dead: false,
            ai: false,
            controller_index: None,
            blink: 0,
            name,
        };
        lo.transit(0);
        lo
    }

    pub fn frame(&self) -> Option<&FrameData> {
        self.data.frames.get(&self.frame_id)
    }

    pub fn state(&self) -> i32 {
        self.frame().map(|f| f.state).unwrap_or(0)
    }

    pub fn transit(&mut self, mut frame: i32) {
        if frame == 999 {
            frame = 0;
        }
        if frame < 0 { return; }
        if let Some(fd) = self.data.frames.get(&frame).cloned() {
            self.frame_id = frame;
            self.animator.set_frame(frame, fd.wait);
            self.sp.pic = fd.pic;
            // dvx: 0 = no change in some states; unspecified sentinel
            if fd.dvx != 0.0 && fd.dvx as i32 != global::UNSPECIFIED {
                // absolute set when specified in many frames; F.LF uses complex rules
                // simplified: treat as velocity impulse along facing for non-zero
            }
            if !fd.sound.is_empty() {
                // sound played by match
            }
        }
    }

    pub fn switch_frame(&mut self, frame: i32) {
        self.transit(frame);
    }

    pub fn dirh(&self) -> i32 { self.facing }

    pub fn try_hit(&mut self, tag: &str) -> bool {
        if let Some(fd) = self.frame().cloned() {
            let next = frame_hit_tag(&fd, tag);
            if next != 0 && next as i32 != global::UNSPECIFIED {
                // mp cost
                if fd.mp > 0 && tag != "hit_a" && tag != "hit_d" && tag != "hit_j" {
                    if self.mp < fd.mp as f64 { return false; }
                    self.mp -= fd.mp as f64;
                }
                self.transit(next);
                return true;
            }
        }
        false
    }

    pub fn injure(&mut self, injury: f64, fall_add: f64, dvx: f64, dvy: f64) {
        self.hp -= injury;
        self.fall += fall_add;
        self.vx += dvx;
        self.vy += dvy;
        if self.hp <= 0.0 {
            self.hp = 0.0;
            self.transit(181); // falling / KO start often 180-212 range; 181 lying transition
            if !self.data.frames.contains_key(&181) {
                self.transit(180);
            }
        } else if self.fall >= global::FALL_KO {
            self.transit(180);
        } else {
            // injury frames 220+
            let inj = 220;
            if self.data.frames.contains_key(&inj) {
                self.transit(inj);
            }
        }
    }

    pub fn recover_tu(&mut self) {
        if self.fall > 0.0 {
            self.fall += global::RECOVER_FALL;
            if self.fall < 0.0 { self.fall = 0.0; }
        }
        if self.bdefend > 0.0 {
            self.bdefend += global::RECOVER_BDEFEND;
            if self.bdefend < 0.0 { self.bdefend = 0.0; }
        }
        if self.arest > 0 { self.arest -= 1; }
        let keys: Vec<u32> = self.vrest.keys().cloned().collect();
        for k in keys {
            if let Some(v) = self.vrest.get_mut(&k) {
                *v -= 1;
                if *v <= 0 { self.vrest.remove(&k); }
            }
        }
        // mp regen slow
        if self.mp < self.mp_full {
            self.mp = (self.mp + 0.2).min(self.mp_full);
        }
        if self.blink > 0 { self.blink -= 1; }
    }

    /// Core TU for generic living object
    pub fn tu_base(&mut self, bg_zwidth: (f64, f64), bg_width: f64) {
        if self.dead { return; }
        self.recover_tu();

        let on_ground = self.ps.y <= 0.0 && self.vy >= 0.0;
        if on_ground {
            self.ps.y = 0.0;
            if self.vy > 0.0 { self.vy = 0.0; }
        } else {
            mechanics::apply_gravity_vy(&mut self.vy, false);
        }

        // Apply frame dvx as walking speed in standing/walking handled by character
        let fd = self.frame().cloned();
        if let Some(ref fd) = fd {
            if fd.dvx != 0.0 && (fd.dvx as i32) != global::UNSPECIFIED {
                // In LF2, dvx on frame often means set horizontal velocity when entering
            }
            if fd.dvy != 0.0 && (fd.dvy as i32) != global::UNSPECIFIED {
                // applied on frame enter typically
            }
        }

        mechanics::integrate(&mut self.ps, self.vx, self.vy, self.vz);

        // bounds
        let (zmin, zmax) = bg_zwidth;
        if self.ps.z < zmin { self.ps.z = zmin; }
        if self.ps.z > zmax { self.ps.z = zmax; }
        if self.ps.x < 0.0 { self.ps.x = 0.0; }
        if self.ps.x > bg_width { self.ps.x = bg_width; }

        // ground friction light
        if self.ps.y <= 0.0 {
            self.vx *= 0.9;
            self.vz *= 0.9;
            if self.vx.abs() < global::MIN_SPEED { self.vx = 0.0; }
            if self.vz.abs() < global::MIN_SPEED { self.vz = 0.0; }
        }

        // animator wait
        if self.animator.tu() {
            if let Some(fd) = self.frame().cloned() {
                let mut next = fd.next;
                if next == 999 { next = 0; }
                if next == 1000 {
                    self.dead = true;
                } else if next >= 0 {
                    self.transit(next);
                }
            }
        }

        // sync sprite
        self.sp.x = self.ps.x;
        self.sp.y = self.ps.y;
        self.sp.z = self.ps.z;
        self.sp.facing = self.facing;
        self.sp.mirror = self.facing < 0;
        if let Some(fd) = self.frame() {
            self.sp.pic = fd.pic;
        }
        if self.blink > 0 && (self.blink / 2) % 2 == 0 {
            self.sp.visible = false;
        } else {
            self.sp.visible = true;
        }
    }
}
