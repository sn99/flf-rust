//! Per-character id_update from LF/character.js (full event matrix)
use crate::lf::character::Character;
use crate::lf::global;

/// Return true if event was fully handled (caller should skip default).
pub fn id_update(ch: &mut Character, event: &str, tag: Option<&str>) -> bool {
    let id = ch.base.id;
    let n = ch.base.frame.n;
    let pn = ch.base.frame.pn;
    match event {
        "generic_combo" => {
            if id == 1 {
                // Deep: face combo direction on hit_Fj
                if tag == Some("hit_Fj") {
                    // facing already set from input; ensure facing matches last horizontal intent
                    return false;
                }
            }
            if id == 6 {
                // Louis: disable transform hit_ja
                if tag == Some("hit_ja") {
                    return true; // block
                }
            }
            false
        }
        "state3_frame" => match id {
            1 if n == 267 => {
                ch.base.ps.vy += 1.0;
                true
            }
            5 if (273..=276).contains(&n) => {
                ch.base.ps.vy = -6.8;
                true
            }
            _ => false,
        },
        "state3_fly_crash" => {
            if id == 10 {
                // Woody
                ch.base.trans.set_wait(0, 10, 1);
                return true;
            }
            false
        }
        "state3_hit_stop" => {
            if id == 11 {
                // Davis many_punch
                match n {
                    271 | 276 | 280 => {
                        ch.base.effect.stuck = false;
                        ch.base.effect.timein = 1;
                        ch.base.effect.timeout = 2;
                        ch.base.trans.inc_wait(1, 10, 1);
                        return true;
                    }
                    273 => {
                        ch.base.effect.stuck = true;
                        ch.base.effect.timeout = 2;
                        return true;
                    }
                    _ => {}
                }
            }
            false
        }
        "state3_frame_force" => {
            if id == 11 && matches!(n, 275 | 278 | 279) {
                return true; // disable pre force
            }
            if id == 1 && n != 290 {
                // deep state 301 uses different path; state3 generic no-op
            }
            false
        }
        "state15_crouch" => {
            if id == 1 && (267..=272).contains(&pn) {
                ch.base.trans.inc_wait(-1, 10, 1);
                return true;
            }
            false
        }
        "state1280_disappear" => {
            if id == 5 && n == 257 {
                ch.base.sp.visible = false;
                ch.base.effect.super_armor = true;
                ch.base.counter_disappear = 0;
                return true;
            }
            false
        }
        "rudolf_transform" => {
            if id == 5 || id == 0 {
                if let Some(uid) = ch.base.holding_uid {
                    ch.transform_target_uid = Some(uid);
                    ch.transform_target_id = ch.transform_caught_id;
                    ch.is_rudolf_transform = true;
                    ch.base.effect.blink = true;
                    ch.pending_transform = true;
                    return true;
                }
                if ch.transform_target_id != 0 {
                    ch.is_rudolf_transform = true;
                    ch.pending_transform = true;
                    return true;
                }
            }
            false
        }
        "revert_transform" => {
            if ch.is_rudolf_transform {
                ch.is_rudolf_transform = false;
                ch.pending_revert_transform = true;
                return true;
            }
            false
        }
        _ => false,
    }
}

