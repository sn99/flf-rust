use serde_json::Value;
use crate::core_engine::sprite::CanvasRenderer;

#[derive(Clone, Debug)]
pub struct Background {
    pub id: i32,
    pub name: String,
    pub width: f64,
    pub zboundary: (f64, f64),
    pub shadow: String,
    pub layers: Vec<BgLayer>,
    pub data: Value,
}

#[derive(Clone, Debug)]
pub struct BgLayer {
    pub path: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub loop_width: f64,
}

impl Background {
    pub fn from_json(id: i32, v: &Value) -> Self {
        let width = v["width"].as_f64()
            .or_else(|| v["bmp"]["width"].as_f64())
            .unwrap_or(1200.0);
        let zb = &v["zboundary"];
        let zboundary = if zb.is_array() {
            let a = zb.as_array().unwrap();
            (
                a.first().and_then(|x| x.as_f64()).unwrap_or(300.0),
                a.get(1).and_then(|x| x.as_f64()).unwrap_or(480.0),
            )
        } else {
            (300.0, 480.0)
        };
        let mut layers = vec![];
        // layer entries vary; collect pic paths
        if let Some(layer) = v.get("layer").and_then(|l| l.as_array()) {
            for L in layer {
                let path = L.get("pic")
                    .or_else(|| L.get("file"))
                    .and_then(|p| p.as_str())
                    .unwrap_or("")
                    .to_string();
                if path.is_empty() { continue; }
                layers.push(BgLayer {
                    path,
                    x: L["x"].as_f64().unwrap_or(0.0),
                    y: L["y"].as_f64().unwrap_or(0.0),
                    width: L["width"].as_f64().unwrap_or(width),
                    height: L["height"].as_f64().unwrap_or(100.0),
                    loop_width: L["width"].as_f64().unwrap_or(width),
                });
            }
        }
        // fallback: single bg image fields
        if layers.is_empty() {
            for key in ["pic", "file", "background"].iter() {
                if let Some(p) = v.get(*key).and_then(|x| x.as_str()) {
                    layers.push(BgLayer {
                        path: p.to_string(),
                        x: 0.0, y: 0.0, width, height: 400.0, loop_width: width,
                    });
                }
            }
        }
        // recursive search for png paths in JSON
        if layers.is_empty() {
            collect_pngs(v, &mut layers, width);
        }
        Self {
            id,
            name: v["name"].as_str().unwrap_or("bg").to_string(),
            width,
            zboundary,
            shadow: v["shadow"].as_str().unwrap_or("bg/shadow.png").to_string(),
            layers,
            data: v.clone(),
        }
    }

    pub fn draw(&self, ren: &mut CanvasRenderer) {
        for layer in &self.layers {
            let scroll = ren.cam_x * 0.5; // parallax light
            let mut x = layer.x - scroll;
            // tile
            let lw = if layer.loop_width > 0.0 { layer.loop_width } else { layer.width };
            while x < ren.width {
                ren.draw_image_scaled(&layer.path, x, layer.y, layer.width, layer.height);
                x += lw;
                if lw <= 0.0 { break; }
            }
        }
        // solid fallback
        if self.layers.is_empty() {
            ren.clear("#2a4a2a");
            ren.fill_text(&self.name, 20.0, 30.0, "#fff", "16px sans-serif");
        }
    }
}

fn collect_pngs(v: &Value, layers: &mut Vec<BgLayer>, width: f64) {
    match v {
        Value::String(s) if s.ends_with(".png") || s.ends_with(".bmp") => {
            layers.push(BgLayer {
                path: s.clone(), x: 0.0, y: 0.0, width, height: 400.0, loop_width: width,
            });
        }
        Value::Array(a) => { for x in a { collect_pngs(x, layers, width); } }
        Value::Object(o) => { for x in o.values() { collect_pngs(x, layers, width); } }
        _ => {}
    }
}
