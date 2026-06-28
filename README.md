# F.LF (Rust)

A **Rust + WebAssembly** rewrite of [Project-F/F.LF](https://github.com/Project-F/F.LF), the open-source HTML5 implementation of **Little Fighter 2**.

Play (GitHub Pages): **https://sn99.github.io/flf-rust/**

## Features

- Full LF2_19 content package (characters, weapons, backgrounds, effects, UI art)
- Game manager UI: title menu, character select, control settings, network shell
- VS match with keyboard controls (P1/P2), COM AI, weapons on field
- LF2-style frames, itrs/bdys, combos, specials (opoint), HP/MP, camera
- 30 TU/s gameplay loop, canvas renderer, FPS meter
- Same visual shell as F.LF (`application.css` + LF2 UI assets)

## Controls

| Action | Player 1 | Player 2 |
|--------|----------|----------|
| Move | W A X D | U H M K |
| Attack | S | J |
| Jump | Q | I |
| Defend | Z | , |
| Pause | F1 / P | |
| Menu | Esc / F4 | |

Character select: **A/D** (P1) or **H/K** (P2) change fighter, **Enter/Q/I** start, **F2** add COM, **Esc** back.

## Build locally

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
wasm-pack build --target web --out-dir www/pkg --release
cd www && python3 -m http.server 8080
# open http://localhost:8080
```

## Credits

- Original LF2 by Marti Wong & Starsky Wong (1999–2005)
- [F.LF](https://github.com/Project-F/F.LF) / [LF2_19](https://github.com/Project-F/LF2_19) by Project F
- This repository: clean-room **Rust** port of the engine for WASM/GitHub Pages

## License

Engine code: MIT (see `LICENSE`). LF2 assets remain under their original rights; LF2_19 is used as in the F.LF project for compatible open implementation.
