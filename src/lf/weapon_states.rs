//! Weapon state handlers from LF/weapon.js (light/heavy shared patterns)
use crate::lf::weapon::Weapon;

pub fn dispatch(w: &mut Weapon, event: &str) {
    let st = w.base.state();
    match event {
        "TU" => match st {
            1001 | 2001 => {} // held passive
            _ => {}
        },
        "frame" => {
            // on-ground idle oscillate sometimes
            if w.base.ps.y >= 0.0 && !w.held && w.light {
                if w.base.frame.n == 0 || w.base.frame.n == 64 {
                    // stay
                }
            }
        }
        "die" => {
            w.die();
        }
        _ => {}
    }
}
