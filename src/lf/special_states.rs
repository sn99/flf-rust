//! Specialattack.js state handlers (whirlwind 15, ice 3000-ish, reflect 3006)
use crate::lf::specialattack::SpecialAttack;

pub fn dispatch(sp: &mut SpecialAttack, event: &str) {
    let st = sp.base.state();
    match (event, st) {
        ("TU", 15) => {
            // whirlwind — slow drift, pull handled in match whirlwind_itr
            sp.base.ps.vx *= 0.95;
        }
        ("frame", 3000) | ("frame", 3001) => {
            // ice column / freeze ball frames — data driven
        }
        ("hit", 3006) | ("TU", 3006) => {
            // john shield / reflect ball — high HP already via hit_a
        }
        ("frame", _) => {
            if let Some(fd) = sp.base.frame_data().cloned() {
                if fd.hit_a != 0 {
                    sp.base.hp -= fd.hit_a as f64;
                }
            }
        }
        _ => {}
    }
}
