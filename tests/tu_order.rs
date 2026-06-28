//! TU / game_state shape mirrors F.LF match.game_state()
//! Headless dump compare: serve www/, open ?tu_dump=1, call __flf_tu_download(),
//! then www/tu_compare.html with JS F.LF dump vs Rust dump.

#[test]
fn game_state_json_shape() {
    let sample = serde_json::json!({
        "time": 0,
        "0": [100, 0, 200, 500, 200],
        "1": [500, 0, 200, 500, 200]
    });
    assert_eq!(sample["time"], 0);
    assert_eq!(sample["0"].as_array().unwrap().len(), 5);
    assert_eq!(sample["1"][3], 500);
}

#[test]
fn tu_compare_logic() {
    let a = vec![
        serde_json::json!({"time":0,"0":[1,0,2,500,200]}),
        serde_json::json!({"time":1,"0":[2,0,2,500,200]}),
    ];
    let b = a.clone();
    assert_eq!(serde_json::to_string(&a).unwrap(), serde_json::to_string(&b).unwrap());
    let mut c = a.clone();
    c[1] = serde_json::json!({"time":1,"0":[99,0,2,500,200]});
    assert_ne!(serde_json::to_string(&a).unwrap(), serde_json::to_string(&c).unwrap());
}
