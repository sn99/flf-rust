//! Per-character id_update from LF/character.js (expanded)
use crate::lf::character::Character;

pub fn id_tu(ch: &mut Character) {
    let id = ch.base.id;
    let n = ch.base.frame.n;
    let pn = ch.base.frame.pn;
    let st = ch.base.state();
    match id {
        1 => {
            // Deep
            if st == 3 && n == 267 {
                ch.base.ps.vy += 1.0;
            }
            if st == 15 && (267..=272).contains(&pn) {
                ch.base.trans.inc_wait(-1, 10, 1);
            }
            if n == 253 {
                // fly crash — slight lift
                ch.base.ps.vy -= 0.5;
            }
        }
        2 => {
            // John
            if (240..280).contains(&n) {
                ch.base.effect.super_armor = true;
            }
            // heal state often high frames
            if st == 1700 || (n >= 240 && n <= 250 && ch.base.mp > 0.0) {
                // passive regen while channeling special start
            }
        }
        4 => {
            // Henry — arrows via opoint in data
        }
        5 => {
            // Rudolf
            if st == 3 && (273..=276).contains(&n) {
                ch.base.ps.vy = -6.8;
            }
            // transform smoke at 240 while catching — flag only
            if n == 240 && ch.base.holding_uid.is_some() {
                ch.base.effect.blink = true;
            }
        }
        6 => {
            // Louis — armor frames
            if (240..260).contains(&n) {
                ch.base.effect.super_armor = true;
            }
        }
        7 => {
            // Firen
            if st == 18 || st == 19 {
                ch.base.effect.blink = (ch.base.trans.wait % 4) < 2;
                // slight forward drift
                ch.base.ps.vx += 0.1 * ch.base.facing as f64;
            }
        }
        8 => {
            // Freeze
            if st == 13 {
                ch.base.ps.vx *= 0.85;
                ch.base.ps.vz *= 0.85;
                ch.base.effect.stuck = ch.base.trans.wait > 2;
            }
        }
        9 => {
            // Dennis chase ball opoint
        }
        10 => {
            // Woody
            if st == 19 {
                ch.base.ps.vy -= 0.3;
            }
        }
        11 => {
            // Davis
            if n == 240 || n == 245 || n == 250 {
                // energy blast frames
            }
        }
        30 => {
            // Bandit — no specials
        }
        _ => {}
    }
    // Generic combo dir for hit_Fj on deep handled in input via facing
    let _ = (id, n, pn, st);
}

pub fn apply_teleport(
    ch: &mut Character,
    nearest_enemy: Option<(f64, f64)>,
    furthest_ally: Option<(f64, f64)>,
) {
    let st = ch.base.state();
    if st == 400 {
        if let Some((ex, ez)) = nearest_enemy {
            let dh = ch.base.facing as f64;
            ch.base.ps.x = ex - 120.0 * dh;
            ch.base.ps.y = 0.0;
            ch.base.ps.z = ez;
            ch.base.ps.vx = 0.0;
            ch.base.ps.vy = 0.0;
            ch.base.ps.vz = 0.0;
            ch.base.effect.super_armor = true;
            ch.base.effect.timeout = 15;
        }
    } else if st == 401 {
        if let Some((ax, az)) = furthest_ally {
            let dh = ch.base.facing as f64;
            ch.base.ps.x = ax + 60.0 * dh;
            ch.base.ps.y = 0.0;
            ch.base.ps.z = az;
            ch.base.ps.vx = 0.0;
            ch.base.ps.vy = 0.0;
            ch.base.ps.vz = 0.0;
            ch.base.effect.super_armor = true;
            ch.base.effect.timeout = 15;
        }
    }
}
