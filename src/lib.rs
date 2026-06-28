//! F.LF — Rust/WASM port of Project-F/F.LF (open source LF2 engine)
#![allow(dead_code, unused_variables, clippy::too_many_arguments)]

mod core_engine;
mod lf;

use wasm_bindgen::prelude::*;

pub use core_engine::*;
pub use lf::*;

#[wasm_bindgen(start)]
pub fn wasm_start() {
    console_error_panic_hook::set_once();
}

/// Boot the game. `asset_root` is the path to LF2_19 assets (e.g. "assets/LF2_19").
#[wasm_bindgen]
pub async fn start_game(asset_root: String) -> Result<(), JsValue> {
    let package = lf::package::Package::load(&asset_root).await.map_err(|e| JsValue::from_str(&e))?;
    let manager = lf::manager::Manager::new(package, "F.LF v0.9.9 (Rust)")?;
    manager.run_loop();
    Ok(())
}

#[wasm_bindgen]
pub fn version() -> String {
    "0.9.9-rust".into()
}
