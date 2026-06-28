//! AI — improved chase / defend / special from LF/AI.js patterns
use crate::core_engine::controller::Controller;
use crate::lf::livingobject::LivingObject;

pub struct AiBrain {
    pub cc: u32,
    pub target_uid: Option<u32>,
}

impl Default for AiBrain {
    fn default() -> Self {
        Self { cc: 0, target_uid: None }
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
    let Some((_, tx, tz, _ty, tstate)) = enemies.iter().find(|e| e.0 == tid) else {
        brain.target_uid = None;
        return;
    };

    let dx = tx - self_obj.ps.x;
    let dz = tz - self_obj.ps.z;
    let st = self_obj.state();

    // defend sometimes when close and enemy attacking (state 3)
    if dx.abs() < 80.0 && dz.abs() < 20.0 && *tstate == 3 && time % 20 < 8 {
        ctrl.keypress("def");
        return;
    }

    if dx.abs() > 45.0 {
        if dx > 0.0 {
            ctrl.keypress("right");
        } else {
            ctrl.keypress("left");
        }
        // run when far
        if dx.abs() > 160.0 && brain.cc % 2 == 0 {
            // double tap simulation: already moving
        }
    }
    if dz.abs() > 10.0 {
        if dz > 0.0 {
            ctrl.keypress("down");
        } else {
            ctrl.keypress("up");
        }
    }

    // attack when in range
    if dx.abs() < 65.0 && dz.abs() < 16.0 {
        if *tstate != 14 {
            // not lying
            ctrl.keypress("att");
        }
    } else if dx.abs() > 120.0 && brain.cc % 50 == 0 {
        ctrl.keypress("jump");
    }

    // occasional special: def+direction+att via sequential — approximate with att when mp high
    if self_obj.mp > 200.0 && dx.abs() < 100.0 && brain.cc % 80 == 0 {
        ctrl.keypress("def");
        if dx > 0.0 {
            ctrl.keypress("right");
        } else {
            ctrl.keypress("left");
        }
        ctrl.keypress("att");
    }

    let _ = st;
}
