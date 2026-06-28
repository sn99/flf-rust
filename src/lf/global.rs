//! Global constants from LF/global.js (faithful port)

pub const WINDOW_WIDTH: f64 = 794.0;
pub const WINDOW_OUTER_WIDTH: f64 = 804.0;
pub const WINDOW_WIDE_WIDTH: f64 = 1000.0;
pub const WINDOW_HEIGHT: f64 = 550.0;
pub const WINDOW_OUTER_HEIGHT: f64 = 590.0;
pub const VIEWER_HEIGHT: f64 = 400.0;
pub const CAMERA_SPEED_FACTOR: f64 = 1.0 / 18.0;

pub const FRAMERATE: f64 = 30.0;
pub const GRAVITY: f64 = 1.7;
pub const MIN_SPEED: f64 = 1.0;
pub const UNSPECIFIED: i32 = -842150451; // 0xCDCDCDCD

pub const HP_FULL: f64 = 500.0;
pub const MP_FULL: f64 = 500.0;
pub const MP_START: f64 = 200.0;

pub const DEFAULT_ITR_ZWIDTH: f64 = 12.0;
pub const DEFAULT_HIT_STOP: i32 = 3;
pub const DEFAULT_THROW_INJURY: f64 = 10.0;
pub const DEFAULT_FALL: f64 = 20.0;
pub const DEFAULT_FALL_DVY: f64 = -6.9;
pub const DEFAULT_MASS: f64 = 1.0;
pub const DEFAULT_AREST: i32 = 7;
pub const DEFAULT_VREST: i32 = 9;

pub const RECOVER_FALL: f64 = -0.45;
pub const RECOVER_BDEFEND: f64 = -0.5;
pub const EFFECT_NUM_TO_ID: i32 = 300;
pub const EFFECT_DURATION: i32 = 3;
pub const DEFEND_INJURY_FACTOR: f64 = 0.1;
pub const DEFEND_BREAK_LIMIT: f64 = 40.0;
pub const FALL_KO: f64 = 60.0;
pub const BOUNCE_LIMIT_XY: f64 = 13.0;
pub const BOUNCE_LIMIT_Y: f64 = 10.0;
pub const BOUNCE_Y: f64 = 6.0;
pub const COMBO_TIMEOUT: u32 = 10;

pub fn combo_list() -> Vec<(String, Vec<String>, bool)> {
    vec![
        ("D<A".into(), vec!["def".into(), "left".into(), "att".into()], false),
        ("D>A".into(), vec!["def".into(), "right".into(), "att".into()], false),
        ("DvA".into(), vec!["def".into(), "down".into(), "att".into()], true),
        ("D^A".into(), vec!["def".into(), "up".into(), "att".into()], true),
        ("D<J".into(), vec!["def".into(), "left".into(), "jump".into()], true),
        ("D>J".into(), vec!["def".into(), "right".into(), "jump".into()], true),
        ("DvJ".into(), vec!["def".into(), "down".into(), "jump".into()], true),
        ("D^J".into(), vec!["def".into(), "up".into(), "jump".into()], true),
        ("D<AJ".into(), vec!["def".into(), "left".into(), "att".into(), "jump".into()], true),
        ("D>AJ".into(), vec!["def".into(), "right".into(), "att".into(), "jump".into()], true),
        ("DJA".into(), vec!["def".into(), "jump".into(), "att".into()], true),
    ]
}

pub fn combo_tag(name: &str) -> Option<&'static str> {
    match name {
        "def" => Some("hit_d"),
        "jump" => Some("hit_j"),
        "att" => Some("hit_a"),
        "D<A" | "D>A" => Some("hit_Fa"),
        "DvA" => Some("hit_Da"),
        "D^A" => Some("hit_Ua"),
        "D<J" | "D>J" | "D<AJ" | "D>AJ" => Some("hit_Fj"),
        "DvJ" => Some("hit_Dj"),
        "D^J" => Some("hit_Uj"),
        "DJA" => Some("hit_ja"),
        _ => None,
    }
}

pub fn friction_fell(speed: f64) -> f64 {
    crate::core_engine::math::lookup(
        &[(2.0, 0.0), (3.0, 1.0), (5.0, 2.0), (6.0, 4.0), (9.0, 5.0), (13.0, 7.0), (25.0, 9.0)],
        speed,
    )
}

pub fn bounce_absorb(dvx: f64) -> f64 {
    crate::core_engine::math::lookup(
        &[(9.0, 1.0), (14.0, 4.0), (20.0, 10.0), (40.0, 20.0), (60.0, 30.0)],
        dvx.abs(),
    )
}

pub fn fall_wait180(dvy: f64) -> i32 {
    let a = dvy.abs();
    crate::core_engine::math::lookup(
        &[(7.0, 1.0), (9.0, 2.0), (11.0, 3.0), (13.0, 4.0), (15.0, 5.0), (17.0, 6.0)],
        a,
    ) as i32
}
