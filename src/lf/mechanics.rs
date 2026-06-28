//! Mechanical helpers for living objects (LF/mechanics.js)
use crate::core_engine::collision::Volume;
use crate::lf::data::{FrameData, ItrData};
use crate::lf::global;
use crate::lf::livingobject::Pos;

pub struct Mech {
    pub mass: f64,
}

impl Mech {
    pub fn new(mass: Option<f64>) -> Self {
        Self { mass: mass.unwrap_or(global::DEFAULT_MASS) }
    }

    pub fn body_volumes(ps: &Pos, facing: i32, frame: &FrameData) -> Vec<Volume> {
        let mut out = vec![];
        for b in &frame.bdy {
            let bx = if facing >= 0 {
                ps.x - frame.centerx + b.x
            } else {
                ps.x + frame.centerx - (b.x + b.w)
            };
            out.push(Volume {
                x: bx,
                y: ps.z - frame.centery + b.y - ps.y,
                z: ps.z,
                w: b.w,
                h: b.h,
                zwidth: global::DEFAULT_ITR_ZWIDTH,
                vx: 0.0,
                vy: 0.0,
                kind: b.kind,
            });
        }
        out
    }

    pub fn itr_volumes(ps: &Pos, facing: i32, frame: &FrameData) -> Vec<(Volume, ItrData)> {
        let mut out = vec![];
        for it in &frame.itr {
            let ix = if facing >= 0 {
                ps.x - frame.centerx + it.x
            } else {
                ps.x + frame.centerx - (it.x + it.w)
            };
            let vol = Volume {
                x: ix,
                y: ps.z - frame.centery + it.y - ps.y,
                z: ps.z,
                w: it.w,
                h: it.h,
                zwidth: it.zwidth,
                vx: it.dvx * facing as f64,
                vy: it.dvy,
                kind: it.kind,
            };
            out.push((vol, it.clone()));
        }
        out
    }
}

/// Integrate position: LF2 style (dvy negative = up, y positive = up)
pub fn integrate(ps: &mut Pos, vx: f64, vy: f64, vz: f64) {
    ps.x += vx;
    ps.z += vz;
    ps.y -= vy;
    if ps.y < 0.0 {
        ps.y = 0.0;
    }
}

pub fn apply_gravity_vy(vy: &mut f64, on_ground: bool) {
    if !on_ground || *vy != 0.0 {
        *vy += global::GRAVITY;
    }
}
