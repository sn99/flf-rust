//! Specialattack.js state handlers — whirlwind, ice/fire, shield, chase balls
use crate::lf::specialattack::SpecialAttack;

pub fn dispatch(sp: &mut SpecialAttack, event: &str) {
    let st = sp.base.state();
    match (event, st) {
        ("TU", 15) => {
            // whirlwind — vx = dirh * dvx style drift
            if let Some(fd) = sp.base.frame_data() {
                let dvx = fd.dvx;
                if dvx != 0.0 && dvx as i32 != crate::lf::global::UNSPECIFIED {
                    sp.base.ps.vx = sp.base.facing as f64 * dvx.abs() * 0.5;
                } else {
                    sp.base.ps.vx *= 0.95;
                }
            } else {
                sp.base.ps.vx *= 0.95;
            }
        }
        ("frame", 3000) | ("frame", 3001) | ("TU", 3000) | ("TU", 3001) => {
            // ice column / freeze ball — data driven; light hp drip via hit_a in SpecialAttack::tu
        }
        ("hit", 3006) | ("TU", 3006) => {
            // john shield — high HP via hit_a; reflect handled in match
        }
        // Chase ball family 300X (Dennis chase etc.) — accelerate toward chase_target set by match
        ("TU", s) if (3002..=3010).contains(&s) || s == 1000 => {
            chase_step(sp);
        }
        ("frame", _) => {
            if let Some(fd) = sp.base.frame_data().cloned() {
                if fd.hit_a != 0 {
                    sp.base.hp -= fd.hit_a as f64;
                }
                // hit_Fa modes: 1/2/10 seek — apply mild tracking each frame entry
                if fd.hit_Fa != 0 {
                    chase_step(sp);
                }
            }
        }
        ("TU", 1002) => {
            // nobounce when parent grounded — approximate: damp vy on near ground
            if sp.base.ps.y > -5.0 && sp.base.ps.vy > 0.0 {
                sp.base.ps.vy *= 0.3;
            }
        }
        _ => {}
    }
}

fn chase_step(sp: &mut SpecialAttack) {
    // Use pending target stored in effect.dvx/dvy as x/z offsets if set by match;
    // otherwise gently keep vx toward facing.
    let tx = sp.chase_x;
    let tz = sp.chase_z;
    if tx.is_finite() && tz.is_finite() {
        let dx = tx - sp.base.ps.x;
        let dz = tz - sp.base.ps.z;
        let ax = 0.35_f64;
        let az = 0.25_f64;
        sp.base.ps.vx += dx.signum() * ax.min(dx.abs() * 0.08);
        sp.base.ps.vz += dz.signum() * az.min(dz.abs() * 0.08);
        // speed clamp
        let max_v = 14.0;
        if sp.base.ps.vx.abs() > max_v {
            sp.base.ps.vx = max_v * sp.base.ps.vx.signum();
        }
        if sp.base.ps.vz.abs() > max_v {
            sp.base.ps.vz = max_v * sp.base.ps.vz.signum();
        }
        if dx.abs() > 2.0 {
            sp.base.facing = if dx > 0.0 { 1 } else { -1 };
        }
    }
}
