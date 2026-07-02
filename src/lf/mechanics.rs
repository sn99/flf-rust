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

    /// F.LF mech.coincideXY — move `vic` so point `b` (on vic) coincides with point `a` along XY
    pub fn coincide_xy(a: (f64, f64), b: (f64, f64), vic: &mut Pos) {
        vic.x += a.0 - b.0;
        vic.y += a.1 - b.1;
    }

    /// F.LF mech.coincideXZ — move `vic` so point `b` coincides with `a` along XZ
    pub fn coincide_xz(a: (f64, f64), b: (f64, f64), vic: &mut Pos) {
        vic.x += a.0 - b.0;
        vic.z += a.1 - b.1;
    }

    /// Convenience: snap victim feet to holder cpoint world position (catch sync)
    pub fn snap_to_cpoint(holder: &Pos, holder_facing: i32, cx: f64, cy: f64, vic: &mut Pos) {
        let ax = if holder_facing >= 0 {
            holder.x - cx
        } else {
            holder.x + cx
        };
        let ay = holder.y + cy;
        let bx = vic.x;
        let by = vic.y;
        Self::coincide_xy((ax, ay), (bx, by), vic);
        vic.z = holder.z;
    }

    /// F.LF mech.reset
    pub fn reset_pos(ps: &mut Pos) {
        *ps = Pos::default();
    }

    /// F.LF mech.unit_friction — ±1 on ground axes
    pub fn unit_friction(ps: &mut Pos) {
        if ps.y >= 0.0 {
            if ps.vx != 0.0 {
                ps.vx += if ps.vx > 0.0 { -1.0 } else { 1.0 };
            }
            if ps.vz != 0.0 {
                ps.vz += if ps.vz > 0.0 { -1.0 } else { 1.0 };
            }
        }
    }

    /// F.LF mech.linear_friction — subtract fixed amounts toward zero
    pub fn linear_friction(ps: &mut Pos, fx: f64, fz: f64) {
        if fx != 0.0 && ps.vx != 0.0 {
            ps.vx += if ps.vx > 0.0 { -fx } else { fx };
        }
        if fz != 0.0 && ps.vz != 0.0 {
            ps.vz += if ps.vz > 0.0 { -fz } else { fz };
        }
    }

    pub fn speed(ps: &Pos) -> f64 {
        (ps.vx * ps.vx + ps.vy * ps.vy + ps.vz * ps.vz).sqrt()
    }

    /// F.LF mech.project — screen xy and z-order from feet + centers
    pub fn project(ps: &Pos, facing: i32, centerx: f64, centery: f64, sp_w: f64) -> (f64, f64, f64) {
        let sx = if facing >= 0 {
            ps.x - centerx
        } else {
            ps.x + centerx - sp_w
        };
        let sy = ps.y - centery;
        let sz = ps.z + ps.zz;
        (sx, sy + ps.z, sz)
    }

    /// Single volume from frame-local rect + optional world offset (F.LF mech.volume)
    pub fn volume(
        ps: &Pos,
        facing: i32,
        centerx: f64,
        centery: f64,
        sp_w: f64,
        ox: f64,
        oy: f64,
        w: f64,
        h: f64,
        zwidth: f64,
        offset: Option<(f64, f64, f64)>,
    ) -> Volume {
        let vx = if facing >= 0 {
            ox
        } else {
            sp_w - ox - w
        };
        let (bx, by, bz) = if let Some((dx, dy, dz)) = offset {
            (ps.x + dx, ps.y + dy, ps.z + dz)
        } else {
            let x = if facing >= 0 {
                ps.x - centerx
            } else {
                ps.x + centerx - sp_w
            };
            (x, ps.y - centery, ps.z)
        };
        Volume {
            x: bx,
            y: by + ps.z,
            z: bz,
            w,
            h,
            zwidth: if zwidth > 0.0 {
                zwidth
            } else {
                global::DEFAULT_ITR_ZWIDTH
            },
            vx,
            vy: oy,
            kind: 0,
        }
    }

    /// Empty body pseudo-array (F.LF body_empty) — zero-size volume at sprite origin
    pub fn body_empty(ps: &Pos) -> Volume {
        Volume {
            x: ps.x,
            y: ps.y + ps.z,
            z: ps.z,
            w: 0.0,
            h: 0.0,
            zwidth: 0.0,
            vx: 0.0,
            vy: 0.0,
            kind: 0,
        }
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
