//! Port of core/math.js

/// Bounded number helper
pub fn bound(v: f64, min: f64, max: f64) -> f64 {
    if v < min {
        min
    } else if v > max {
        max
    } else {
        v
    }
}

pub fn sign(v: f64) -> f64 {
    if v > 0.0 {
        1.0
    } else if v < 0.0 {
        -1.0
    } else {
        0.0
    }
}

pub fn abs(v: f64) -> f64 {
    v.abs()
}

pub fn inbetween(x: f64, l: f64, r: f64) -> bool {
    let (lo, hi) = if l <= r { (l, r) } else { (r, l) };
    x >= lo && x <= hi
}

pub fn round_d2(i: f64) -> f64 {
    (i * 100.0).round() / 100.0
}

pub fn negligible(m: f64) -> bool {
    m > -1e-8 && m < 1e-8
}

pub type Vec2 = (f64, f64);

pub fn add(a: Vec2, b: Vec2) -> Vec2 {
    (a.0 + b.0, a.1 + b.1)
}

pub fn sub(a: Vec2, b: Vec2) -> Vec2 {
    (a.0 - b.0, a.1 - b.1)
}

pub fn sca(a: Vec2, s: f64) -> Vec2 {
    (a.0 * s, a.1 * s)
}

pub fn length(a: Vec2) -> f64 {
    (a.0 * a.0 + a.1 * a.1).sqrt()
}

pub fn distance(a: Vec2, b: Vec2) -> f64 {
    length(sub(a, b))
}

pub fn negative(a: Vec2) -> Vec2 {
    (-a.0, -a.1)
}

pub fn normalize(a: Vec2) -> Vec2 {
    let l = length(a);
    if negligible(l) {
        (0.0, 0.0)
    } else {
        (a.0 / l, a.1 / l)
    }
}

pub fn perpen(a: Vec2) -> Vec2 {
    (-a.1, a.0)
}

pub fn signed_area(p1: Vec2, p2: Vec2, p3: Vec2) -> f64 {
    (p2.0 - p1.0) * (p3.1 - p1.1) - (p3.0 - p1.0) * (p2.1 - p1.1)
}

/// Segment intersection; returns point if segments cross
pub fn intersect(p1: Vec2, p2: Vec2, p3: Vec2, p4: Vec2) -> Option<Vec2> {
    let den = (p4.1 - p3.1) * (p2.0 - p1.0) - (p4.0 - p3.0) * (p2.1 - p1.1);
    if negligible(den) {
        return None;
    }
    let ua = ((p4.0 - p3.0) * (p1.1 - p3.1) - (p4.1 - p3.1) * (p1.0 - p3.0)) / den;
    let ub = ((p2.0 - p1.0) * (p1.1 - p3.1) - (p2.1 - p1.1) * (p1.0 - p3.0)) / den;
    if (0.0..=1.0).contains(&ua) && (0.0..=1.0).contains(&ub) {
        Some((p1.0 + ua * (p2.0 - p1.0), p1.1 + ua * (p2.1 - p1.1)))
    } else {
        None
    }
}

pub fn bezier2(p0: Vec2, p1: Vec2, p2: Vec2, t: f64) -> Vec2 {
    let u = 1.0 - t;
    (
        u * u * p0.0 + 2.0 * u * t * p1.0 + t * t * p2.0,
        u * u * p0.1 + 2.0 * u * t * p1.1 + t * t * p2.1,
    )
}

pub fn bezier2_step(p0: Vec2, p1: Vec2, p2: Vec2, steps: usize) -> Vec<Vec2> {
    let n = steps.max(1);
    (0..=n)
        .map(|i| bezier2(p0, p1, p2, i as f64 / n as f64))
        .collect()
}

pub fn getstep(a: Vec2, b: Vec2, steps: usize) -> Vec<Vec2> {
    let n = steps.max(1);
    (0..=n)
        .map(|i| {
            let t = i as f64 / n as f64;
            (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t)
        })
        .collect()
}

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
