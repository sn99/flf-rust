//! Complete port of LF/character.js `states` object — event dispatch per state.
//! Called from Character::tu / handle_input as F.LF does via states[N](event, K).

use crate::lf::character::Character;
use crate::lf::global;

/// Return value: Some(frame) for fall_onto_ground next frame; Some(1) as "consumed combo" sentinel via CONSUMED
pub const COMBO_CONSUMED: i32 = -1;

/// Dispatch character state event (F.LF states[state](event, K))
pub fn dispatch(ch: &mut Character, event: &str, key: Option<&str>) -> Option<i32> {
    // generic always first for frame/TU health/disappear
    if event == "frame" || event == "TU" {
        generic(ch, event);
    }
    let st = ch.base.state();
    match st {
        0 => state0(ch, event, key),
        1 => state1(ch, event, key),
        2 => state2(ch, event, key),
        3 => state3(ch, event, key),
        4 => state4(ch, event, key),
        5 => state5(ch, event, key),
        6 => state6(ch, event, key),
        7 => state7(ch, event, key),
        8 => state8(ch, event, key),
        9 => state9(ch, event, key),
        10 => state10(ch, event, key),
        11 => state11(ch, event, key),
        12 => state12(ch, event, key),
        13 => state13(ch, event, key),
        14 => state14(ch, event, key),
        15 => state15(ch, event, key),
        16 => state16(ch, event, key),
        18 => state18(ch, event, key),
        19 => state19(ch, event, key),
        301 => state301(ch, event, key),
        400 => state400(ch, event, key),
        401 => state401(ch, event, key),
        501 => state501(ch, event, key),
        1700 => state1700(ch, event, key),
        _ => None,
    }
}

fn generic(ch: &mut Character, event: &str) {
    match event {
        "frame" => {
            // mp on frame (F.LF generic frame)
            if let Some(fd) = ch.base.frame_data().cloned() {
                if fd.mp != 0 {
                    let pn_next = ch
                        .base
                        .data
                        .frames
                        .get(&ch.base.frame.pn)
                        .map(|f| f.next)
                        .unwrap_or(-1);
                    if pn_next == ch.base.frame.n {
                        // transit by next of previous
                        if fd.mp < 0 {
                            ch.base.mp = (ch.base.mp + fd.mp as f64).max(0.0);
                            if ch.base.mp <= 0.0 && fd.hit_d != 0 {
                                ch.base.trans_frame(fd.hit_d, 10);
                            }
                        }
                    } else {
                        let dmp = (fd.mp % 1000).abs() as f64;
                        let dhp = (fd.mp / 1000).abs() as f64 * 10.0;
                        ch.base.mp = (ch.base.mp - dmp).max(0.0);
                        if dhp > 0.0 {
                            ch.base.hp = (ch.base.hp - dhp).max(0.0);
                        }
                    }
                }
            }
            ch.base.opoint_spawned = false; // allow opoint this frame — match sets true after spawn
        }
        "TU" => {
            // disappear handled in character_ids; dead blink in character tu
        }
        _ => {}
    }
}

fn holding_heavy(ch: &Character) -> bool {
    ch.hold_weapon.is_some() && ch.base.hold_type == "heavyweapon"
}
fn holding_light(ch: &Character) -> bool {
    ch.hold_weapon.is_some()
        && (ch.base.hold_type == "lightweapon" || ch.base.hold_type == "drink")
}

