use crate::lf::data::ObjectData;
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
        s.base.trans_frame(0, 0);
        s
    }

    pub fn tu(&mut self, bg_z: (f64, f64), bg_w: f64) {
        self.base.physics_tu(bg_z, bg_w);
    }
}
