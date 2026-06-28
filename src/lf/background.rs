use serde_json::Value;
use crate::core_engine::sprite::{lf2_rect_color, CanvasRenderer};
use crate::lf::global;

#[derive(Clone, Debug)]
pub struct Background {
    pub id: i32,
    pub name: String,
    pub width: f64,
    pub zboundary: (f64, f64),
    pub shadow: String,
    pub shadow_w: f64,
    pub shadow_h: f64,
    pub layers: Vec<BgLayer>,
    pub timer: u32,
}

#[derive(Clone, Debug)]
pub struct BgLayer {
    pub path: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub rect_color: Option<String>,
    pub cc: i32,
    pub c1: i32,
    pub c2: i32,
    /// Parallax ratio (F.LF layer width group): 0 = static sky, 1 = floor scroll 1:1
    pub ratio: f64,
}

impl Background {
    pub fn from_json(id: i32, v: &Value) -> Self {
        let width = v["width"].as_f64().unwrap_or(794.0);
        let zb = &v["zboundary"];
        let zboundary = if let Some(a) = zb.as_array() {
            (
                a.first().and_then(|x| x.as_f64()).unwrap_or(300.0),
                a.get(1).and_then(|x| x.as_f64()).unwrap_or(480.0),
            )
        } else {
            (316.0, 442.0)
        };
        let shadow = v["shadow"].as_str().unwrap_or("bg/hkc/s.png").to_string();
        let (shadow_w, shadow_h) = if let Some(a) = v["shadowsize"].as_array() {
            (
                a.first().and_then(|x| x.as_f64()).unwrap_or(37.0),
                a.get(1).and_then(|x| x.as_f64()).unwrap_or(9.0),
            )
        } else {
            (37.0, 9.0)
        };
        let mut layers = vec![];
        if let Some(layer) = v.get("layer").and_then(|l| l.as_array()) {
            for L in layer {
                let path = L.get("pic").and_then(|p| p.as_str()).unwrap_or("").to_string();
                let rect_color = L.get("rect").and_then(|r| r.as_i64()).map(lf2_rect_color);
                let lw = L["width"].as_f64().unwrap_or(width);
                // F.LF: ratio = (layerWidth - window) / (bgWidth - window)
                let denom = (width - global::WINDOW_WIDTH).max(1.0);
                let ratio = ((lw - global::WINDOW_WIDTH) / denom).clamp(0.0, 1.5);
                layers.push(BgLayer {
                    path,
                    x: L["x"].as_f64().unwrap_or(0.0),
                    y: L["y"].as_f64().unwrap_or(0.0),
                    width: lw,
                    height: L["height"].as_f64().unwrap_or(0.0),
                    rect_color,
                    cc: L["cc"].as_i64().unwrap_or(0) as i32,
                    c1: L["c1"].as_i64().unwrap_or(0) as i32,
                    c2: L["c2"].as_i64().unwrap_or(0) as i32,
                    ratio,
                });
            }
        }
        Self {
            id,
            name: v["name"].as_str().unwrap_or("bg").to_string(),
            width,
            zboundary,
            shadow,
            shadow_w,
            shadow_h,
            layers,
            timer: 0,
        }
    }

    /// F.LF background.TU — tick timed layers
    pub fn tu(&mut self, time: u32) {
        self.timer = time;
    }

    /// F.LF background.leaving(o, xt) — true if x is outside scrollable field + margin
    pub fn leaving(&self, x: f64, margin: f64) -> bool {
        x < -margin || x > self.width + margin
    }

    pub fn draw(&self, ren: &mut CanvasRenderer, time: u32) {
        for layer in &self.layers {
            if layer.cc > 0 {
                let phase = (time as i32 / 2) % layer.cc;
                if phase < layer.c1 || phase > layer.c2 {
                    continue;
                }
            }
            // parallax: scroll by cam * ratio
            let x = layer.x - ren.cam_x * layer.ratio;
            if let Some(ref col) = layer.rect_color {
                let h = if layer.height > 0.0 { layer.height } else { 20.0 };
                ren.fill_rect_color(x, layer.y, layer.width, h, col);
            } else if !layer.path.is_empty() {
                if layer.height > 0.0 {
                    ren.draw_image_scaled(&layer.path, x, layer.y, layer.width, layer.height);
                } else {
                    ren.draw_image_full(&layer.path, x, layer.y);
                }
            }
        }
        if self.layers.is_empty() {
            ren.clear("#10206c");
        }
    }

    pub fn draw_shadow(&mut self, ren: &mut CanvasRenderer, feet_x: f64, feet_z: f64) {
        let x = feet_x - ren.cam_x - self.shadow_w / 2.0;
        let y = feet_z - ren.cam_y - self.shadow_h / 2.0;
        ren.draw_image_scaled(&self.shadow, x, y, self.shadow_w, self.shadow_h);
    }
}
