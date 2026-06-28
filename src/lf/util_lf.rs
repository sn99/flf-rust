//! LF/util.js helpers used across the port
use crate::lf::data::ObjectEntry;

pub fn select_characters(objects: &[ObjectEntry]) -> Vec<&ObjectEntry> {
    objects.iter().filter(|o| o.obj_type == "character").collect()
}

pub fn normalize_path(p: &str) -> String {
    let mut s = p.replace('\\', "/");
    if !s.is_empty() && !s.ends_with('/') {
        s.push('/');
    }
    s
}

pub fn lookup_abs(table: &[(f64, f64)], key: f64) -> f64 {
    let k = key.abs();
    crate::core_engine::math::lookup(table, k)
}
