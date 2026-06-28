//! AI — chase / defend / special / weapon / fall-recover (LF/AI.js patterns)
use crate::core_engine::controller::Controller;
use crate::lf::livingobject::LivingObject;

pub struct AiBrain {
    pub cc: u32,
    pub target_uid: Option<u32>,
    pub run_phase: u8,
    pub special_phase: u8,
    pub weapon_uid: Option<u32>,
}

impl Default for AiBrain {
    fn default() -> Self {
        Self {
            cc: 0,
            target_uid: None,
            run_phase: 0,
            special_phase: 0,
            weapon_uid: None,
        }
    }
}

/// Fill controller for one TU of AI
/// weapons: (uid, x, z, held)
pub fn ai_fill(
    brain: &mut AiBrain,
    self_obj: &LivingObject,
    enemies: &[(u32, f64, f64, f64, i32)],
    weapons: &[(u32, f64, f64, bool, bool)],
    holding_weapon: bool,
    ctrl: &mut Controller,
    time: u32,
) {
    brain.cc = brain.cc.wrapping_add(1);
    ctrl.clear_states();

    let st = self_obj.state();

    if st == 12 && self_obj.fall < 60.0 && self_obj.hp > 0.0 && brain.cc % 7 == 0 {
        ctrl.keypress("jump");
        return;
    }
    if matches!(st, 10 | 13 | 14) {
        return;
    }

    // seek weapon if unarmed and one is near
    if !holding_weapon {
        let mut best = None;
        let mut best_d = 120.0_f64;
        for (uid, x, z, held, is_drink) in weapons {
            if *held {
                continue;
            }
            let d = (x - self_obj.ps.x).hypot(z - self_obj.ps.z);
            let score = d - (if *is_drink && self_obj.hp < self_obj.hp_full * 0.5 { 40.0 } else { 0.0 });
            if score < best_d {
                best_d = score;
                best = Some((*uid, *x, *z));
            }
        }
        if let Some((uid, wx, wz)) = best {
            brain.weapon_uid = Some(uid);
            let dx = wx - self_obj.ps.x;
            let dz = wz - self_obj.ps.z;
            if dx.abs() > 8.0 {
                ctrl.keypress(if dx > 0.0 { "right" } else { "left" });
            }
            if dz.abs() > 6.0 {
                ctrl.keypress(if dz > 0.0 { "down" } else { "up" });
            }
            if dx.abs() < 40.0 && dz.abs() < 14.0 {
                ctrl.keypress("att");
            }
            // still engage enemies if very close
            if best_d > 50.0 {
                return;
            }
        }
    } else {
        // holding weapon — LF AI.js weapon_holder: prefer throw or melee
        if self_obj.hp < self_obj.hp_full * 0.35 && brain.cc % 40 == 0 {
            // try throw
            ctrl.keypress("att");
        } else if brain.cc % 90 == 0 && self_obj.mp < 100.0 {
            ctrl.keypress("att");
        }
    }

    if brain.cc % 100 == 1 || brain.target_uid.is_none() {
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

    if dx.abs() < 90.0 && dz.abs() < 22.0 && (*tstate == 3 || *tstate == 5) && time % 18 < 7 {
        ctrl.keypress("def");
        return;
    }

    if dx.abs() > 50.0 {
        let dir = if dx > 0.0 { "right" } else { "left" };
        ctrl.keypress(dir);
        if dx.abs() > 140.0 {
            brain.run_phase = brain.run_phase.wrapping_add(1);
            if brain.run_phase % 3 == 0 {
                ctrl.keypress(dir);
            }
        }
    }
    if dz.abs() > 12.0 {
        if dz > 0.0 {
            ctrl.keypress("down");
        } else {
            ctrl.keypress("up");
        }
    }

    if dx.abs() < 70.0 && dz.abs() < 18.0 {
        if *tstate != 14 {
            if dy < -20.0 {
                ctrl.keypress("jump");
                if brain.cc % 4 == 0 {
                    ctrl.keypress("att");
                }
            } else {
                ctrl.keypress("att");
            }
        }
    } else if dx.abs() > 110.0 && brain.cc % 45 == 0 {
        ctrl.keypress("jump");
    }

    if self_obj.mp > 180.0 && dx.abs() < 160.0 {
        brain.special_phase = brain.special_phase.wrapping_add(1);
        match brain.special_phase % 120 {
            1..=3 => ctrl.keypress("def"),
            4..=6 => {
                if dx > 0.0 {
                    ctrl.keypress("right");
                } else {
                    ctrl.keypress("left");
                }
            }
            7..=9 => {
                if self_obj.mp > 250.0 && brain.cc % 2 == 0 {
                    ctrl.keypress("jump");
                } else {
                    ctrl.keypress("att");
                }
            }
            _ => {}
        }
    }

    let _ = st;
}
