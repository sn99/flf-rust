use crate::lf::data::ObjectData;
use crate::lf::livingobject::LivingObject;

pub struct Weapon {
    pub base: LivingObject,
    pub held: bool,
}

impl Weapon {
    pub fn new(uid: u32, data: ObjectData, x: f64, z: f64) -> Self {
        let mut w = Self {
            base: LivingObject::new(uid, data, 0, x, z),
            held: false,
        };
        w.base.trans_frame(0, 0);
        w
    }

    pub fn tu(&mut self, bg_z: (f64, f64), bg_w: f64) {
        if self.held {
            return;
        }
        self.base.physics_tu(bg_z, bg_w);
    }
}
