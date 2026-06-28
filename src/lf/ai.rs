//! AI — LF/AI.js AIin patterns + LF2_19 script bridge (window.__flf_ai_run) + heuristics fallback
use crate::core_engine::controller::Controller;
use crate::lf::livingobject::LivingObject;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

pub struct AiBrain {
    pub cc: u32,
    pub target_uid: Option<u32>,
    pub run_phase: u8,
    pub special_phase: u8,
    pub weapon_uid: Option<u32>,
    /// AIcon-style key buffer: (key, down)
    pub key_buf: Vec<(String, bool)>,
    pub difficulty: i8, // -1 crazy, 0 hard, 1 normal, 2 easy
    /// LF2_19 AI script name (e.g. dumbass, Challangar, Crusher, Ninja)
    pub script_name: String,
    pub prefer_script: bool,
}

impl Default for AiBrain {
    fn default() -> Self {
        Self {
            cc: 0,
            target_uid: None,
            run_phase: 0,
            special_phase: 0,
            weapon_uid: None,
            key_buf: vec![],
            difficulty: 0,
            script_name: "dumbass".into(),
            prefer_script: true,
        }
    }
}

/// Build JSON snapshot for __flf_ai_run
pub fn snapshot_json(
    self_obj: &LivingObject,
    hold_type: &str,
    hold_oid: i32,
    hold_uid: i32,
    catch_counter: i32,
    bg_w: f64,
    bg_z0: f64,
    bg_z1: f64,
) -> String {
    serde_json::json!({
        "x": self_obj.ps.x,
        "y": self_obj.ps.y,
        "z": self_obj.ps.z,
        "vx": self_obj.ps.vx,
        "vy": self_obj.ps.vy,
        "vz": self_obj.ps.vz,
        "facing": self_obj.facing,
        "hp": self_obj.hp,
        "mp": self_obj.mp,
        "fall": self_obj.fall,
        "team": self_obj.team,
        "id": self_obj.id,
        "uid": self_obj.uid,
        "state": self_obj.state(),
        "frame": self_obj.frame.n,
        "hold_type": hold_type,
        "hold_oid": hold_oid,
        "hold_uid": hold_uid,
        "blink": self_obj.effect.blink,
        "effect_timeout": self_obj.effect.timeout,
        "catch_counter": catch_counter,
        "bg_w": bg_w,
        "bg_z": [bg_z0, bg_z1],
    })
    .to_string()
}

/// Try LF2 AI script via JS bridge; returns Some(keys) if bridge returned presses
pub fn try_script_keys_sync(asset_root: &str, script: &str, self_json: &str, others_json: &str) -> Option<Vec<String>> {
    let win = web_sys::window()?;
    let runner = js_sys::Reflect::get(&win, &"__flf_ai_run".into()).ok()?;
    if !runner.is_function() {
        return None;
    }
    let args = js_sys::Array::of4(
        &JsValue::from_str(asset_root),
        &JsValue::from_str(script),
        &JsValue::from_str(self_json),
        &JsValue::from_str(others_json),
    );
    let ret = js_sys::Reflect::apply(&runner.into(), &win, &args).ok()?;
    // may be Promise
    if js_sys::Reflect::has(&ret, &"then".into()).unwrap_or(false) {
        // can't block; return None and let heuristics run this TU
        // fire-and-forget store on window for next TU — skip for determinism
        let _ = ret;
        return None;
    }
    let arr = ret.dyn_into::<js_sys::Array>().ok()?;
    let mut keys = vec![];
    for i in 0..arr.length() {
        if let Some(s) = arr.get(i).as_string() {
            keys.push(s);
        }
    }
    if keys.is_empty() {
        None
    } else {
        Some(keys)
    }
}

/// AIin.type() mapping
pub fn ai_type(obj_type: &str) -> i32 {
    match obj_type {
        "character" => 0,
        "lightweapon" => 1,
        "heavyweapon" => 2,
        "specialattack" => 3,
        "baseball" => 4,
        "criminal" => 5,
        "drink" => 6,
        _ => 0,
    }
}

fn ai_keypress(ctrl: &mut Controller, key: &str, hold: bool) {
    if hold {
        ctrl.keypress(key);
    } else {
        // tap
        ctrl.keypress(key);
    }
}

