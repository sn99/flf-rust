# F.LF (Rust / WASM)

Open-source **Little Fighter 2** engine — full rewrite of [Project-F/F.LF](https://github.com/Project-F/F.LF) in **Rust**, targeting the browser via WebAssembly.

## Play

| | URL |
|--|-----|
| **Game (engine)** | https://sn99.github.io/flf-rust/ |
| **Assets (LF2_19 JSON)** | https://sn99.github.io/LF2_19/ |
| **Asset repo** | https://github.com/sn99/LF2_19 |
| **Engine repo** | https://github.com/sn99/flf-rust |
| **Original JS demo** | https://project-f.github.io/F.LF/game/game.html |

Same split as upstream: **engine** vs **[LF2_19](https://github.com/Project-F/LF2_19)** content package.

If the game page fails to load assets, open with an explicit package:

`https://sn99.github.io/flf-rust/?package=https://sn99.github.io/LF2_19`

Hard-refresh (Ctrl+Shift+R) if you see an old cached build.

## Architecture

```
flf-rust (this repo)     → WASM engine + HTML shell (gh-pages)
LF2_19 (sibling repo)    → sprites, data JSON, UI, sound, backgrounds (gh-pages)
```

`www/index.html` embeds:

```html
<pre id="flf-config">{"package":"https://sn99.github.io/LF2_19"}</pre>
```

Local dev can use `www/assets/LF2_19` (copied package) — bootstrap probes remote first, then local.

## Build

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
wasm-pack build --target web --out-dir www/pkg --release
# optional: copy assets
# cp -a /path/to/LF2_19-rust/. www/assets/LF2_19/
cd www && python3 -m http.server 8080
```

## Port status (toward 1:1 F.LF)

Implemented / in progress:

- Package loader (manifest + all objects/backgrounds)
- Manager UI: frontpage dialog art, char select, computer count, VS dialog, settings
- Match loop 30 TU/s, camera, panels HP/MP
- LivingObject physics + `frame_force` (F.LF rules), frame transistor locks
- Character states 0–16 paths (stand/walk/run/jump/dash/defend/attack/fall/injury/lying)
- Combos, specials via `opoint`, projectile hits
- Weapons on field, pick-up, hold on `wpoint`, throw frames
- Background layers + LF2 `rect` colors + shadows
- AI opponents

Still porting for full parity:

- Complete `character.js` id-specific states (Deep/Rudolf/Davis/…)
- Catch/throw (states 9–10), full itr kinds (2–15+)
- Effect objects (blood/blast) pool + soundpack WebAudio
- DOM sprite path parity, exact panel faces
- Network multiplayer (Peer/lobby)
- Touch controller

## Credits

- LF2 — Marti Wong, Starsky Wong
- F.LF / LF2_19 — Project F
- Rust port — sn99

## License

Engine: MIT. Game assets follow LF2 / Project F package norms.
