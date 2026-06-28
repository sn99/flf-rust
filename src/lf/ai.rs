//! AI — chase / defend / special / fall-recover patterns from LF/AI.js
use crate::core_engine::controller::Controller;
use crate::lf::livingobject::LivingObject;

pub struct AiBrain {
    pub cc: u32,
    pub target_uid: Option<u32>,
    pub run_phase: u8,
    pub special_phase: u8,
}

impl Default for AiBrain {
    fn default() -> Self {
        Self {
            cc: 0,
            target_uid: None,
            run_phase: 0,
            special_phase: 0,
        }
    }
}

/// Fill controller for one TU of AI
pub fn ai_fill(
    brain: &mut AiBrain,
    self_obj: &LivingObject,
    enemies: &[(u32, f64, f64, f64, i32)], // uid, x, z, y, state
    ctrl: &mut Controller,
    time: u32,
) {
    brain.cc = brain.cc.wrapping_add(1);
    ctrl.clear_states();

    let st = self_obj.state();

    // while falling — try recover jump on frames approximated by state 12
    if st == 12 && self_obj.fall < 60.0 && self_obj.hp > 0.0 && brain.cc % 7 == 0 {
        ctrl.keypress("jump");
        return;
    }

    // frozen / caught / lying — no actions
    if matches!(st, 10 | 13 | 14) {
        return;
    }

    // acquire target every 100 TU
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

    // defend when close and enemy attacking (state 3) or airborne threat
    if dx.abs() < 90.0 && dz.abs() < 22.0 && (*tstate == 3 || *tstate == 5) && time % 18 < 7 {
        ctrl.keypress("def");
        return;
    }

    // approach / run when far
    if dx.abs() > 50.0 {
        let dir = if dx > 0.0 { "right" } else { "left" };
        ctrl.keypress(dir);
        if dx.abs() > 140.0 {
            // simulate double-tap run: pulse key
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

    // attack when in range
    if dx.abs() < 70.0 && dz.abs() < 18.0 {
        if *tstate != 14 {
            if dy < -20.0 {
                // enemy airborne — jump attack setup
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

    // special sequences: def + direction + att/jump
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

    // pick up weapon impulse when idle and far — att near weapons handled by match
    let _ = st;
}
