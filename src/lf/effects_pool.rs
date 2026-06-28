//! effects-pool — reuse short-lived effect instances (core/effects-pool.js circular flavor)
use crate::lf::data::ObjectData;
use crate::lf::effect::EffectObj;
use std::collections::VecDeque;

pub struct EffectsPool {
    free: VecDeque<EffectObj>,
    live: Vec<EffectObj>,
    template: Option<ObjectData>,
    next_uid: u32,
    max_size: usize,
    batch_size: usize,
}

impl EffectsPool {
    pub fn new(max_size: usize) -> Self {
        Self {
            free: VecDeque::new(),
            live: Vec::new(),
            template: None,
            next_uid: 50_000,
            max_size,
            batch_size: 8,
        }
    }

    pub fn set_template(&mut self, data: ObjectData) {
        self.template = Some(data);
        // pre-warm a few
        for _ in 0..4 {
            if let Some(d) = self.template.clone() {
                let e = EffectObj::new(self.next_uid, d, 0.0, 0.0, 0.0);
                self.next_uid += 1;
                self.free.push_back(e);
            }
        }
    }

    /// Activate effect at position; returns true if spawned into `live`.
    pub fn create(&mut self, x: f64, y: f64, z: f64, frame: i32) -> bool {
        let mut eo = if let Some(mut e) = self.free.pop_front() {
            e.base.removed = false;
            e.base.dead = false;
            e.base.ps.x = x;
            e.base.ps.y = y;
            e.base.ps.z = z;
            e.base.ps.vx = 0.0;
            e.base.ps.vy = 0.0;
            e.base.ps.vz = 0.0;
            e.base.sp.visible = true;
            e.base.trans_frame(if frame > 0 { frame } else { 0 }, 0);
            e
        } else if self.live.len() + self.free.len() < self.max_size {
            let Some(data) = self.template.clone() else {
                return false;
            };
            // expand batch
            for _ in 0..self.batch_size.min(self.max_size - self.live.len() - self.free.len()) {
                let e = EffectObj::new(self.next_uid, data.clone(), 0.0, 0.0, 0.0);
                self.next_uid += 1;
                self.free.push_back(e);
            }
            let Some(mut e) = self.free.pop_front() else {
                return false;
            };
            e.base.ps.x = x;
            e.base.ps.y = y;
            e.base.ps.z = z;
            e.base.removed = false;
            e.base.dead = false;
            e.base.trans_frame(if frame > 0 { frame } else { 0 }, 0);
            e
        } else {
            return false;
        };
        eo.base.ps.x = x;
        eo.base.ps.y = y;
        eo.base.ps.z = z;
        self.live.push(eo);
        true
    }

    pub fn tu(&mut self, bg_z: (f64, f64), bg_w: f64) {
        let mut i = 0;
        while i < self.live.len() {
            self.live[i].tu(bg_z, bg_w);
            self.live[i].base.physics_tu(bg_z, bg_w);
            let done = self.live[i].base.removed
                || self.live[i].base.dead
                || self.live[i].base.frame.n >= 1000
                || (self.live[i].base.frame.wait_left == 0
                    && self
                        .live[i]
                        .base
                        .frame_data()
                        .map(|f| f.next == 1000 || f.next == 0)
                        .unwrap_or(false));
            if done {
                let mut e = self.live.swap_remove(i);
                e.base.sp.visible = false;
                e.base.removed = false;
                e.base.dead = false;
                self.free.push_back(e);
            } else {
                i += 1;
            }
        }
    }

    pub fn drain_live_refs(&self) -> impl Iterator<Item = &EffectObj> {
        self.live.iter()
    }

    pub fn live_mut(&mut self) -> &mut Vec<EffectObj> {
        &mut self.live
    }
}