/// standing
fn state0(ch: &mut Character, event: &str, key: Option<&str>) -> Option<i32> {
    match event {
        "frame" => None,
        "combo" => {
            let k = key?;
            match k {
                "left" | "right" => {
                    ch.switch_dir(k);
                    None
                }
                "left-left" | "right-right" => {
                    let dir = if k.starts_with("left") { "left" } else { "right" };
                    ch.switch_dir(dir);
                    if holding_heavy(ch) {
                        ch.base.trans_frame(16, 10);
                    } else {
                        ch.running = true;
                        ch.base.trans_frame(9, 10);
                    }
                    Some(COMBO_CONSUMED)
                }
                "def" => {
                    if holding_heavy(ch) {
                        return Some(COMBO_CONSUMED);
                    }
                    if !ch.base.try_hit_tag("hit_d") {
                        ch.base.trans_frame(110, 10);
                    }
                    Some(COMBO_CONSUMED)
                }
                "jump" => {
                    if holding_heavy(ch) && !ch.weapon_proper_bool("heavy_weapon_jump") {
                        // block unless property missing (= allow)
                        let blocked = ch
                            .base
                            .properties
                            .get(&ch.hold_weapon_oid.to_string())
                            .and_then(|o| o.get("heavy_weapon_jump"))
                            .and_then(|v| v.as_bool())
                            == Some(false);
                        if blocked {
                            return Some(COMBO_CONSUMED);
                        }
                    }
                    if !ch.base.try_hit_tag("hit_j") {
                        ch.base.trans_frame(210, 10);
                    }
                    Some(COMBO_CONSUMED)
                }
                "att" => {
                    if holding_heavy(ch) {
                        ch.base.trans_frame(50, 10);
                        return Some(COMBO_CONSUMED);
                    }
                    if holding_light(ch) {
                        if ch.weapon_proper_bool("just_throw") {
                            ch.base.trans_frame(45, 10);
                        } else if ch.weapon_proper_bool("attackable")
                            || !ch.weapon_proper_bool("stand_throw")
                        {
                            let fr = if js_sys::Math::random() < 0.5 { 20 } else { 25 };
                            ch.base.trans_frame(fr, 10);
                        } else {
                            ch.base.trans_frame(45, 10);
                        }
                        return Some(COMBO_CONSUMED);
                    }
                    let fr = if ch.want_super_punch {
                        ch.want_super_punch = false;
                        70
                    } else if js_sys::Math::random() < 0.5 {
                        60
                    } else {
                        65
                    };
                    if !ch.base.try_hit_tag("hit_a") {
                        ch.base.trans_frame(fr, 10);
                    }
                    Some(COMBO_CONSUMED)
                }
                _ => None,
            }
        }
        _ => None,
    }
}

/// walking
fn state1(ch: &mut Character, event: &str, key: Option<&str>) -> Option<i32> {
    match event {
        "frame" => {
            if holding_heavy(ch) {
                ch.base.frame_ani_oscillate(12, 15);
            } else {
                ch.base.frame_ani_oscillate(5, 8);
            }
            let rate = ch.base.data.bmp.walking_frame_rate.max(1);
            ch.base.trans.set_wait(rate - 1, 5, 1);
            None
        }
        "TU" => {
            let dv = ch.dirv();
            let xfactor = 1.0 - (if dv != 0.0 { 1.0 } else { 0.0 }) * (2.0 / 7.0);
            if holding_heavy(ch) {
                let hs = ch.base.data.bmp.heavy_walking_speed;
                let hsz = ch.base.data.bmp.heavy_walking_speedz;
                if ch.base.ps.vx.abs() > 0.01 {
                    ch.base.ps.vx = xfactor * ch.base.facing as f64 * hs;
                }
                ch.base.ps.vz = dv * hsz;
            } else {
                let wsz = ch.base.data.bmp.walking_speedz;
                ch.base.ps.vz = dv * wsz;
                // vx maintained by handle_input walking
                let _ = xfactor;
            }
            None
        }
        "state_entry" => {
            ch.base.trans.set_wait(0, 5, 1);
            None
        }
        "combo" => {
            if key.is_some() {
                return state0(ch, "combo", key);
            }
            None
        }
        _ => None,
    }
}

