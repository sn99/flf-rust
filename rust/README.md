# F.LF — Little Fighter (Rust port + hosted original)

## Play (same as Project F)

**https://sn99.github.io/flf-rust/** → redirects to **game/game.html**

This is the **original [F.LF](https://github.com/Project-F/F.LF)** engine with **[LF2_19](https://github.com/Project-F/LF2_19)** assets — behaviour matches:

https://project-f.github.io/F.LF/game/game.html

| Path | What |
|------|------|
| [/](https://sn99.github.io/flf-rust/) | Entry → original F.LF |
| [/game/game.html](https://sn99.github.io/flf-rust/game/game.html) | Canonical game |
| [/rust/](https://sn99.github.io/flf-rust/rust/) | Experimental **Rust/WASM** rewrite (in progress) |

## Repos

- **This repo** — Rust engine source (`src/`), Pages deploy (F.LF + LF2_19 on `gh-pages`)
- **[sn99/LF2_19](https://github.com/sn99/LF2_19)** — JSON content package for the Rust loader

## Build Rust WASM (optional)

```bash
wasm-pack build --target web --out-dir www/pkg --release
cd www && python3 -m http.server 8080
```

## Credits

LF2 — Marti Wong & Starsky Wong. F.LF / LF2_19 — Project F. Rust port — sn99.
