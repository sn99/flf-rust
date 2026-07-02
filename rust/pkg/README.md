# F.LF — Little Fighter (Rust port)

Rust/WASM rewrite of [Project-F/F.LF](https://github.com/Project-F/F.LF) with [LF2_19](https://github.com/Project-F/LF2_19) content (JSON package: [sn99/LF2_19](https://github.com/sn99/LF2_19)).

## Play

**https://sn99.github.io/flf-rust/** — Pages deploy serves the classic JS engine at `game/game.html` for reference parity with https://project-f.github.io/F.LF/game/game.html.

Local Rust/WASM build:

```bash
wasm-pack build --target web --out-dir www/pkg --release
cd www && python3 -m http.server 8080
# open http://localhost:8080/
```

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
