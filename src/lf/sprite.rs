//! LF/sprite.js — multi-sheet sprite animator over canvas SpriteInstance
use crate::core_engine::animator::Animator;
use crate::core_engine::sprite::{SheetMeta, SpriteInstance};
use crate::lf::data::BmpData;

pub struct LfSprite {
    pub inst: SpriteInstance,
    pub w: f64,
    pub h: f64,
    pub dir: i32,
    pub cur_img: usize,
    pub animators: Vec<Animator>,
}

impl LfSprite {
    pub fn from_bmp(bmp: &BmpData) -> Self {
        let w = bmp.sheets.first().map(|s| s.w + 1.0).unwrap_or(80.0);
        let h = bmp.sheets.first().map(|s| s.h + 1.0).unwrap_or(80.0);
        let sheets = bmp.sheets.clone();
        let n = sheets.len().max(1);
        Self {
            inst: SpriteInstance {
                sheets,
                ..Default::default()
            },
            w,
            h,
            dir: 1,
            cur_img: 0,
            animators: (0..n).map(|_| Animator::default()).collect(),
        }
    }

    pub fn show_pic(&mut self, pic: i32) {
        self.inst.pic = pic;
        // pick sheet by pic range
        if let Some((i, _)) = self
            .inst
            .sheets
            .iter()
            .enumerate()
            .find(|(_, s)| pic as u32 >= s.pic_from && pic as u32 <= s.pic_to)
        {
            self.cur_img = i;
        }
    }

    pub fn switch_lr(&mut self, dir: &str) {
        self.dir = if dir == "left" { -1 } else { 1 };
        self.inst.facing = self.dir;
        self.inst.mirror = self.dir < 0;
    }

    pub fn set_x_y(&mut self, x: f64, y: f64) {
        self.inst.x = x;
        self.inst.z = y; // LF sprite y often screen y; we map in renderer
    }

    pub fn show(&mut self) {
        self.inst.visible = true;
    }
    pub fn hide(&mut self) {
        self.inst.visible = false;
    }

    /// F.LF sprite.set_alpha
    pub fn set_alpha(&mut self, a: f64) {
        self.inst.alpha = a.clamp(0.0, 1.0);
    }

    /// F.LF sprite z-order hint (maps to instance z for sort)
    pub fn set_z(&mut self, z: f64) {
        self.inst.z = z;
    }

    pub fn set_xy(&mut self, x: f64, y: f64) {
        self.set_x_y(x, y);
    }

    pub fn destroy(&mut self) {
        self.inst.visible = false;
        self.inst.sheets.clear();
        self.animators.clear();
    }
}

/// Build sheets from bmp (already in BmpData)
pub fn sheets_from_bmp(bmp: &BmpData) -> Vec<SheetMeta> {
    bmp.sheets.clone()
}
