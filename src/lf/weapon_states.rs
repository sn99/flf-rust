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
                    // on ground idle
                    if w.base.ps.y >= 0.0 && !w.held {
                        w.base.ps.vx *= 0.9;
                        w.base.ps.vz *= 0.9;
                    }
                }
                1002 | 1004 => {
                    // thrown / attacking light
                    if w.base.ps.y >= 0.0 && w.base.ps.vy >= 0.0 {
                        // land → bounce or settle
                        w.base.ps.vy = 0.0;
                        if w.base.ps.vx.abs() > 2.0 {
                            w.base.ps.vx *= -0.4;
                            w.base.trans_frame(if w.light { 70 } else { 20 }, 5);
                        } else {
                            w.base.trans_frame(if w.light { 0 } else { 20 }, 5);
                        }
                    }
                }
                2004 => {
                    // heavy thrown
                    if w.base.ps.y >= 0.0 {
                        w.base.ps.vy = 0.0;
                        w.base.ps.vx *= 0.5;
                        w.base.trans_frame(20, 5);
                    }
                }
                _ => {}
            }
            if w.base.hp <= 0.0 && !w.held {
                w.die();
            }
        }
        "frame" => {
            if w.base.ps.y >= 0.0 && !w.held && w.light {
                if w.base.frame.n == 0 || w.base.frame.n == 64 {
                    // idle oscillate possible via data next
                }
            }
            // team loss frames
            if matches!(w.base.frame.n, 70 | 21 | 20 | 64) && !w.held {
                w.base.team = 0;
            }
        }
        "die" => {
            w.die();
        }
        "hit" => {
            // light in air → 1004; heavy bounce
            if w.light && !w.held {
                if w.base.state() == 1002 {
                    w.base.trans_frame(1004, 8);
                }
            } else if !w.light && !w.held {
                w.base.ps.vx = -w.base.ps.vx * 0.6;
                w.base.trans_frame(2000, 8);
            }
        }
        _ => {}
    }
}