/// Per-TU character-specific physics/flags (called every TU after state TU).
pub fn id_tu(ch: &mut Character) {
    let id = ch.base.id;
    let n = ch.base.frame.n;
    let pn = ch.base.frame.pn;
    let st = ch.base.state();

    // disappear blink sequence (Rudolf / generic)
    if ch.base.counter_disappear >= 0 {
        ch.base.counter_disappear += 1;
        let c = ch.base.counter_disappear;
        // GC.effect.disappear typically shadow_blink ~ body_blink windows
        const SHADOW_BLINK: i32 = 8;
        const BODY_BLINK: i32 = 16;
        if c < SHADOW_BLINK {
            ch.base.sp.visible = false;
        } else if c < BODY_BLINK {
            ch.base.sp.visible = c % 2 == 0;
            ch.base.effect.blink = true;
        } else if c == BODY_BLINK {
            ch.base.sp.visible = true;
            ch.base.effect.blink = true;
        } else {
            // dismiss
            ch.base.counter_disappear = -1;
            ch.base.sp.visible = true;
            ch.base.effect.blink = false;
        }
    }

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
                ch.base.ps.vy -= 0.5;
            }
            if st == 301 {
                let spd_z = ch.base.data.bmp.walking_speedz;
                ch.base.ps.vz = ch.dirv() * spd_z;
            }
        }
        2 => {
            // John
            if (240..280).contains(&n) {
                ch.base.effect.super_armor = true;
            }
            if st == 1700 {
                ch.base.effect.super_armor = true;
                if ch.base.hp < ch.base.hp_full {
                    ch.base.hp = (ch.base.hp + global::HP_FULL * 0.004).min(ch.base.hp_full);
                }
            }
        }
        4 => {
            // Henry — arrows via opoint
            if (240..260).contains(&n) && ch.base.mp > 0.0 {
                // slight aim assist: no-op physics
            }
        }
        5 => {
            // Rudolf
            if st == 3 && (273..=276).contains(&n) {
                ch.base.ps.vy = -6.8;
            }
            if n == 240 && ch.base.holding_uid.is_some() {
                ch.base.effect.blink = true;
                let _ = id_update(ch, "rudolf_transform", None);
            }
            if n == 298 && st == 501 {
                let _ = id_update(ch, "rudolf_transform", None);
            }
            if n == 257 {
                let _ = id_update(ch, "state1280_disappear", None);
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
                ch.base.ps.vx += 0.1 * ch.base.facing as f64;
                let rz = ch.base.data.bmp.running_speedz;
                ch.base.ps.vz = ch.dirv() * rz;
            }
            // burn trail frames
            if (240..255).contains(&n) {
                ch.base.effect.super_armor = true;
            }
        }
        8 => {
            // Freeze
            if st == 13 {
                ch.base.ps.vx *= 0.85;
                ch.base.ps.vz *= 0.85;
                ch.base.effect.stuck = ch.base.trans.wait > 2;
            }
            if (240..260).contains(&n) {
                // ice armor while casting
                ch.base.effect.super_armor = true;
            }
        }
        9 => {
            // Dennis
            if (240..270).contains(&n) {
                ch.base.effect.super_armor = (n % 3) != 0;
            }
        }
        10 => {
            // Woody
            if st == 19 {
                ch.base.ps.vy -= 0.3;
            }
            if n == 253 {
                ch.base.trans.set_wait(0, 10, 1);
            }
        }
        11 => {
            // Davis energy blast frames
            if matches!(n, 240 | 245 | 250 | 271 | 276 | 280) {
                // visual blink during many punch
                ch.base.effect.blink = n % 2 == 0;
            }
        }
        30..=39 => {
            // bandits / generic NPCs — no specials
        }
        50..=60 => {
            // bosses / extended — slight armor on high frames
            if n >= 240 && n < 300 {
                ch.base.effect.super_armor = true;
            }
        }
        _ => {}
    }

    // State-generic hooks mirrored from character.js states
    match st {
        9 if n == 240 => {
            let _ = id_update(ch, "rudolf_transform", None);
        }
        13 => {
            // leaving ice will spawn broken effect — set flag for match
            if ch.base.trans.wait == 0 && ch.base.frame.wait_left == 0 {
                ch.pending_broken_effect = 212;
            }
        }
        14 => {
            // lying: clear fall/bdefend on entry handled elsewhere
        }
        18 | 19 if id != 7 => {
            // non-firen in burn states still get slight drift
            ch.base.ps.vx += 0.05 * ch.base.facing as f64;
        }
        301 => {
            let spd_z = ch.base.data.bmp.walking_speedz;
            ch.base.ps.vz = ch.dirv() * spd_z;
        }
        1700 => {
            // heal aura
            ch.base.effect.super_armor = true;
        }
        _ => {}
    }

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
            ch.base.sp.visible = true;
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

/// Hit-stop override for state 3 (returns true if character handled it).
pub fn state3_hit_stop(ch: &mut Character) -> bool {
    id_update(ch, "state3_hit_stop", None)
}

/// Frame force disable for state 3.
pub fn state3_frame_force_block(ch: &mut Character) -> bool {
    id_update(ch, "state3_frame_force", None)
}
