//! Per-character id_update blocks from LF/character.js
use crate::lf::character::Character;

/// Called every TU after state logic; may mutate velocity/facing
pub fn id_tu(ch: &mut Character) {
    let id = ch.base.id;
    let n = ch.base.frame.n;
    let st = ch.base.state();
    match id {
        1 => {
            // Deep
            if st == 3 && n == 267 {
                ch.base.ps.vy += 1.0;
            }
            if st == 15 && (267..=272).contains(&ch.base.frame.pn) {
                ch.base.trans.inc_wait(-1, 10, 1);
            }
        }
        5 => {
            // Rudolf
            if st == 3 && (273..=276).contains(&n) {
                ch.base.ps.vy = -6.8;
            }
        }
        2 => {
            // John super frames
            if (240..280).contains(&n) {
                ch.base.effect.super_armor = true;
            }
        }
        6 => {
            // Louis
        }
        7 => {
            // Firen burning assist
            if st == 18 {
                ch.base.effect.blink = (ch.base.frame.wait_left % 4) < 2;
            }
        }
        8 => {
            // Freeze
            if st == 13 {
                ch.base.ps.vx *= 0.5;
                ch.base.ps.vz *= 0.5;
            }
        }
        10 => { /* Woody */ }
        11 => {
            // Davis
            if n == 240 || n == 245 {
                // opoint in data
            }
        }
        4 => { /* Henry */ }
        9 => { /* Dennis */ }
        30 => { /* Bandit */ }
        _ => {}
    }
}

/// Teleport states 400 / 401 — needs target positions (x,z) options
pub fn apply_teleport(ch: &mut Character, nearest_enemy: Option<(f64, f64)>, furthest_ally: Option<(f64, f64)>) {
    let st = ch.base.state();
    let n = ch.base.frame.n;
    // Apply on first frame of teleport sequence (frame event in JS)
    if !ch.base.statemem_frame_tu && n != 0 {
        // still allow if state is 400/401
    }
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
        }
    }
}
