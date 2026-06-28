/// Bounded number helper (from core/math.js)
pub fn bound(v: f64, min: f64, max: f64) -> f64 {
    if v < min { min } else if v > max { max } else { v }
}

pub fn sign(v: f64) -> f64 {
    if v > 0.0 { 1.0 } else if v < 0.0 { -1.0 } else { 0.0 }
}

pub fn abs(v: f64) -> f64 { v.abs() }

/// Lookup table with thresholds (speed:value pairs sorted ascending by key)
pub fn lookup(table: &[(f64, f64)], key: f64) -> f64 {
    let mut best = table.first().map(|t| t.1).unwrap_or(0.0);
    for &(k, v) in table {
        if key <= k {
            return v;
        }
        best = v;
    }
    best
}
