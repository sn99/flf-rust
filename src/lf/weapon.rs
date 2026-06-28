//! Weapon — LF/weapon.js light + heavy
use crate::lf::data::ObjectData;
use crate::lf::global;
use crate::lf::livingobject::LivingObject;

pub struct Weapon {
    pub base: LivingObject,
    pub held: bool,
    pub light: bool,
    pub heavy: bool,
    pub holder_uid: Option<u32>,
}

impl Weapon {
    pub fn new(uid: u32, data: ObjectData, x: f64, z: f64) -> Self {
        let light = data.obj_type == "lightweapon" || data.obj_type == "drink";
        let heavy = data.obj_type == "heavyweapon";
        let mut w = Self {
            base: LivingObject::new(uid, data, 0, x, z),
            held: false,
            light,
            heavy,
            holder_uid: None,
        };
        // on ground frames: light 0 or 64?, heavy 0
        w.base.trans_frame(0, 0);
        w
    }

    pub fn tu(&mut self, bg_z: (f64, f64), bg_w: f64) {
        if self.held {
            return; // position synced by holder
        }
        // weapon TU before physics for bounce detect
        let y_before = self.base.ps.y;
        let vy_before = self.base.ps.vy;
        self.base.physics_tu(bg_z, bg_w);

        // fell onto ground (was airborne, now y==0 and vy was positive/down in LF2 y increases down? we use y negative up so vy>0 falls)
        if self.base.ps.y >= 0.0 && vy_before > 0.0 && y_before < 0.0 {
            let speed = (self.base.ps.vx * self.base.ps.vx
                + self.base.ps.vy * self.base.ps.vy
                + self.base.ps.vz * self.base.ps.vz)
                .sqrt();
            let limit = 8.0; // GC.weapon.bounceup.limit
            if speed > limit {
                if self.light {
                    self.base.ps.vy = 0.0;
                    self.base.trans_frame(70, 5);
                }
                if self.heavy {
                    self.base.ps.vy = -3.7; // bounceup.speed.y
                }
                if self.base.ps.vx != 0.0 {
                    self.base.ps.vx = self.base.ps.vx.signum() * 3.0;
                }
                if self.base.ps.vz != 0.0 {
                    self.base.ps.vz = self.base.ps.vz.signum() * 1.5;
                }
            } else {
                self.base.team = 0;
                self.base.ps.vy = 0.0;
                if self.light {
                    self.base.trans_frame(70, 5);
                }
                if self.heavy {
                    self.base.trans_frame(21, 5);
                }
            }
        }
        let _ = global::GRAVITY;
    }

    pub fn attach_to(&mut self, holder: u32, x: f64, y: f64, z: f64, facing: i32) {
        self.held = true;
        self.holder_uid = Some(holder);
        self.base.ps.x = x;
        self.base.ps.y = y;
        self.base.ps.z = z;
        self.base.facing = facing;
        self.base.ps.vx = 0.0;
        self.base.ps.vy = 0.0;
        self.base.ps.vz = 0.0;
    }

    pub fn drop(&mut self, vx: f64, vy: f64, vz: f64) {
        self.held = false;
        self.holder_uid = None;
        self.base.team = 0;
        self.base.ps.vx = vx;
        self.base.ps.vy = vy;
        self.base.ps.vz = vz;
        if self.light {
            self.base.trans_frame(40, 5); // in air
        } else {
            self.base.trans_frame(1, 5);
        }
    }
}
