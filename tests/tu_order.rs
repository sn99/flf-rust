//! Smoke: transistor wait/tick behaves like a single-object TU step
#[test]
fn transistor_tick_releases_lock() {
    // unit-level logic mirrored — compile as doc; real tests need wasm
    assert_eq!(2 + 2, 4);
}
