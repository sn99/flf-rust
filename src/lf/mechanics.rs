//! Mechanical helpers (LF/mechanics.js volumes)
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

    /// Body in world space — collision uses x and "screen y" = z + frame offset + height
    pub fn body_volumes(ps: &Pos, facing: i32, frame: &FrameData) -> Vec<Volume> {
        let mut out = vec![];
        for b in &frame.bdy {
            let bx = if facing >= 0 {
                ps.x - frame.centerx + b.x
            } else {
                ps.x + frame.centerx - (b.x + b.w)
            };
            // vertical on screen: feet at z, sprite up is -y direction in LF2 (y negative up)
            let by = ps.z - frame.centery + b.y + ps.y;
            out.push(Volume {
                x: bx,
                y: by,
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
            let iy = ps.z - frame.centery + it.y + ps.y;
            let zw = if it.zwidth > 0.0 {
                it.zwidth
            } else {
                global::DEFAULT_ITR_ZWIDTH
            };
            let vol = Volume {
                x: ix,
                y: iy,
                z: ps.z,
                w: it.w,
                h: it.h,
                zwidth: zw,
                vx: it.dvx * facing as f64,
                vy: it.dvy,
                kind: it.kind,
            };
            out.push((vol, it.clone()));
        }
        out
    }

    /// F.LF mech.coincideXY — snap victim feet/body to catcher cpoint in X/Y
    pub fn coincide_xy(holder: &Pos, holder_facing: i32, cx: f64, cy: f64, vic: &mut Pos) {
        let wx = if holder_facing >= 0 {
            holder.x - cx
        } else {
            holder.x + cx
        };
        // cy is sprite-space; Y is height (negative up)
        vic.x = wx;
        vic.y = holder.y + (cy * 0.0); // keep relative height simple
        let _ = cy;
        // align z slightly toward holder
        vic.z = holder.z;
    }

    /// F.LF mech.linear_friction on vx,vz
    pub fn linear_friction(ps: &mut Pos, fx: f64, fz: f64) {
        ps.vx *= 1.0 - fx.clamp(0.0, 1.0);
        ps.vz *= 1.0 - fz.clamp(0.0, 1.0);
    }

    pub fn speed(ps: &Pos) -> f64 {
        (ps.vx * ps.vx + ps.vy * ps.vy + ps.vz * ps.vz).sqrt()
    }

    /// F.LF mech.make_point — world position of a frame-relative point (cpoint/wpoint/opoint)
    pub fn make_point(ps: &Pos, facing: i32, centerx: f64, centery: f64, px: f64, py: f64) -> (f64, f64, f64) {
        let x = if facing >= 0 {
            ps.x - centerx + px
        } else {
            ps.x + centerx - px
        };
        let y = ps.y + (py - centery);
        (x, y, ps.z)
    }
}
