# F.LF — Little Fighter (Rust port)

Rust/WASM rewrite of [Project-F/F.LF](https://github.com/Project-F/F.LF) with [LF2_19](https://github.com/Project-F/LF2_19) content (JSON package: [sn99/LF2_19](https://github.com/sn99/LF2_19)).

## Play

**https://sn99.github.io/flf-rust/** → `game/game.html` (classic [F.LF](https://github.com/Project-F/F.LF) + LF2_19), same entry style as https://project-f.github.io/F.LF/game/game.html.

| Path | What |
|------|------|
| [/](https://sn99.github.io/flf-rust/) | Redirect → classic game |
| [/game/game.html](https://sn99.github.io/flf-rust/game/game.html) | Canonical JS engine |
| [/rust/](https://sn99.github.io/flf-rust/rust/) | Rust/WASM port (`www/`) |

Local Rust/WASM build:

```bash
wasm-pack build --target web --out-dir www/pkg --release
cd www && python3 -m http.server 8080
# open http://localhost:8080/
```

Push to `main` runs CI and deploys GitHub Pages (classic `/game` + `/rust` mirror of `www/`).

## Layout

| Path | Contents |
|------|----------|
| `src/core_engine/` | Port of `F.LF/core` |
| `src/lf/` | Port of `F.LF/LF` |
| `www/` | Shell HTML/CSS/JS glue + assets |
| `tools/` | Data converter, decrypt, unit test suite, AI parser (from F.LF) |
| `tests/` | Parity integration tests |

`cargo test` runs shipped-path parity checks. See `PORT_STATUS.md` and `GAP_ANALYSIS.md`.

## Credits

LF2 — Marti Wong & Starsky Wong. F.LF / LF2_19 — Project F. Rust port — sn99.
