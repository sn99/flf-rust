# Port status

| Entry | Role |
|-------|------|
| [Project-F/F.LF](https://github.com/Project-F/F.LF) | Reference JS engine |
| This repo `src/core_engine` + `src/lf` + `www/` | Rust/WASM port + LF2_19 assets |
| `tests/parity_surface.rs`, `tests/tu_order.rs` | Shipped-path parity tests |
| `www/tu_compare.html` | Optional TU dump compare vs JS |
| `tools/` | F.LF dev tools (converter, unit suite, AI parser) vendored |

## Module map (F.LF → Rust)

| F.LF | Port |
|------|------|
| `core/*` | `src/core_engine/*` (+ `www/js/network_core.js`, `peer_glue.js`) |
| `LF/*` | `src/lf/*` |
| `game/game.js` | `src/lib.rs` `start_game` + `www/js/bootstrap.js` |
| `core/controller-changer.js` | `src/core_engine/controller_changer.rs` + manager settings UI |
| `LF/loader.js` | `src/lf/package.rs` / `loader.rs` re-export |

See `GAP_ANALYSIS.md`, `PARITY_CHECKLIST.md`.
