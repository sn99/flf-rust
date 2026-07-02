//! LF2 object data structures (frames, bmp, itr, bdy, etc.)
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ObjectEntry {
    #[serde(default)]
    pub id: i32,
    #[serde(default)]
    pub name: String,
    #[serde(default, rename = "type")]
    pub obj_type: String,
    #[serde(default)]
    pub file: String,
    #[serde(default)]
    pub pic: String,
    #[serde(default)]
    pub AI: Option<i32>,
    #[serde(default)]
    pub pack: Vec<Value>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DataList {
    #[serde(default)]
    pub object: Vec<ObjectEntry>,
    #[serde(default)]
    pub background: Vec<ObjectEntry>,
    #[serde(default)]
    pub AI: Vec<ObjectEntry>,
    #[serde(default)]
    pub UI: Option<Value>,
    #[serde(default)]
    pub properties: Option<Value>,
    #[serde(default)]
    pub stage: Option<Value>,
}

#[derive(Clone, Debug, Default)]
pub struct FrameData {
    pub name: String,
    pub pic: i32,
    pub state: i32,
    pub wait: i32,
    pub next: i32,
    pub dvx: f64,
    pub dvy: f64,
    pub dvz: f64,
    pub centerx: f64,
    pub centery: f64,
    pub hit_a: i32,
    pub hit_d: i32,
    pub hit_j: i32,
    pub hit_Fa: i32,
    pub hit_Ua: i32,
    pub hit_Da: i32,
    pub hit_Fj: i32,
    pub hit_Uj: i32,
    pub hit_Dj: i32,
    pub hit_ja: i32,
    pub mp: i32,
    pub sound: String,
    pub bdy: Vec<BoxData>,
    pub itr: Vec<ItrData>,
    pub wpoint: Option<WPoint>,
    pub opoint: Option<OPoint>,
    pub cpoint: Option<CPoint>,
    pub bpoint: Option<(f64, f64)>,
    pub extra: HashMap<String, Value>,
}

#[derive(Clone, Debug, Default)]
pub struct BoxData {
    pub kind: i32,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

#[derive(Clone, Debug, Default)]
pub struct ItrData {
    pub kind: i32,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub dvx: f64,
    pub dvy: f64,
    pub fall: f64,
    pub arest: i32,
    pub vrest: i32,
    pub bdefend: f64,
    pub injury: f64,
    pub effect: i32,
    pub zwidth: f64,
    pub catchingact: Vec<i32>,
    pub caughtact: Vec<i32>,
}

#[derive(Clone, Debug, Default)]
pub struct WPoint {
    pub kind: i32,
    pub x: f64,
    pub y: f64,
    pub weaponact: i32,
    pub attacking: i32,
    pub cover: i32,
    pub dvx: f64,
    pub dvy: f64,
    pub dvz: f64,
}

#[derive(Clone, Debug, Default)]
pub struct OPoint {
    pub kind: i32,
    pub x: f64,
    pub y: f64,
    pub action: i32,
    pub dvx: f64,
    pub dvy: f64,
    pub dvz: f64,
    pub oid: i32,
    pub facing: i32,
}

#[derive(Clone, Debug, Default)]
pub struct CPoint {
    pub kind: i32,
    pub x: f64,
    pub y: f64,
    pub injury: f64,
    pub vaction: i32,
    pub aaction: i32,
    pub taction: i32,
    pub jaction: i32,
    pub throwvx: f64,
    pub throwvy: f64,
    pub throwvz: f64,
    pub hurtable: i32,
    /// -1 means unspecified (use GC.default); present in data as 0/1
    pub hurtable_set: bool,
    pub throwinjury: f64,
    pub dircontrol: i32,
    pub cover: i32,
    /// Victim hurt reaction frames while caught (cpoint kind 2)
    pub fronthurtact: i32,
    pub backhurtact: i32,
}

#[derive(Clone, Debug, Default)]
pub struct BmpData {
    pub name: String,
    pub head: String,
    pub small: String,
    pub weapon_hit_sound: String,
    pub weapon_drop_sound: String,
    pub weapon_broken_sound: String,
    pub weapon_hp: f64,
    pub weapon_drop_hurt: f64,
    pub walking_frame_rate: i32,
    pub walking_speed: f64,
    pub walking_speedz: f64,
    pub running_frame_rate: i32,
    pub running_speed: f64,
    pub running_speedz: f64,
    pub heavy_walking_speed: f64,
    pub heavy_walking_speedz: f64,
    pub heavy_running_speed: f64,
    pub heavy_running_speedz: f64,
    pub jump_height: f64,
    pub jump_distance: f64,
    pub jump_distancez: f64,
    pub dash_height: f64,
    pub dash_distance: f64,
    pub dash_distancez: f64,
    pub rowing_height: f64,
    pub rowing_distance: f64,
    pub sheets: Vec<crate::core_engine::sprite::SheetMeta>,
}

#[derive(Clone, Debug, Default)]
pub struct ObjectData {
    pub id: i32,
    pub obj_type: String,
    pub bmp: BmpData,
    pub frames: HashMap<i32, FrameData>,
    /// F.LF weapon_strength_list keyed by attacking id string
    pub weapon_strength_list: HashMap<i32, ItrData>,
}

fn f64_of(v: &Value) -> f64 {
    v.as_f64().or_else(|| v.as_i64().map(|i| i as f64)).unwrap_or(0.0)
}
fn i32_of(v: &Value) -> i32 {
    v.as_i64().map(|i| i as i32).or_else(|| v.as_f64().map(|f| f as i32)).unwrap_or(0)
}
fn str_of(v: &Value) -> String {
    v.as_str().unwrap_or("").to_string()
}

fn parse_box(v: &Value) -> BoxData {
    BoxData {
        kind: i32_of(&v["kind"]),
        x: f64_of(&v["x"]),
        y: f64_of(&v["y"]),
        w: f64_of(&v["w"]),
        h: f64_of(&v["h"]),
    }
}

fn parse_boxes(v: &Value) -> Vec<BoxData> {
    if v.is_null() { return vec![]; }
    if v.is_array() {
        return v.as_array().unwrap().iter().map(parse_box).collect();
    }
    vec![parse_box(v)]
}

fn parse_itr(v: &Value) -> ItrData {
    let mut catchingact = vec![];
    let mut caughtact = vec![];
    if let Some(a) = v.get("catchingact") {
        if let Some(arr) = a.as_array() {
            catchingact = arr.iter().map(i32_of).collect();
        } else {
            catchingact.push(i32_of(a));
        }
    }
    if let Some(a) = v.get("caughtact") {
        if let Some(arr) = a.as_array() {
            caughtact = arr.iter().map(i32_of).collect();
        } else {
            caughtact.push(i32_of(a));
        }
    }
    ItrData {
        kind: i32_of(&v["kind"]),
        x: f64_of(&v["x"]),
        y: f64_of(&v["y"]),
        w: f64_of(&v["w"]),
        h: f64_of(&v["h"]),
        dvx: f64_of(&v["dvx"]),
        dvy: f64_of(&v["dvy"]),
        fall: f64_of(&v["fall"]),
        arest: i32_of(&v["arest"]),
        vrest: i32_of(&v["vrest"]),
        bdefend: f64_of(&v["bdefend"]),
        injury: f64_of(&v["injury"]),
        effect: i32_of(&v["effect"]),
        zwidth: {
            let z = f64_of(&v["zwidth"]);
            if z == 0.0 { crate::lf::global::DEFAULT_ITR_ZWIDTH } else { z }
        },
        catchingact,
        caughtact,
    }
}

fn parse_itrs(v: &Value) -> Vec<ItrData> {
    if v.is_null() { return vec![]; }
    if v.is_array() {
        return v.as_array().unwrap().iter().map(parse_itr).collect();
    }
    vec![parse_itr(v)]
}

fn parse_frame(v: &Value) -> FrameData {
    let mut fd = FrameData {
        name: str_of(&v["name"]),
        pic: i32_of(&v["pic"]),
        state: i32_of(&v["state"]),
        wait: i32_of(&v["wait"]),
        next: i32_of(&v["next"]),
        dvx: f64_of(&v["dvx"]),
        dvy: f64_of(&v["dvy"]),
        dvz: f64_of(&v["dvz"]),
        centerx: f64_of(&v["centerx"]),
        centery: f64_of(&v["centery"]),
        hit_a: i32_of(&v["hit_a"]),
        hit_d: i32_of(&v["hit_d"]),
        hit_j: i32_of(&v["hit_j"]),
        hit_Fa: i32_of(&v["hit_Fa"]),
        hit_Ua: i32_of(&v["hit_Ua"]),
        hit_Da: i32_of(&v["hit_Da"]),
        hit_Fj: i32_of(&v["hit_Fj"]),
        hit_Uj: i32_of(&v["hit_Uj"]),
        hit_Dj: i32_of(&v["hit_Dj"]),
        hit_ja: i32_of(&v["hit_ja"]),
        mp: i32_of(&v["mp"]),
        sound: str_of(&v["sound"]),
        bdy: parse_boxes(&v["bdy"]),
        itr: parse_itrs(&v["itr"]),
        ..Default::default()
    };
    if let Some(wp) = v.get("wpoint") {
        if !wp.is_null() {
            fd.wpoint = Some(WPoint {
                kind: i32_of(&wp["kind"]),
                x: f64_of(&wp["x"]),
                y: f64_of(&wp["y"]),
                weaponact: i32_of(&wp["weaponact"]),
                attacking: i32_of(&wp["attacking"]),
                cover: i32_of(&wp["cover"]),
                dvx: f64_of(&wp["dvx"]),
                dvy: f64_of(&wp["dvy"]),
                dvz: f64_of(&wp["dvz"]),
            });
        }
    }
    if let Some(op) = v.get("opoint") {
        if !op.is_null() {
            fd.opoint = Some(OPoint {
                kind: i32_of(&op["kind"]),
                x: f64_of(&op["x"]),
                y: f64_of(&op["y"]),
                action: i32_of(&op["action"]),
                dvx: f64_of(&op["dvx"]),
                dvy: f64_of(&op["dvy"]),
                dvz: f64_of(&op["dvz"]),
                oid: i32_of(&op["oid"]),
                facing: i32_of(&op["facing"]),
            });
        }
    }
    if let Some(cp) = v.get("cpoint") {
        if !cp.is_null() {
            fd.cpoint = Some(CPoint {
                kind: i32_of(&cp["kind"]),
                x: f64_of(&cp["x"]),
                y: f64_of(&cp["y"]),
                injury: f64_of(&cp["injury"]),
                vaction: i32_of(&cp["vaction"]),
                aaction: i32_of(&cp["aaction"]),
                taction: i32_of(&cp["taction"]),
                jaction: i32_of(&cp["jaction"]),
                throwvx: f64_of(&cp["throwvx"]),
                throwvy: f64_of(&cp["throwvy"]),
                throwvz: f64_of(&cp["throwvz"]),
                hurtable: i32_of(&cp["hurtable"]),
                hurtable_set: !cp["hurtable"].is_null(),
                throwinjury: f64_of(&cp["throwinjury"]),
                dircontrol: i32_of(&cp["dircontrol"]),
                cover: i32_of(&cp["cover"]),
                fronthurtact: i32_of(&cp["fronthurtact"]),
                backhurtact: i32_of(&cp["backhurtact"]),
            });
        }
    }
    if let Some(bp) = v.get("bpoint") {
        if !bp.is_null() {
            fd.bpoint = Some((f64_of(&bp["x"]), f64_of(&bp["y"])));
        }
    }
    fd
}

fn parse_sheets(bmp: &Value) -> Vec<crate::core_engine::sprite::SheetMeta> {
    let mut sheets = vec![];
    let file = &bmp["file"];
    let arr = if file.is_array() { file.as_array().unwrap().clone() } else { vec![file.clone()] };
    for entry in arr {
        if !entry.is_object() { continue; }
        let obj = entry.as_object().unwrap();
        let mut path = String::new();
        let mut pic_from = 0u32;
        let mut pic_to = 0u32;
        for (k, val) in obj {
            if k.starts_with("file(") {
                path = str_of(val);
                // file(0-69)
                if let Some(inner) = k.strip_prefix("file(").and_then(|s| s.strip_suffix(')')) {
                    let parts: Vec<&str> = inner.split('-').collect();
                    if parts.len() == 2 {
                        pic_from = parts[0].parse().unwrap_or(0);
                        pic_to = parts[1].parse().unwrap_or(0);
                    }
                }
            }
        }
        sheets.push(crate::core_engine::sprite::SheetMeta {
            path,
            w: f64_of(&entry["w"]),
            h: f64_of(&entry["h"]),
            row: i32_of(&entry["row"]) as u32,
            col: i32_of(&entry["col"]) as u32,
            pic_from,
            pic_to,
        });
    }
    sheets
}

pub fn parse_object_data(id: i32, obj_type: &str, v: &Value) -> ObjectData {
    let bmp_v = &v["bmp"];
    let bmp = BmpData {
        name: str_of(&bmp_v["name"]),
        head: str_of(&bmp_v["head"]),
        small: str_of(&bmp_v["small"]),
        walking_frame_rate: i32_of(&bmp_v["walking_frame_rate"]),
        walking_speed: f64_of(&bmp_v["walking_speed"]),
        walking_speedz: f64_of(&bmp_v["walking_speedz"]),
        running_frame_rate: i32_of(&bmp_v["running_frame_rate"]),
        running_speed: f64_of(&bmp_v["running_speed"]),
        running_speedz: f64_of(&bmp_v["running_speedz"]),
        heavy_walking_speed: f64_of(&bmp_v["heavy_walking_speed"]),
        heavy_walking_speedz: f64_of(&bmp_v["heavy_walking_speedz"]),
        heavy_running_speed: f64_of(&bmp_v["heavy_running_speed"]),
        heavy_running_speedz: f64_of(&bmp_v["heavy_running_speedz"]),
        jump_height: f64_of(&bmp_v["jump_height"]),
        jump_distance: f64_of(&bmp_v["jump_distance"]),
        jump_distancez: f64_of(&bmp_v["jump_distancez"]),
        dash_height: f64_of(&bmp_v["dash_height"]),
        dash_distance: f64_of(&bmp_v["dash_distance"]),
        dash_distancez: f64_of(&bmp_v["dash_distancez"]),
        rowing_height: f64_of(&bmp_v["rowing_height"]),
        rowing_distance: f64_of(&bmp_v["rowing_distance"]),
        weapon_hit_sound: str_of(&bmp_v["weapon_hit_sound"]),
        weapon_drop_sound: str_of(&bmp_v["weapon_drop_sound"]),
        weapon_broken_sound: str_of(&bmp_v["weapon_broken_sound"]),
        weapon_hp: {
            let h = f64_of(&bmp_v["weapon_hp"]);
            if h > 0.0 { h } else { 200.0 }
        },
        weapon_drop_hurt: f64_of(&bmp_v["weapon_drop_hurt"]),
        sheets: parse_sheets(bmp_v),
    };
    let mut frames = HashMap::new();
    if let Some(obj) = v["frame"].as_object() {
        for (k, fv) in obj {
            if let Ok(num) = k.parse::<i32>() {
                frames.insert(num, parse_frame(fv));
            }
        }
    }
    let mut weapon_strength_list = HashMap::new();
    if let Some(obj) = v.get("weapon_strength_list").and_then(|x| x.as_object()) {
        for (k, sv) in obj {
            if let Ok(num) = k.parse::<i32>() {
                let mut itr = parse_itr(sv);
                if itr.injury == 0.0 {
                    itr.injury = f64_of(&sv["injury"]);
                }
                if itr.fall == 0.0 {
                    itr.fall = f64_of(&sv["fall"]);
                }
                if itr.dvx == 0.0 {
                    itr.dvx = f64_of(&sv["dvx"]);
                }
                weapon_strength_list.insert(num, itr);
            }
        }
    }
    ObjectData {
        id,
        obj_type: obj_type.to_string(),
        bmp,
        frames,
        weapon_strength_list,
    }
}

pub fn frame_hit_tag(frame: &FrameData, tag: &str) -> i32 {
    match tag {
        "hit_a" => frame.hit_a,
        "hit_d" => frame.hit_d,
        "hit_j" => frame.hit_j,
        "hit_Fa" => frame.hit_Fa,
        "hit_Ua" => frame.hit_Ua,
        "hit_Da" => frame.hit_Da,
        "hit_Fj" => frame.hit_Fj,
        "hit_Uj" => frame.hit_Uj,
        "hit_Dj" => frame.hit_Dj,
        "hit_ja" => frame.hit_ja,
        _ => 0,
    }
}
