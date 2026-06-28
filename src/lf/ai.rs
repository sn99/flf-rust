//! Basic AI (dumbass-style chase and attack)
use crate::core_engine::controller::Controller;
use crate::lf::livingobject::LivingObject;

pub fn ai_tu(self_obj: &LivingObject, target: Option<&LivingObject>, ctrl: &mut Controller) {
    let Some(target) = target else { return; };
    let dx = target.ps.x - self_obj.ps.x;
    let dz = target.ps.z - self_obj.ps.z;
    // clear
    ctrl.clear_states();
    if dx.abs() > 40.0 {
        if dx > 0.0 { ctrl.keypress("right"); } else { ctrl.keypress("left"); }
    }
    if dz.abs() > 8.0 {
        if dz > 0.0 { ctrl.keypress("down"); } else { ctrl.keypress("up"); }
    }
    if dx.abs() < 60.0 && dz.abs() < 15.0 {
        ctrl.keypress("att");
    }
    // occasional jump
    if dx.abs() > 120.0 && (js_sys::Math::random() < 0.02) {
        ctrl.keypress("jump");
    }
}
