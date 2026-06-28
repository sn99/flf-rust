use crate::lf::data::ObjectData;
use crate::lf::livingobject::LivingObject;

pub struct EffectObj {
    pub base: LivingObject,
}

impl EffectObj {
    pub fn new(uid: u32, data: ObjectData, x: f64, y: f64, z: f64) -> Self {
        let mut e = Self { base: LivingObject::new(uid, data, 0, x, z) };
        e.base.ps.y = y;
        e
    }

    pub fn tu(&mut self, bg_z: (f64, f64), bg_w: f64) {
        self.base.physics_tu(bg_z, bg_w);
    }
}
