//! Scene graph ordering by z then y
use crate::lf::livingobject::LivingObject;
use std::cmp::Ordering;

pub fn sort_draw_order(objects: &[&LivingObject]) -> Vec<usize> {
    let mut idx: Vec<usize> = (0..objects.len()).collect();
    idx.sort_by(|&a, &b| {
        let za = objects[a].ps.z;
        let zb = objects[b].ps.z;
        za.partial_cmp(&zb).unwrap_or(Ordering::Equal)
            .then_with(|| {
                objects[a]
                    .ps
                    .y
                    .partial_cmp(&objects[b].ps.y)
                    .unwrap_or(Ordering::Equal)
            })
    });
    idx
}
