//! Special attack / projectiles — LF/specialattack.js
use crate::lf::data::ObjectData;
use crate::lf::global;
use crate::lf::livingobject::LivingObject;

pub struct SpecialAttack {
    pub base: LivingObject,
}

impl SpecialAttack {
    pub fn new(uid: u32, data: ObjectData, team: i32, x: f64, y: f64, z: f64, facing: i32) -> Self {
        let mut s = Self {
            base: LivingObject::new(uid, data, team, x, z),
        };
        s.base.ps.y = y;
        s.base.facing = facing;
        s.base.hp = global::HP_FULL;
        s.base.trans_frame(0, 0);
        s
    }

    pub fn tu(&mut self, bg_z: (f64, f64), bg_w: f64) {
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
        }
        if self.base.frame.n == 15 && self.base.ps.y >= 0.0 {
            self.base.trans_frame(1000, 5);
        }
        if self.base.ps.x < -200.0 || self.base.ps.x > bg_w + 200.0 {
            self.base.trans_frame(1000, 5);
        }
        self.base.physics_tu(bg_z, bg_w);
    }
}
