//! Special attack / projectiles — LF/specialattack.js (expanded)
use crate::lf::data::ObjectData;
use crate::lf::global;
use crate::lf::livingobject::LivingObject;

pub struct SpecialAttack {
    pub base: LivingObject,
    /// opoint spawned this frame
    pub opoint_done: bool,
    /// Chase target world position (F.LF chase_target)
    pub chase_x: f64,
    pub chase_z: f64,
}

impl SpecialAttack {
    pub fn new(uid: u32, data: ObjectData, team: i32, x: f64, y: f64, z: f64, facing: i32) -> Self {
        let mut s = Self {
            base: LivingObject::new(uid, data, team, x, z),
            opoint_done: false,
            chase_x: f64::NAN,
            chase_z: f64::NAN,
        };
        s.base.ps.y = y;
        s.base.facing = facing;
        s.base.obj_type = "specialattack".into();
        s.base.hp = global::HP_FULL;
        s.base.trans_frame(0, 0);
        s
    }

    pub fn with_velocity(mut self, dvx: f64, dvy: f64) -> Self {
        if dvx != 0.0 && dvx as i32 != global::UNSPECIFIED {
            self.base.ps.vx = dvx * self.base.facing as f64;
        }
        if dvy != 0.0 && dvy as i32 != global::UNSPECIFIED {
            self.base.ps.vy = dvy;
        }
        self
    }

    /// Effect num on active itr (2 fire, 3 ice) for ball clash logic
    pub fn itr_effect(&self) -> i32 {
        self.base
            .frame_data()
            .and_then(|f| f.itr.first().map(|i| i.effect))
            .unwrap_or(0)
    }

    pub fn is_ice_ball(&self) -> bool {
        let e = self.itr_effect();
        e == 3 || self.base.state() == 3000
    }

    pub fn is_fire_ball(&self) -> bool {
        let e = self.itr_effect();
        e == 2 || e == 20 || e == 21
    }

    pub fn tu(&mut self, bg_z: (f64, f64), bg_w: f64) {
        // reset opoint flag on frame change
        if self.base.frame.pn != self.base.frame.n && self.base.enter_frame_applied {
            // wait — opoint_spawned on LO handles char; we use opoint_done
        }
        if self.base.frame.pn != self.base.statemem_attlock {
            // reuse attlock as frame stamp noop
        }
        let frame_changed = self.base.frame.pn != self.base.frame.n;
        if frame_changed {
            self.opoint_done = false;
        }

        if let Some(fd) = self.base.frame_data().cloned() {
            if fd.hit_a != 0 {
                self.base.hp -= fd.hit_a as f64;
                if self.base.hp <= 0.0 {
                    let die = if fd.hit_d != 0 { fd.hit_d } else { 1000 };
                    self.base.trans_frame(die, 5);
                }
            }
            if fd.hit_j != 0 {
                self.base.ps.vz = (fd.hit_j - 50) as f64;
            }
            if fd.dvx != 0.0 && fd.dvx as i32 != global::UNSPECIFIED {
                self.base.ps.vx = fd.dvx * self.base.facing as f64;
            }
            if fd.dvy != 0.0 && fd.dvy as i32 != global::UNSPECIFIED {
                // some balls use dvy as lift; only apply lightly in air
                if self.base.ps.y < 0.0 {
                    self.base.ps.vy += fd.dvy * 0.05;
                }
            }
            // state 15 whirlwind — slow horizontal drift only
            if fd.state == 15 {
                self.base.ps.vx *= 0.92;
            }
        }
        // land dissolve for many light projectiles
        if self.base.frame.n == 15 && self.base.ps.y >= 0.0 {
            self.base.trans_frame(1000, 5);
        }
        if self.base.ps.x < -200.0 || self.base.ps.x > bg_w + 200.0 {
            self.base.trans_frame(1000, 5);
        }
        if self.base.frame.n >= 1000 || self.base.hp <= 0.0 && self.base.frame.n != 0 {
            self.base.removed = self.base.frame.n >= 1000;
        }
        crate::lf::special_states::dispatch(self, "frame");
        crate::lf::special_states::dispatch(self, "TU");
        self.base.physics_tu(bg_z, bg_w);
    }

    /// Shatter into ice debris oid 209-style — match spawns effect
    pub fn mark_die(&mut self, frame: i32) {
        self.base.trans_frame(if frame > 0 { frame } else { 1000 }, 5);
        self.base.hp = 0.0;
    }
}
