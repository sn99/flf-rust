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
    /// F.LF hold.pre — holder that receives attacked/killed credit
    pub hold_pre_uid: Option<u32>,
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
            hold_pre_uid: None,
        };
        w.base.hp = w.base.data.bmp.weapon_hp.max(1.0);
        w.base.hp_full = w.base.hp;
        w.base.hp_bound = w.base.hp;
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

        // just landed — F.LF weapon.js generic TU (bounceup / settle)
        if self.base.ps.y >= 0.0 && vy_before > 0.0 && y_before < 0.0 {
            let speed = (self.base.ps.vx * self.base.ps.vx
                + vy_before * vy_before
                + self.base.ps.vz * self.base.ps.vz)
                .sqrt();
            if speed > global::WEAPON_BOUNCEUP_LIMIT {
                if self.light {
                    self.base.ps.vy = 0.0;
                    self.base.trans_frame(70, 5);
                }
                if self.heavy {
                    self.base.ps.vy = global::WEAPON_BOUNCEUP_SPEED_Y;
                }
                if self.base.ps.vx != 0.0 {
                    self.base.ps.vx =
                        self.base.ps.vx.signum() * global::WEAPON_BOUNCEUP_SPEED_X;
                }
                if self.base.ps.vz != 0.0 {
                    self.base.ps.vz =
                        self.base.ps.vz.signum() * global::WEAPON_BOUNCEUP_SPEED_Z;
                }
                let hurt = self.base.data.bmp.weapon_drop_hurt;
                if hurt > 0.0 {
                    self.base.hp -= hurt;
                    if self.base.hp <= 0.0 {
                        self.die();
                    }
                }
            } else {
                self.base.team = 0;
                self.hold_pre_uid = None;
                self.base.ps.vy = 0.0;
                if self.light {
                    self.base.trans_frame(70, 5);
                }
                if self.heavy {
                    self.base.trans_frame(21, 5);
                }
            }
            self.base.ps.zz = 0.0;
        }

        // in-air weapon frames often 20-60 range; on ground 0/64/70
        if self.base.ps.y < 0.0 && self.light && self.base.frame.n < 20 {
            // keep flying frame if data has it
        }
    }

    pub fn attach_to(&mut self, holder: u32, x: f64, y: f64, z: f64, facing: i32) {
        self.held = true;
        self.holder_uid = Some(holder);
        self.hold_pre_uid = Some(holder);
        self.base.ps.x = x;
        self.base.ps.y = y;
        self.base.ps.z = z;
        self.base.facing = facing;
        self.base.ps.vx = 0.0;
        self.base.ps.vy = 0.0;
        self.base.ps.vz = 0.0;
        // LF held weapon pseudo-states
        let held_frame = if self.heavy { 2001 } else { 1001 };
        if self.base.data.frames.contains_key(&held_frame) {
            self.base.trans_frame(held_frame, 0);
        }
    }

    pub fn drop(&mut self, vx: f64, vy: f64, vz: f64) {
        self.held = false;
        self.holder_uid = None;
        // hold_pre kept until land so thrown credit works (cleared on land)
        if self.light {
            self.base.trans_frame(40, 5);
        } else {
            self.base.trans_frame(1, 5);
        }
        // keep team so thrown weapon damages opponents (cleared on land in tu)
        self.base.ps.vx = vx;
        self.base.ps.vy = vy;
        self.base.ps.vz = vz;
    }

    pub fn die(&mut self) {
        self.base.trans_frame(1000, 20);
        self.base.removed = true;
    }

    /// F.LF typeweapon.attacked — forward to holder pre when present; returns credit uid
    pub fn attacked_credit_uid(&self) -> Option<u32> {
        self.hold_pre_uid.or(self.holder_uid)
    }

    /// F.LF typeweapon.itr_rest_update override — heavier weapons keep arest longer
    pub fn itr_rest_update(&mut self, arest: i32, vrest_uid: u32, vrest: i32) {
        let a = if self.heavy {
            arest.max(global::DEFAULT_AREST + 2)
        } else if arest > 0 {
            arest
        } else {
            global::DEFAULT_AREST
        };
        if a > self.base.arest {
            self.base.arest = a;
        }
        if vrest > 0 {
            self.base.itr_vrest_update(vrest_uid, vrest);
        }
    }

    /// F.LF typeweapon.pick
    pub fn pick(&mut self, holder: u32) {
        self.held = true;
        self.holder_uid = Some(holder);
        self.hold_pre_uid = Some(holder);
        let held_frame = if self.heavy { 2001 } else { 1001 };
        if self.base.data.frames.contains_key(&held_frame) {
            self.base.trans_frame(held_frame, 0);
        }
    }
}

