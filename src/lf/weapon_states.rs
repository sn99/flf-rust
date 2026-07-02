//! Weapon state handlers from LF/weapon.js (light/heavy)
use crate::lf::weapon::Weapon;

pub fn dispatch(w: &mut Weapon, event: &str) {
    let st = w.base.state();
    match event {
        "TU" => {
            match st {
                1001 | 2001 => {
                    // held passive — position set by holder wpoint in match
                }
                1000 | 2000 => {
                    // on ground / in sky idle friction
                    if w.base.ps.y >= 0.0 && !w.held {
                        w.base.ps.vx *= 0.9;
                        w.base.ps.vz *= 0.9;
                    }
                }
                _ => {}
            }
            if w.base.hp <= 0.0 && !w.held {
                w.die();
            }
        }
        "frame" => {
            // F.LF states 1003 / 1004 / 2000 / 2004 frame events
            let n = w.base.frame.n;
            if st == 1003 || w.light {
                // just_on_ground 70 — drop sound optional (match plays)
                let _ = n;
            }
            if st == 1004 && n == 64 {
                w.base.team = 0; // on ground loses team
            }
            if st == 2000 && n == 21 {
                w.base.trans.set_next(20, 5);
            }
            if st == 2004 && n == 20 {
                w.base.team = 0;
            }
            // team loss frames on settle
            if matches!(n, 70 | 21 | 20 | 64) && !w.held {
                w.base.team = 0;
            }
        }
        "die" => {
            w.die();
        }
        _ => {}
    }
}
