use std::collections::HashMap;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement};
use crate::core_engine::util::document;

/// Sprite sheet descriptor (from character bmp.file entries)
#[derive(Clone, Debug)]
pub struct SheetMeta {
    pub path: String,
    pub w: f64,
    pub h: f64,
    pub row: u32,
    pub col: u32,
    pub pic_from: u32,
    pub pic_to: u32,
}

#[derive(Clone, Debug)]
pub struct SpriteInstance {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub pic: i32,
    pub facing: i32, // 1 right, -1 left
    pub mirror: bool,
    pub visible: bool,
    pub alpha: f64,
    pub scale_x: f64,
    pub scale_y: f64,
    pub sheets: Vec<SheetMeta>,
}

impl Default for SpriteInstance {
    fn default() -> Self {
        Self {
            x: 0.0, y: 0.0, z: 0.0, pic: 0, facing: 1, mirror: false,
            visible: true, alpha: 1.0, scale_x: 1.0, scale_y: 1.0, sheets: vec![],
        }
    }
}

impl SpriteInstance {
    pub fn sheet_for_pic(&self, pic: u32) -> Option<&SheetMeta> {
        self.sheets.iter().find(|s| pic >= s.pic_from && pic <= s.pic_to)
    }

    pub fn pic_uv(&self, pic: u32) -> Option<(f64, f64, f64, f64, &str)> {
        let sheet = self.sheet_for_pic(pic)?;
        let local = pic - sheet.pic_from;
        let col = (local % sheet.row) as f64;
        let row = (local / sheet.row) as f64;
        Some((col * sheet.w, row * sheet.h, sheet.w, sheet.h, sheet.path.as_str()))
    }
}

/// Canvas renderer with image cache
pub struct CanvasRenderer {
    pub canvas: HtmlCanvasElement,
    pub ctx: CanvasRenderingContext2d,
    pub images: HashMap<String, HtmlImageElement>,
    pub width: f64,
    pub height: f64,
    pub cam_x: f64,
    pub cam_y: f64,
    asset_root: String,
}

impl CanvasRenderer {
    pub fn from_selector(sel: &str, asset_root: &str) -> Result<Self, String> {
        let el = document()
            .query_selector(sel)
            .map_err(|e| format!("{:?}", e))?
            .ok_or_else(|| format!("canvas {} not found", sel))?;
        let canvas: HtmlCanvasElement = el.dyn_into().map_err(|_| "not canvas")?;
        let ctx = canvas
            .get_context("2d")
            .map_err(|e| format!("{:?}", e))?
            .ok_or("no 2d")?
            .dyn_into::<CanvasRenderingContext2d>()
            .map_err(|_| "ctx")?;
        Ok(Self {
            canvas,
            ctx,
            images: HashMap::new(),
            width: 794.0,
            height: 400.0,
            cam_x: 0.0,
            cam_y: 0.0,
            asset_root: asset_root.trim_end_matches('/').to_string(),
        })
    }

    pub fn set_size(&mut self, w: u32, h: u32) {
        self.canvas.set_width(w);
        self.canvas.set_height(h);
        self.width = w as f64;
        self.height = h as f64;
    }

    pub fn ensure_image(&mut self, path: &str) {
        if self.images.contains_key(path) { return; }
        let img = document()
            .create_element("img")
            .ok()
            .and_then(|e| e.dyn_into::<HtmlImageElement>().ok());
        if let Some(img) = img {
            let url = format!("{}/{}", self.asset_root, path);
            img.set_src(&url);
            img.set_cross_origin(Some("anonymous"));
            self.images.insert(path.to_string(), img);
        }
    }

    pub fn clear(&self, color: &str) {
        self.ctx.set_fill_style_str(color);
        self.ctx.fill_rect(0.0, 0.0, self.width, self.height);
    }

    pub fn draw_sprite(&mut self, sp: &SpriteInstance, centerx: f64, centery: f64) {
        if !sp.visible || sp.pic < 0 { return; }
        let pic = sp.pic as u32;
        let Some((sx, sy, sw, sh, path)) = sp.pic_uv(pic) else { return; };
        self.ensure_image(path);
        let Some(img) = self.images.get(path) else { return; };
        if !img.complete() { return; }

        let screen_x = sp.x - self.cam_x;
        let screen_y = sp.y - sp.z * 0.0 - self.cam_y; // z is depth; y is height in LF2
        // In LF2, ps.y is height (up), ps.z is depth on ground plane
        // Render position: x, z maps to screen y offset
        let draw_x = screen_x - centerx;
        let draw_y = (sp.z - self.cam_y) - centery - sp.y; // ground line uses z; y lifts up

        self.ctx.save();
        let _ = self.ctx.set_global_alpha(sp.alpha);
        let mirror = sp.mirror || sp.facing < 0;
        if mirror {
            self.ctx.translate(draw_x + sw, draw_y).ok();
            self.ctx.scale(-1.0 * sp.scale_x, sp.scale_y).ok();
            let _ = self.ctx.draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                img, sx, sy, sw, sh, 0.0, 0.0, sw, sh,
            );
        } else {
            let _ = self.ctx.draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                img, sx, sy, sw, sh, draw_x, draw_y, sw * sp.scale_x, sh * sp.scale_y,
            );
        }
        self.ctx.restore();
    }

    pub fn draw_image_full(&mut self, path: &str, x: f64, y: f64) {
        self.ensure_image(path);
        if let Some(img) = self.images.get(path) {
            if img.complete() {
                let _ = self.ctx.draw_image_with_html_image_element(img, x, y);
            }
        }
    }

    pub fn draw_image_scaled(&mut self, path: &str, x: f64, y: f64, w: f64, h: f64) {
        self.ensure_image(path);
        if let Some(img) = self.images.get(path) {
            if img.complete() {
                let _ = self.ctx.draw_image_with_html_image_element_and_dw_and_dh(img, x, y, w, h);
            }
        }
    }

    pub fn fill_text(&self, text: &str, x: f64, y: f64, color: &str, font: &str) {
        self.ctx.set_fill_style_str(color);
        self.ctx.set_font(font);
        let _ = self.ctx.fill_text(text, x, y);
    }
}