/// Full TU AI fill (LF difficult-style + weapon/drink seek)
pub fn ai_fill(
    brain: &mut AiBrain,
    self_obj: &LivingObject,
    enemies: &[(u32, f64, f64, f64, i32)],
    weapons: &[(u32, f64, f64, bool, bool)],
    holding_weapon: bool,
    hold_is_heavy: bool,
    hold_is_drink: bool,
    ctrl: &mut Controller,
    time: u32,
) {
    brain.cc = brain.cc.wrapping_add(1);
    ctrl.clear_states();
    brain.key_buf.clear();

    let st = self_obj.state();
    let easy_skip = brain.difficulty >= 2 && brain.cc % 2 == 1;
    if easy_skip {
        return;
    }

    // fall recover
    if st == 12 && self_obj.fall < 60.0 && self_obj.hp > 0.0 && brain.cc % 6 == 0 {
        ai_keypress(ctrl, "jump", true);
        return;
    }
    if matches!(st, 10 | 13 | 14) {
        return;
    }
    // being caught — mash
    if st == 10 && brain.cc % 3 == 0 {
        ai_keypress(ctrl, "att", true);
        return;
    }

    // drink when holding drink and hurt
    if holding_weapon && hold_is_drink && self_obj.hp < self_obj.hp_full * 0.7 {
        if brain.cc % 5 == 0 {
            ai_keypress(ctrl, "att", true);
        }
        return;
    }

    // seek weapon / drink
    if !holding_weapon {
        let mut best = None;
        let mut best_score = 200.0_f64;
        for (uid, x, z, held, is_drink) in weapons {
            if *held {
                continue;
            }
            let d = (x - self_obj.ps.x).hypot(z - self_obj.ps.z);
            let score = d
                - (if *is_drink && self_obj.hp < self_obj.hp_full * 0.5 {
                    50.0
                } else {
                    0.0
                });
            if score < best_score {
                best_score = score;
                best = Some((*uid, *x, *z));
            }
        }
        if let Some((uid, wx, wz)) = best {
            brain.weapon_uid = Some(uid);
            let dx = wx - self_obj.ps.x;
            let dz = wz - self_obj.ps.z;
            if dx.abs() > 8.0 {
                ai_keypress(ctrl, if dx > 0.0 { "right" } else { "left" }, true);
            }
            if dz.abs() > 6.0 {
                ai_keypress(ctrl, if dz > 0.0 { "down" } else { "up" }, true);
            }
            if dx.abs() < 40.0 && dz.abs() < 14.0 {
                ai_keypress(ctrl, "att", true);
            }
            if best_score > 55.0 {
                return;
            }
        }
    } else if hold_is_heavy {
        // heavy: approach and smash
    } else if brain.cc % 100 == 0 && self_obj.mp < 80.0 {
        ai_keypress(ctrl, "att", true); // throw light
    }

    if brain.cc % 80 == 1 || brain.target_uid.is_none() {
        let mut best = None;
        let mut best_d = f64::MAX;
        for (uid, x, z, _, _) in enemies {
            if *uid == self_obj.uid {
                continue;
            }
            let d = (x - self_obj.ps.x).hypot(z - self_obj.ps.z);
            if d < best_d {
                best_d = d;
                best = Some(*uid);
            }
        }
        brain.target_uid = best;
    }

    let Some(tid) = brain.target_uid else {
        return;
    };
    let Some((_, tx, tz, ty, tstate)) = enemies.iter().find(|e| e.0 == tid) else {
        brain.target_uid = None;
        return;
    };

    let dx = tx - self_obj.ps.x;
    let dz = tz - self_obj.ps.z;
    let dy = ty - self_obj.ps.y;
    let absx = dx.abs();
    let absz = dz.abs();

    // defend vs attack / dash
    let def_window = if brain.difficulty <= 0 { 10 } else { 6 };
    if absx < 95.0
        && absz < 24.0
        && (*tstate == 3 || *tstate == 5 || *tstate == 9)
        && time as i32 % 20 < def_window
    {
        ai_keypress(ctrl, "def", true);
        return;
    }

    // approach / run
    let run_stop = 90.0;
    if absx > 48.0 {
        let dir = if dx > 0.0 { "right" } else { "left" };
        ai_keypress(ctrl, dir, true);
        if absx > run_stop {
            brain.run_phase = brain.run_phase.wrapping_add(1);
            if brain.run_phase % 2 == 0 {
                ai_keypress(ctrl, dir, true);
            }
        }
    }
    if absz > 12.0 {
        ai_keypress(ctrl, if dz > 0.0 { "down" } else { "up" }, true);
    }

    // combat range
    if absx < 72.0 && absz < 18.0 {
        if *tstate != 14 {
            if dy < -25.0 {
                ai_keypress(ctrl, "jump", true);
                if brain.cc % 3 == 0 {
                    ai_keypress(ctrl, "att", true);
                }
            } else if holding_weapon && !hold_is_drink {
                ai_keypress(ctrl, "att", true);
            } else {
                ai_keypress(ctrl, "att", true);
                // combo pressure on hard
                if brain.difficulty <= 0 && brain.cc % 11 == 0 {
                    ai_keypress(ctrl, "att", true);
                }
            }
        }
    } else if absx > 120.0 && brain.cc % 40 == 0 {
        ai_keypress(ctrl, "jump", true);
    }

    // specials D>A / D>J style via def+dir+att
    let mp_need = if brain.difficulty < 0 { 120.0 } else { 200.0 };
    if self_obj.mp > mp_need && absx < 180.0 && !holding_weapon {
        brain.special_phase = brain.special_phase.wrapping_add(1);
        let period = if brain.difficulty < 0 { 60 } else { 100 };
        match brain.special_phase % period {
            1..=4 => ai_keypress(ctrl, "def", true),
            5..=8 => ai_keypress(ctrl, if dx > 0.0 { "right" } else { "left" }, true),
            9..=12 => {
                if self_obj.mp > 280.0 {
                    ai_keypress(ctrl, "jump", true);
                } else {
                    ai_keypress(ctrl, "att", true);
                }
            }
            _ => {}
        }
    }

    let _ = (st, ai_type(&self_obj.obj_type));
}
