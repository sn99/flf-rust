//! Specialattack.js state handlers — whirlwind, ice/fire, shield, chase balls
use crate::lf::global;
use crate::lf::specialattack::SpecialAttack;

pub fn dispatch(sp: &mut SpecialAttack, event: &str) {
    let st = sp.base.state();
    // generic 300X chase runs for all specials on TU (F.LF always calls states['300X'])
    if event == "TU" {
        chase_300x(sp);
    }
    match (event, st) {
        ("TU", 15) => {
            // whirlwind — vx = dirh * dvx
            if let Some(fd) = sp.base.frame_data() {
                let dvx = fd.dvx;
                if dvx != 0.0 && dvx as i32 != global::UNSPECIFIED {
                    sp.base.ps.vx = sp.base.facing as f64 * dvx;
                }
            }
        }
        ("TU", 1002) => {
            // projectile land / bounce (F.LF specialattack state 1002)
            if sp.base.ps.y >= 0.0 && sp.base.ps.vy > 0.0 {
                if sp.nobounce {
                    sp.base.trans_frame(1000, 5);
                } else {
                    let speed = (sp.base.ps.vx * sp.base.ps.vx
                        + sp.base.ps.vy * sp.base.ps.vy
                        + sp.base.ps.vz * sp.base.ps.vz)
                        .sqrt();
                    if speed > global::WEAPON_BOUNCEUP_LIMIT {
                        sp.base.trans_frame(10, 5);
                        sp.base.ps.vy = global::WEAPON_BOUNCEUP_SPEED_Y;
                        if sp.base.ps.vx != 0.0 {
                            sp.base.ps.vx =
                                sp.base.ps.vx.signum() * global::WEAPON_BOUNCEUP_SPEED_X;
                        }
                        if sp.base.ps.vz != 0.0 {
                            sp.base.ps.vz =
                                sp.base.ps.vz.signum() * global::WEAPON_BOUNCEUP_SPEED_Z;
                        }
                    }
                }
            }
        }
        ("frame", 1002) => {}
        ("hit_others", 1002) => {
            sp.base.ps.vx = 0.0;
            sp.base.trans_frame(10, 5);
        }
        ("frame", _) => {
            if let Some(fd) = sp.base.frame_data().cloned() {
                if fd.hit_a != 0 {
                    // hit_a on frame entry handled in SpecialAttack::tu for TU;
                    // F.LF applies on TU only for specials
                }
            }
        }
        _ => {}
    }
}

/// F.LF states['300X'] chase — hit_Fa 1/2 seek, hit_Fa 10 exhaust speed
fn chase_300x(sp: &mut SpecialAttack) {
    let Some(fd) = sp.base.frame_data().cloned() else {
        return;
    };
    let hit_fa = fd.hit_Fa;
    if hit_fa == 1 || hit_fa == 2 {
        if sp.base.hp > 0.0 && sp.chase_x.is_finite() && sp.chase_z.is_finite() {
            let dx = sp.chase_x - sp.base.ps.x;
            let dz = sp.chase_z - sp.base.ps.z;
            let sx = if dx >= 0.0 { 1.0 } else { -1.0 };
            let sz = if dz >= 0.0 { 1.0 } else { -1.0 };
            if sp.base.ps.vx * sx < global::CHASE_MAX_VX {
                sp.base.ps.vx += sx * global::CHASE_AX;
            }
            if sp.base.ps.vz * sz < global::CHASE_MAX_VZ {
                sp.base.ps.vz += sz * global::CHASE_AZ;
            }
            sp.base.facing = if sp.base.ps.vx >= 0.0 { 1 } else { -1 };
        }
    } else if hit_fa == 10 {
        let sx = if sp.base.ps.vx > 0.0 {
            1.0
        } else if sp.base.ps.vx < 0.0 {
            -1.0
        } else {
            sp.base.facing as f64
        };
        sp.base.ps.vx = sx * global::CHASE_EXHAUST_VX;
        sp.base.ps.vz = 0.0;
    }
}