impl Weapon {
    /// F.LF typeweapon.hit — reverse on head-on throw, bounce on ground
    /// `att_team` adopted on airborne reverse (F.LF).
    pub fn on_hit_by(
        &mut self,
        att_facing: i32,
        att_vx: f64,
        att_vz: f64,
        fall: f64,
        injury: f64,
        att_team: i32,
        att_is_weapon: bool,
    ) -> bool {
        if self.held {
            return false;
        }
        let st = self.base.state();
        let mut accept = false;
        if self.light {
            if st == 1002 {
                accept = true;
                // head-on reverse
                if (att_facing > 0) != (self.base.ps.vx > 0.0) {
                    self.base.ps.vx *= global::WEAPON_REVERSE_VX;
                }
                self.base.ps.vy *= global::WEAPON_REVERSE_VY;
                self.base.ps.vz *= global::WEAPON_REVERSE_VZ;
                self.base.team = att_team;
            } else if st == 1004 {
                // on_ground — only weapons kick it
                if att_is_weapon {
                    accept = true;
                    self.base.ps.vx = if att_vx != 0.0 {
                        att_vx.signum() * global::WEAPON_BOUNCEUP_SPEED_X
                    } else {
                        0.0
                    };
                    self.base.ps.vz = if att_vz != 0.0 {
                        att_vz.signum() * global::WEAPON_BOUNCEUP_SPEED_Z
                    } else {
                        0.0
                    };
                }
            }
        }
        if self.heavy {
            if st == 2004 {
                accept = true;
                if fall < 30.0 {
                    self.base.effect_create(0, global::EFFECT_DURATION, 0.0, 0.0);
                } else if fall < global::FALL_KO {
                    self.base.ps.vy = global::WEAPON_SOFT_BOUNCEUP_SPEED_Y;
                } else {
                    self.base.ps.vy = global::WEAPON_BOUNCEUP_SPEED_Y;
                    if att_vx != 0.0 {
                        self.base.ps.vx = att_vx.signum() * global::WEAPON_BOUNCEUP_SPEED_X;
                    }
                    if att_vz != 0.0 {
                        self.base.ps.vz = att_vz.signum() * global::WEAPON_BOUNCEUP_SPEED_Z;
                    }
                    self.base.trans_frame(999, 5);
                }
            } else if st == 2000 && fall >= global::FALL_KO {
                accept = true;
                if (att_facing > 0) != (self.base.ps.vx > 0.0) {
                    self.base.ps.vx *= global::WEAPON_REVERSE_VX;
                }
                self.base.ps.vy *= global::WEAPON_REVERSE_VY;
                self.base.ps.vz *= global::WEAPON_REVERSE_VZ;
                self.base.team = att_team;
            }
        }
        if accept && injury > 0.0 {
            self.base.hp -= injury;
            if self.base.hp <= 0.0 {
                self.die();
            }
        }
        accept
    }

    /// After weapon damages a character — F.LF: vx = sign(travel) * GC.weapon.hit.vx (−3)
    pub fn after_hit_character(&mut self) {
        if self.light {
            let sign = if self.base.ps.vx == 0.0 {
                0.0
            } else if self.base.ps.vx > 0.0 {
                1.0
            } else {
                -1.0
            };
            self.base.ps.vx = sign * global::WEAPON_HIT_VX;
            self.base.ps.vy = global::WEAPON_HIT_VY;
            // F.LF weapon.js: effect_stuck(0, timeout) — timers decay via recover_tu
            self.base.effect_stuck(0, 2);
        } else if self.heavy {
            self.base.effect_stuck(0, 4);
        }
    }

    /// F.LF typeweapon.act — held weaponact + throw impulse from wpoint kind 2
    /// Returns true if thrown this frame
    pub fn act(
        &mut self,
        holder_uid: u32,
        holder_x: f64,
        holder_y: f64,
        holder_z: f64,
        holder_facing: i32,
        weaponact: i32,
        attacking: i32,
        cover: i32,
        dvx: f64,
        dvy: f64,
        dvz: f64,
        wpoint_kind: i32,
    ) -> bool {
        self.held = true;
        self.holder_uid = Some(holder_uid);
        self.base.team = 0; // set by caller usually
        if weaponact > 0 && self.base.data.frames.contains_key(&weaponact) {
            self.base.trans_frame(weaponact, 2);
        }
        let mut thrown = false;
        if wpoint_kind == 2 {
            if dvx != 0.0 {
                self.base.ps.vx = holder_facing as f64 * dvx;
            }
            if dvz != 0.0 {
                self.base.ps.vz = dvz; // dirv * dvz approximated as signed vz from data
            }
            if dvy != 0.0 {
                self.base.ps.vy = dvy;
            }
            if self.base.ps.vx != 0.0 || self.base.ps.vy != 0.0 || self.base.ps.vz != 0.0 {
                let (imx, imy) = if self.light { (58.0, -15.0) } else { (48.0, -40.0) };
                self.base.ps.x = holder_x + holder_facing as f64 * imx;
                self.base.ps.y = holder_y + imy;
                self.base.ps.z = holder_z + self.base.ps.vz;
                self.base.ps.zz = 1.0;
                if self.light {
                    self.base.trans_frame(40, 5);
                } else {
                    self.base.trans_frame(20, 5);
                }
                self.held = false;
                self.holder_uid = None;
                thrown = true;
            }
        }
        if !thrown {
            self.base.ps.zz = if cover == 1 { -1.0 } else { 1.0 };
            self.base.facing = holder_facing;
            self.base.ps.z = holder_z;
            // position applied by attach_to from wpoint world
            let _ = attacking;
        }
        thrown
    }

    pub fn strength_itr(&self, attacking: i32) -> Option<&crate::lf::data::ItrData> {
        self.base.data.weapon_strength_list.get(&attacking)
    }
}