fn state2(ch: &mut Character, event: &str, key: Option<&str>) -> Option<i32> {
    match event {
        "frame" => {
            if holding_heavy(ch) {
                ch.base.frame_ani_oscillate(16, 18);
            } else {
                ch.base.frame_ani_oscillate(9, 11);
            }
            let rate = ch.base.data.bmp.running_frame_rate.max(1);
            ch.base.trans.set_wait(rate, 5, 1);
            // fall through TU velocity in F.LF — also apply here
            apply_run_velocity(ch);
            None
        }
        "TU" => {
            apply_run_velocity(ch);
            None
        }
        "combo" => {
            let k = key?;
            match k {
                "left" | "right" | "left-left" | "right-right" => {
                    let want = if k.contains("left") { -1 } else { 1 };
                    if want != ch.base.facing {
                        if holding_heavy(ch) {
                            ch.base.trans_frame(19, 10);
                        } else {
                            ch.base.trans_frame(218, 10);
                        }
                        ch.running = false;
                        return Some(COMBO_CONSUMED);
                    }
                    None
                }
                "def" => {
                    if holding_heavy(ch) {
                        return Some(COMBO_CONSUMED);
                    }
                    ch.base.trans_frame(102, 10);
                    Some(COMBO_CONSUMED)
                }
                "jump" => {
                    if holding_heavy(ch) {
                        let block = ch
                            .base
                            .properties
                            .get(&ch.hold_weapon_oid.to_string())
                            .and_then(|o| o.get("heavy_weapon_dash"))
                            .and_then(|v| v.as_bool())
                            == Some(false);
                        if block {
                            return Some(COMBO_CONSUMED);
                        }
                    }
                    ch.base.trans_frame(213, 10);
                    ch.running = false;
                    Some(COMBO_CONSUMED)
                }
                "att" => {
                    if holding_light(ch) {
                        if ch.weapon_proper_bool("run_throw") {
                            ch.base.trans_frame(45, 10);
                        } else {
                            ch.base.trans_frame(35, 10);
                        }
                    } else if holding_heavy(ch) {
                        ch.base.trans_frame(50, 10);
                    } else {
                        ch.base.trans_frame(85, 10);
                    }
                    ch.running = false;
                    Some(COMBO_CONSUMED)
                }
                _ => None,
            }
        }
        _ => None,
    }
}

fn apply_run_velocity(ch: &mut Character) {
    let dv = ch.dirv();
    let xfactor = 1.0 - (if dv != 0.0 { 1.0 } else { 0.0 }) * (1.0 / 7.0);
    if holding_heavy(ch) {
        let rs = ch.base.data.bmp.heavy_running_speed;
        let rsz = ch.base.data.bmp.heavy_running_speedz;
        ch.base.ps.vx = xfactor * ch.base.facing as f64 * rs;
        ch.base.ps.vz = dv * rsz;
    } else {
        let rs = ch.base.data.bmp.running_speed;
        let rsz = ch.base.data.bmp.running_speedz;
        ch.base.ps.vx = xfactor * ch.base.facing as f64 * rs;
        ch.base.ps.vz = dv * rsz;
    }
}

