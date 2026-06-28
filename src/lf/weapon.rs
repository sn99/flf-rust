//! Weapon — LF/weapon.js light + heavy generalization
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
        let light = matches!(data.obj_type.as_str(), "lightweapon" | "drink");
        let heavy = data.obj_type == "heavyweapon";
        let mut w = Self {
            base: LivingObject::new(uid, data, 0, x, z),
            held: false,
            light,
            heavy,
            holder_uid: None,
        };
        w.base.trans_frame(0, 0);
        w
    }

    pub fn tu(&mut self, bg_z: (f64, f64), bg_w: f64) {
        if self.held {
            return;
        }
        let y_before = self.base.ps.y;
        let vy_before = self.base.ps.vy;
        let st = self.base.state();
        // passive held-by-states 1001/2001 in LF2 — skip dynamics if ever set
        if st == 1001 || st == 2001 {
            return;
        }
        crate::lf::weapon_states::dispatch(self, "frame");
        crate::lf::weapon_states::dispatch(self, "TU");
        self.base.physics_tu(bg_z, bg_w);

        // just landed
        if self.base.ps.y >= 0.0 && vy_before > 0.0 && y_before < 0.0 {
            self.base.team = 0;
            let speed = (self.base.ps.vx * self.base.ps.vx
                + vy_before * vy_before
                + self.base.ps.vz * self.base.ps.vz)
                .sqrt();
            let limit = 8.0;
            if speed > limit {
                if self.light {
                    self.base.ps.vy = 0.0;
                    self.base.trans_frame(70, 5);
                }
                if self.heavy {
                    self.base.ps.vy = -3.7;
                    self.base.ps.vx = self.base.ps.vx.signum() * 3.0;
                    self.base.ps.vz = self.base.ps.vz.signum() * 1.5;
                }
                // weapon_drop_hurt if in bmp — skip optional field
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
            let _ = global::GRAVITY;
        }

        // in-air weapon frames often 20-60 range; on ground 0/64/70
        if self.base.ps.y < 0.0 && self.light && self.base.frame.n < 20 {
            // keep flying frame if data has it
        }
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
        self.base.team = 0; // will be set by holder team in match
    }

    pub fn drop(&mut self, vx: f64, vy: f64, vz: f64) {
        self.held = false;
        self.holder_uid = None;
        // keep team so thrown weapon damages opponents (cleared on land in tu)
        self.base.ps.vx = vx;
        self.base.ps.vy = vy;
        self.base.ps.vz = vz;
        if self.light {
            self.base.trans_frame(40, 5);
        } else {
            self.base.trans_frame(1, 5);
        }
    }

    pub fn die(&mut self) {
        self.base.trans_frame(1000, 20);
        self.base.removed = true;
    }
}
