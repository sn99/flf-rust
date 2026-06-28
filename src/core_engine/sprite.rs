use std::collections::HashMap;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement};
use crate::core_engine::util::document;

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
    pub facing: i32,
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
        if self.images.contains_key(path) {
            return;
        }
        let img = document()
            .create_element("img")
            .ok()
            .and_then(|e| e.dyn_into::<HtmlImageElement>().ok());
        if let Some(img) = img {
            let url = format!("{}/{}", self.asset_root, path.trim_start_matches('/'));
            img.set_src(&url);
            let _ = img.set_attribute("crossorigin", "anonymous");
            self.images.insert(path.to_string(), img);
        }
    }

    pub fn clear(&self, color: &str) {
        self.ctx.set_fill_style_str(color);
        self.ctx.fill_rect(0.0, 0.0, self.width, self.height);
    }

    /// Draw LF2 sprite: feet at (x, z) on ground plane; y is height (negative = up)
    pub fn draw_sprite(&mut self, sp: &SpriteInstance, centerx: f64, centery: f64) {
        if !sp.visible || sp.pic < 0 {
            return;
        }
        let pic = sp.pic as u32;
        let Some((sx, sy, sw, sh, path)) = sp.pic_uv(pic) else {
            return;
        };
        self.ensure_image(path);
        let Some(img) = self.images.get(path) else {
            return;
        };
        if !img.complete() || img.natural_width() == 0 {
            return;
        }

        // Screen: x horizontal, z maps to vertical with y lifting sprite up
        let feet_x = sp.x - self.cam_x;
        let feet_y = sp.z - self.cam_y;
        let draw_x = feet_x - centerx;
        let draw_y = feet_y - centery + sp.y; // y negative => draw higher

        self.ctx.save();
        let _ = self.ctx.set_global_alpha(sp.alpha);
        let mirror = sp.mirror || sp.facing < 0;
        if mirror {
            let _ = self.ctx.translate(draw_x + sw * sp.scale_x, draw_y);
            let _ = self.ctx.scale(-sp.scale_x, sp.scale_y);
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

    pub fn fill_rect_color(&self, x: f64, y: f64, w: f64, h: f64, color: &str) {
        self.ctx.set_fill_style_str(color);
        self.ctx.fill_rect(x, y, w, h);
    }

    pub fn fill_text(&self, text: &str, x: f64, y: f64, color: &str, font: &str) {
        self.ctx.set_fill_style_str(color);
        self.ctx.set_font(font);
        let _ = self.ctx.fill_text(text, x, y);
    }
}

/// LF2 `rect` color integer → CSS hex (5-6 bit RGB style used by LF2)
pub fn lf2_rect_color(rect: i64) -> String {
    // Common approach: treat as 0xRRGGBB-ish 16-bit LF color
    let r = ((rect / 65536) % 256) as u8;
    let g = ((rect / 256) % 256) as u8;
    let b = (rect % 256) as u8;
    // LF2 often stores 16-bit — try both
    if rect < 65536 {
        let r = ((rect >> 11) & 0x1F) as u8 * 8;
        let g = ((rect >> 5) & 0x3F) as u8 * 4;
        let b = (rect & 0x1F) as u8 * 8;
        return format!("#{:02x}{:02x}{:02x}", r, g, b);
    }
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

/// DOM sprite layer (F.LF core/sprite-dom) — div + img with CSS transforms
pub struct DomSpriteLayer {
    pub root: web_sys::HtmlElement,
    pub asset_root: String,
    pub cam_x: f64,
    pub cam_y: f64,
    nodes: HashMap<String, web_sys::HtmlElement>,
    imgs: HashMap<String, HtmlImageElement>,
}

impl DomSpriteLayer {
    pub fn attach(parent_sel: &str, asset_root: &str) -> Result<Self, String> {
        let parent = document()
            .query_selector(parent_sel)
            .map_err(|e| format!("{:?}", e))?
            .ok_or_else(|| format!("missing {}", parent_sel))?;
        let parent: web_sys::HtmlElement = parent.dyn_into().map_err(|_| "not el")?;
        // ensure layer
        let layer = if let Some(el) = parent.query_selector(".F_sprite_layer").ok().flatten() {
            el.dyn_into().map_err(|_| "layer")?
        } else {
            let el = document()
                .create_element("div")
                .map_err(|e| format!("{:?}", e))?
                .dyn_into::<web_sys::HtmlElement>()
                .map_err(|_| "div")?;
            el.set_class_name("F_sprite_layer");
            let _ = el.style().set_property("position", "absolute");
            let _ = el.style().set_property("left", "0");
            let _ = el.style().set_property("top", "0");
            let _ = el.style().set_property("width", "100%");
            let _ = el.style().set_property("height", "100%");
            let _ = el.style().set_property("pointer-events", "none");
            let _ = el.style().set_property("overflow", "hidden");
            let _ = parent.append_child(&el);
            el
        };
        Ok(Self {
            root: layer,
            asset_root: asset_root.trim_end_matches('/').to_string(),
            cam_x: 0.0,
            cam_y: 0.0,
            nodes: HashMap::new(),
            imgs: HashMap::new(),
        })
    }

    pub fn set_visible(&self, on: bool) {
        let _ = self.root.style().set_property("display", if on { "block" } else { "none" });
    }

    pub fn clear_frame(&mut self) {
        // hide all nodes; reused next frame
        for n in self.nodes.values() {
            let _ = n.style().set_property("visibility", "hidden");
        }
    }

    fn ensure_node(&mut self, id: &str) -> Option<&web_sys::HtmlElement> {
        if !self.nodes.contains_key(id) {
            let el = document().create_element("div").ok()?.dyn_into::<web_sys::HtmlElement>().ok()?;
            el.set_class_name("F_sprite");
            let _ = el.style().set_property("position", "absolute");
            let _ = el.style().set_property("overflow", "hidden");
            let img = document().create_element("img").ok()?.dyn_into::<HtmlImageElement>().ok()?;
            let _ = img.style().set_property("position", "absolute");
            let _ = img.style().set_property("left", "0");
            let _ = img.style().set_property("top", "0");
            let _ = img.set_attribute("draggable", "false");
            let _ = el.append_child(&img);
            let _ = self.root.append_child(&el);
            self.imgs.insert(id.to_string(), img);
            self.nodes.insert(id.to_string(), el);
        }
        self.nodes.get(id)
    }

    pub fn draw_sprite_id(&mut self, id: &str, sp: &SpriteInstance, centerx: f64, centery: f64) {
        if !sp.visible || sp.pic < 0 {
            return;
        }
        let pic = sp.pic as u32;
        let Some((sx, sy, sw, sh, path)) = sp.pic_uv(pic) else {
            return;
        };
        let url = format!("{}/{}", self.asset_root, path.trim_start_matches('/'));
        let feet_x = sp.x - self.cam_x;
        let feet_y = sp.z - self.cam_y;
        let draw_x = feet_x - centerx;
        let draw_y = feet_y - centery + sp.y;
        let mirror = sp.mirror || sp.facing < 0;
        let w = sw * sp.scale_x;
        let h = sh * sp.scale_y;

        let _ = self.ensure_node(id);
        let Some(el) = self.nodes.get(id) else {
            return;
        };
        let _ = el.style().set_property("visibility", "visible");
        let _ = el.style().set_property("width", &format!("{}px", w));
        let _ = el.style().set_property("height", &format!("{}px", h));
        let _ = el.style().set_property("left", &format!("{}px", draw_x));
        let _ = el.style().set_property("top", &format!("{}px", draw_y));
        let _ = el.style().set_property("z-index", &format!("{}", (sp.z as i32) + 1000));
        let _ = el.style().set_property("opacity", &format!("{}", sp.alpha));
        if mirror {
            let _ = el.style().set_property("transform", "scaleX(-1)");
            let _ = el.style().set_property("transform-origin", "center center");
        } else {
            let _ = el.style().set_property("transform", "none");
        }
        let _ = el.style().set_property("background-image", &format!("url(\"{}\")", url));
        let _ = el.style().set_property("background-repeat", "no-repeat");
        let _ = el
            .style()
            .set_property("background-position", &format!("-{}px -{}px", sx, sy));
        if let Some(img) = self.imgs.get(id) {
            let _ = img.style().set_property("display", "none");
        }
    }
}

/// Active renderer backend (canvas default; DOM optional like F.LF sprite-select)
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RendererKind {
    Canvas,
    Dom,
}