/// attack / punch states driven by frames
fn state3(ch: &mut Character, event: &str, _key: Option<&str>) -> Option<i32> {
    match event {
        "frame" => {
            let _ = crate::lf::character_ids::id_update(ch, "state3_frame", None);
            None
        }
        "TU" => None,
        "frame_force" => {
            if crate::lf::character_ids::state3_frame_force_block(ch) {
                Some(1)
            } else {
                None
            }
        }
        "hit_stop" => {
            if crate::lf::character_ids::state3_hit_stop(ch) {
                Some(1)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// jump
fn state4(ch: &mut Character, event: &str, key: Option<&str>) -> Option<i32> {
    match event {
        "state_entry" => None,
        "TU" => {
            if ch.base.frame.n == 212 && ch.base.frame.pn == 211 && ch.base.statemem_frame_tu {
                ch.base.statemem_frame_tu = false;
                let bmp = &ch.base.data.bmp;
                ch.base.ps.vy = bmp.jump_height;
                ch.base.ps.vz = ch.dirv() * (bmp.jump_distancez - 1.0);
            }
            None
        }
        "frame" => None,
        "combo" => {
            let k = key?;
            if k == "att" && ch.base.frame.n == 212 && ch.base.statemem_attlock == 0 {
                if holding_light(ch) {
                    ch.base.trans_frame(30, 10);
                } else {
                    ch.base.trans_frame(80, 10);
                }
                ch.base.statemem_attlock = 2;
                return Some(COMBO_CONSUMED);
            }
            None
        }
        _ => None,
    }
}

/// dash
fn state5(ch: &mut Character, event: &str, key: Option<&str>) -> Option<i32> {
    match event {
        "state_entry" => {
            let pn = ch.base.frame.pn;
            if (9..=11).contains(&pn) || pn == 215 {
                let sign = if ch.base.frame.n == 213 { 1.0 } else { -1.0 };
                let bmp = &ch.base.data.bmp;
                ch.base.ps.vx = ch.base.facing as f64 * (bmp.dash_distance - 1.0) * sign;
                ch.base.ps.vz = ch.dirv() * (bmp.dash_distancez - 1.0);
                ch.base.ps.vy = bmp.dash_height;
            }
            None
        }
        "combo" => {
            let k = key.unwrap_or("");
            if k == "att" {
                let front = ch.base.facing as f64 * ch.base.ps.vx >= 0.0;
                if front || ch.base.proper_bool("dash_backattack") {
                    if holding_light(ch) && ch.weapon_proper_bool("attackable") {
                        ch.base.trans_frame(40, 10);
                    } else {
                        ch.base.trans_frame(90, 10);
                    }
                    ch.base.allow_switch_dir = false;
                    return Some(COMBO_CONSUMED);
                }
            }
            if k == "left" || k == "right" {
                let want = if k == "left" { -1 } else { 1 };
                if want != ch.base.facing {
                    let front = ch.base.facing as f64 * ch.base.ps.vx >= 0.0;
                    if front {
                        if ch.base.frame.n == 213 {
                            ch.base.trans_frame(214, 0);
                        }
                        if ch.base.frame.n == 216 {
                            ch.base.trans_frame(217, 0);
                        }
                    } else {
                        if ch.base.frame.n == 214 {
                            ch.base.trans_frame(213, 0);
                        }
                        if ch.base.frame.n == 217 {
                            ch.base.trans_frame(216, 0);
                        }
                    }
                    ch.switch_dir(k);
                    return Some(COMBO_CONSUMED);
                }
            }
            None
        }
        _ => None,
    }
}

/// rowing
fn state6(ch: &mut Character, event: &str, _key: Option<&str>) -> Option<i32> {
    match event {
        "TU" => {
            if matches!(ch.base.frame.n, 100 | 108) {
                ch.base.ps.vy = 0.0;
            }
            None
        }
        "frame" => {
            if matches!(ch.base.frame.n, 100 | 108) {
                ch.base.trans.set_wait(1, 10, 1);
            }
            if ch.base.statemem_frame_tu {
                ch.base.statemem_frame_tu = false;
                let rd = ch.base.data.bmp.rowing_distance;
                let rh = ch.base.data.bmp.rowing_height;
                ch.base.ps.vx = ch.base.facing as f64 * rd;
                if rh != 0.0 {
                    ch.base.ps.vy = rh;
                }
            }
            None
        }
        "fall_onto_ground" => {
            if matches!(ch.base.frame.n, 101 | 109) {
                Some(215)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn state7(ch: &mut Character, event: &str, _key: Option<&str>) -> Option<i32> {
    if event == "frame" && ch.base.frame.n == 111 {
        ch.base.trans.inc_wait(4, 10, 1);
    }
    None
}

fn state8(ch: &mut Character, event: &str, _key: Option<&str>) -> Option<i32> {
    if event == "TU" {
        ch.base.bdefend = (ch.base.bdefend - 2.0).max(0.0);
    }
    None
}

fn state9(ch: &mut Character, event: &str, key: Option<&str>) -> Option<i32> {
    match event {
        "state_entry" => {
            ch.catch_counter = 43;
            ch.catch_attacks = 0;
            None
        }
        "state_exit" => {
            ch.base.ps.zz = 0.0;
            None
        }
        "frame" => {
            match ch.base.frame.n {
                123 => {
                    ch.catch_attacks += 1;
                    ch.catch_counter += 3;
                    ch.base.trans.inc_wait(1, 10, 1);
                }
                233 | 234 => ch.base.trans.inc_wait(-1, 10, 1),
                240 => {
                    let _ = crate::lf::character_ids::id_update(ch, "rudolf_transform", None);
                }
                _ => {}
            }
            None
        }
        "TU" => {
            ch.catch_counter -= 1;
            if ch.catch_counter <= 0
                && !(ch.base.frame.n == 122 && ch.catch_attacks == 4)
                && matches!(ch.base.frame.n, 121 | 122)
            {
                ch.base.holding_uid = None;
                ch.base.trans_frame(0, 15);
            }
            if let Some(fd) = ch.base.frame_data() {
                if let Some(cp) = &fd.cpoint {
                    let cover = if cp.cover != 0 { cp.cover } else { 0 };
                    ch.base.ps.zz = if cover == 0 || cover == 10 { 1.0 } else { -1.0 };
                }
            }
            None
        }
        "combo" => {
            let k = key?;
            if k == "att" {
                if let Some(fd) = ch.base.frame_data().cloned() {
                    if let Some(cp) = fd.cpoint {
                        if cp.taction != 0 {
                            let tac = cp.taction;
                            if tac < 0 {
                                let nd = if ch.base.facing > 0 { "left" } else { "right" };
                                ch.switch_dir(nd);
                                ch.base.trans_frame(-tac, 10);
                            } else {
                                ch.base.trans_frame(tac, 10);
                            }
                            ch.catch_counter += 10;
                        } else if cp.aaction != 0 {
                            ch.base.trans_frame(cp.aaction, 10);
                        } else {
                            ch.base.trans_frame(121, 12);
                        }
                    } else {
                        ch.base.trans_frame(121, 12);
                    }
                }
                return Some(COMBO_CONSUMED);
            }
            if k == "jump" && ch.base.frame.n == 121 {
                if let Some(fd) = ch.base.frame_data() {
                    if let Some(cp) = &fd.cpoint {
                        if cp.jaction != 0 {
                            ch.base.trans_frame(cp.jaction, 10);
                            return Some(COMBO_CONSUMED);
                        }
                    }
                }
            }
            None
        }
        _ => None,
    }
}

fn state10(ch: &mut Character, event: &str, _key: Option<&str>) -> Option<i32> {
    match event {
        "state_exit" => {
            ch.base.held_by = None;
            None
        }
        "frame" => {
            ch.base.statemem_frame_tu = true;
            ch.base.trans.set_wait(99, 10, 99);
            None
        }
        "TU" => {
            if ch.base.frame.n == 135 {
                ch.base.ps.vy = 0.0;
            }
            None
        }
        _ => None,
    }
}

fn state11(ch: &mut Character, event: &str, _key: Option<&str>) -> Option<i32> {
    if event == "state_entry" {
        ch.base.trans.inc_wait(0, 20, 1);
    }
    if event == "frame" && matches!(ch.base.frame.n, 221 | 223 | 225) {
        ch.base.trans.set_next(999, 20);
    }
    None
}

fn state12(ch: &mut Character, event: &str, key: Option<&str>) -> Option<i32> {
    match event {
        "frame" => {
            // fall frame chain — also in character.state12_fall_tu
            None
        }
        "fall_onto_ground" | "fell_onto_ground" => {
            // return next frame id for physics
            let n = ch.base.frame.n;
            let spd = (ch.base.ps.vx * ch.base.ps.vx
                + ch.base.ps.vy * ch.base.ps.vy
                + ch.base.ps.vz * ch.base.ps.vz)
                .sqrt();
            let bounce = spd > global::BOUNCE_LIMIT_XY || ch.base.ps.vy.abs() > global::BOUNCE_LIMIT_Y;
            if bounce {
                if (203..=206).contains(&n) || (180..=185).contains(&n) {
                    return Some(185);
                }
                if (186..=191).contains(&n) {
                    return Some(191);
                }
            } else {
                if (203..=206).contains(&n) || (180..=185).contains(&n) {
                    return Some(230);
                }
                if (186..=191).contains(&n) {
                    return Some(231);
                }
            }
            None
        }
        "combo" => {
            let k = key?;
            if k == "jump" && matches!(ch.base.frame.n, 182 | 188) {
                if ch.base.fall < global::FALL_KO && ch.base.hp > 0.0 {
                    if ch.base.frame.n == 182 {
                        ch.base.trans_frame(100, 10);
                    } else {
                        ch.base.trans_frame(108, 10);
                    }
                    if ch.base.ps.vx != 0.0 {
                        ch.base.ps.vx = 5.0 * ch.base.ps.vx.signum();
                    }
                    if ch.base.ps.vy == 0.0 {
                        ch.base.ps.vy = 5.0;
                    }
                    if ch.base.ps.vz != 0.0 {
                        ch.base.ps.vz = 2.0 * ch.base.ps.vz.signum();
                    }
                    return Some(COMBO_CONSUMED);
                }
            }
            Some(COMBO_CONSUMED) // always consume jump on fall
        }
        _ => None,
    }
}

fn state13(ch: &mut Character, event: &str, _key: Option<&str>) -> Option<i32> {
    if event == "state_exit" {
        ch.base.request_broken(212, 8);
    }
    None
}

fn state14(ch: &mut Character, event: &str, _key: Option<&str>) -> Option<i32> {
    match event {
        "state_entry" => {
            ch.base.fall = 0.0;
            ch.base.bdefend = 0.0;
            if ch.base.hp <= 0.0 && ch.base.counter_dead_blink < 0 {
                ch.base.counter_dead_blink = 0;
            }
            None
        }
        "state_exit" => {
            ch.base.effect.blink = true;
            ch.base.effect.super_armor = true;
            ch.base.effect.timeout = 30;
            None
        }
        _ => None,
    }
}

fn state15(ch: &mut Character, event: &str, key: Option<&str>) -> Option<i32> {
    match event {
        "frame" => {
            let n = ch.base.frame.n;
            let pn = ch.base.frame.pn;
            if n == 19 && holding_heavy(ch) {
                ch.base.trans.set_next(12, 10);
            }
            if n == 215 {
                ch.base.trans.inc_wait(-1, 10, 1);
            }
            if n == 219 {
                if !crate::lf::character_ids::id_update(ch, "state15_crouch", None) {
                    match pn {
                        105 => {
                            ch.base.ps.vx *= 0.5;
                            ch.base.ps.vz *= 0.5;
                        }
                        216 | 90 | 91 | 92 => ch.base.trans.inc_wait(-1, 10, 1),
                        _ => {}
                    }
                }
            }
            if n == 54 {
                if let Some(fd) = ch.base.frame_data() {
                    if fd.next == 999 && ch.base.ps.y < 0.0 {
                        ch.base.trans.set_next(212, 10);
                    }
                }
            }
            if n == 257 {
                let _ = crate::lf::character_ids::id_update(ch, "state1280_disappear", None);
            }
            None
        }
        "combo" => {
            if ch.base.frame.n != 215 {
                return None;
            }
            let k = key?;
            if k == "def" {
                ch.base.trans_frame(102, 10);
                return Some(COMBO_CONSUMED);
            }
            if k == "jump" {
                // dash variants from crouch — simplified in handle_input too
                ch.base.trans_frame(213, 10);
                return Some(COMBO_CONSUMED);
            }
            None
        }
        _ => None,
    }
}

fn state16(_ch: &mut Character, _event: &str, _key: Option<&str>) -> Option<i32> {
    None // dance of pain — vulnerable; no input
}

fn state18(ch: &mut Character, event: &str, _key: Option<&str>) -> Option<i32> {
    if event == "frame" {
        ch.base.visualeffect_create(2); // burn sparks use effect path; 302 via broken in match
    }
    if event == "fall_onto_ground" || event == "fell_onto_ground" {
        return state12(ch, event, None);
    }
    None
}

fn state19(ch: &mut Character, event: &str, _key: Option<&str>) -> Option<i32> {
    if event == "TU" {
        let rz = ch.base.data.bmp.running_speedz;
        ch.base.ps.vz = ch.dirv() * rz;
    }
    None
}

fn state301(ch: &mut Character, event: &str, _key: Option<&str>) -> Option<i32> {
    match event {
        "frame_force" => {
            if ch.base.frame.n != 290 {
                return Some(1); // disable pre force
            }
            None
        }
        "TU" => {
            let wz = ch.base.data.bmp.walking_speedz;
            ch.base.ps.vz = ch.dirv() * wz;
            None
        }
        "hit_stop" => {
            ch.base.effect_stuck(1, 2);
            ch.base.trans.inc_wait(1, 10, 1);
            Some(1)
        }
        _ => None,
    }
}

fn state400(_ch: &mut Character, event: &str, _key: Option<&str>) -> Option<i32> {
    // teleport applied by match
    if event == "frame" {
        None
    } else {
        None
    }
}

fn state401(_ch: &mut Character, event: &str, _key: Option<&str>) -> Option<i32> {
    let _ = event;
    None
}

fn state501(ch: &mut Character, event: &str, _key: Option<&str>) -> Option<i32> {
    if event == "frame" && ch.base.frame.n == 298 {
        let _ = crate::lf::character_ids::id_update(ch, "rudolf_transform", None);
    }
    None
}

fn state1700(ch: &mut Character, event: &str, _key: Option<&str>) -> Option<i32> {
    if event == "frame" {
        ch.base.effect.timeout = 30;
        ch.base.effect.super_armor = true;
    }
    None
}
