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
    /// Prefer targets chased less often (F.LF chasing.chased scores)
    pub chased_counts: std::collections::HashMap<u32, i32>,
    /// State 1002: no bounce when parent was grounded at spawn
    pub nobounce: bool,
    pub parent_uid: Option<u32>,
}

impl SpecialAttack {
    pub fn new(uid: u32, data: ObjectData, team: i32, x: f64, y: f64, z: f64, facing: i32) -> Self {
        let mut s = Self {
            base: LivingObject::new(uid, data, team, x, z),
            opoint_done: false,
            chase_x: f64::NAN,
            chase_z: f64::NAN,
            chased_counts: std::collections::HashMap::new(),
            nobounce: false,
            parent_uid: None,
        };
        s.base.ps.y = y;
        s.base.facing = facing;
        s.base.obj_type = "specialattack".into();
        s.base.hp = global::HP_FULL;
        // F.LF: mass 0 unless oid in specialattack_projectiles
        if !global::SPECIALATTACK_PROJECTILES.contains(&s.base.id) {
            s.base.mech.mass = 0.0;
        }
        s.base.trans_frame(0, 0);
        s
    }

    /// F.LF specialattack.init facing from opoint.facing codes
    pub fn apply_opoint_facing(&mut self, parent_facing: i32, opoint_facing: i32) {
        let mut face = opoint_facing;
        if face >= 20 {
            face %= 10;
        }
        let dir = if face == 0 {
            parent_facing
        } else if face == 1 {
            -parent_facing
        } else if (2..=10).contains(&face) {
            1
        } else if (11..=19).contains(&face) {
            -1
        } else {
            parent_facing
        };
        self.base.facing = if dir >= 0 { 1 } else { -1 };
    }

    pub fn with_parent(mut self, parent_uid: u32, parent_y: f64) -> Self {
        self.parent_uid = Some(parent_uid);
        // state 1002 nobounce if parent grounded
        self.nobounce = parent_y >= 0.0;
        self
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
            // dvx/dvy frame force — skip when chase hit_Fa active (300X owns vx)
            let chasing = fd.hit_Fa == 1 || fd.hit_Fa == 2 || fd.hit_Fa == 10;
            if !chasing && fd.state != 15 {
                if fd.dvx != 0.0 && fd.dvx as i32 != global::UNSPECIFIED {
                    self.base.ps.vx = fd.dvx * self.base.facing as f64;
                }
            }
            if fd.dvy != 0.0 && fd.dvy as i32 != global::UNSPECIFIED && !chasing {
                if self.base.ps.y < 0.0 {
                    self.base.ps.vy += fd.dvy * 0.05;
                }
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

    /// F.LF specialattack.attacked / killed / offset_attack — credit parent character uid
    pub fn credit_uid(&self) -> Option<u32> {
        self.parent_uid
    }
}
