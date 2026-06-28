//! Visual effects — blood, blast (LF/effect.js simplified set)
use crate::lf::data::ObjectData;
use crate::lf::livingobject::LivingObject;

pub struct EffectObj {
    pub base: LivingObject,
    pub subnum: i32,
}

impl EffectObj {
    pub fn new(uid: u32, data: ObjectData, x: f64, y: f64, z: f64) -> Self {
        let mut e = Self {
            base: LivingObject::new(uid, data, 0, x, z),
            subnum: 0,
        };
        e.base.ps.y = y;
        e.base.trans_frame(0, 0);
        e
    }

    pub fn with_frame(uid: u32, data: ObjectData, x: f64, y: f64, z: f64, frame: i32) -> Self {
        let mut e = Self::new(uid, data, x, y, z);
        if frame > 0 {
            e.base.trans_frame(frame, 0);
        }
        e
    }

    pub fn tu(&mut self, bg_z: (f64, f64), bg_w: f64) {
        self.base.physics_tu(bg_z, bg_w);
        if self.base.frame.n >= 1000 {
            self.base.removed = true;
            self.base.dead = true;
            return;
        }
        if let Some(fd) = self.base.frame_data().cloned() {
            if fd.next == 1000 && self.base.trans.wait == 0 {
                self.base.removed = true;
                self.base.dead = true;
            }
        }
        let _ = (bg_z, bg_w);
    }
}

/// Map effect.num from itr → object id (F.LF GC.effect.num_to_id = 300)
pub fn effect_id_from_num(num: i32) -> i32 {
    if num <= 0 {
        return 301; // default blood
    }
    300 + num // often 0->300 blast, 1->301 blood in LF2 extended
}
