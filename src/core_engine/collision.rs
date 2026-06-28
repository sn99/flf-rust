#[derive(Clone, Copy, Debug, Default)]
pub struct Rect {
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
}

pub fn rect(a: &Rect, b: &Rect) -> bool {
    if a.bottom < b.top { return false; }
    if a.top > b.bottom { return false; }
    if a.right < b.left { return false; }
    if a.left > b.right { return false; }
    true
}

pub fn rect_flat(
    a_left: f64, a_top: f64, a_right: f64, a_bottom: f64,
    b_left: f64, b_top: f64, b_right: f64, b_bottom: f64,
) -> bool {
    if a_bottom < b_top { return false; }
    if a_top > b_bottom { return false; }
    if a_right < b_left { return false; }
    if a_left > b_right { return false; }
    true
}

pub fn normalize_rect(mut r: Rect) -> Rect {
    if r.left > r.right {
        std::mem::swap(&mut r.left, &mut r.right);
    }
    if r.top > r.bottom {
        std::mem::swap(&mut r.top, &mut r.bottom);
    }
    r
}

/// 3D volume for LF2 itr/bdy (xy rect + z range)
#[derive(Clone, Copy, Debug, Default)]
pub struct Volume {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
    pub h: f64,
    pub zwidth: f64,
    pub vx: f64,
    pub vy: f64,
    pub kind: i32,
}

impl Volume {
    pub fn rect(&self) -> Rect {
        Rect {
            left: self.x,
            top: self.y,
            right: self.x + self.w,
            bottom: self.y + self.h,
        }
    }

    pub fn intersects_xy(&self, other: &Volume) -> bool {
        rect(&self.rect(), &other.rect())
    }

    pub fn intersects_z(&self, other: &Volume) -> bool {
        let a0 = self.z - self.zwidth;
        let a1 = self.z + self.zwidth;
        let b0 = other.z - other.zwidth;
        let b1 = other.z + other.zwidth;
        a1 >= b0 && a0 <= b1
    }

    pub fn intersects(&self, other: &Volume) -> bool {
        self.intersects_xy(other) && self.intersects_z(other)
    }
}
