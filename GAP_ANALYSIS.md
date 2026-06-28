# flf-rust vs Project-F/F.LF — completeness analysis

**Date:** 2026-06-29  
**Repos:** [sn99/flf-rust](https://github.com/sn99/flf-rust) · [Project-F/F.LF](https://github.com/Project-F/F.LF)  
**Data (not engine):** [sn99/LF2_19](https://github.com/sn99/LF2_19) (used by both)

## Executive verdict

| Question | Answer |
|----------|--------|
| Is **Rust WASM alone** a complete **1-on-1** rewrite of F.LF? | **No** |
| Is **flf-rust on GitHub Pages** a complete playable F.LF+LF2 experience? | **Yes** — via **`/game/game.html`** (hosts unmodified F.LF JS + LF2_19) |
| Is Rust a **substantial** engine port? | **Yes** — combat, frames, UI shell, AI bridge, local lockstep |

**1-on-1 means:** same behavior as F.LF JS for every module/TU edge case.  
**Rust is not there.** **JS host path is.**

## Architecture

```
F.LF (JS)                          flf-rust
─────────                          ────────
RequireJS AMD                      wasm-bindgen crate
LF/*.js + core/*.js                src/lf/*.rs + src/core_engine/*.rs
sprite-dom OR canvas               canvas only
PeerJS + F.Lobby lockstep          BroadcastChannel + optional PeerJS glue
LF2_19 AI/*.js executed            ai_bridge.js runs scripts (partial AIin) + Rust fallback
game/game.html                     game/game.html (JS) AND rust/ (WASM)
```

## Module map

| F.LF | Rust | 1-on-1? |
|------|------|---------|
| livingobject.js | livingobject.rs | ~85% |
| character.js | character.rs + character_ids.rs | ~75% |
| weapon.js | weapon.rs | ~80% |
| specialattack.js | specialattack.rs | ~75% |
| match.js | match_game.rs (+ task queue) | ~80% |
| manager.js | manager.rs | ~50% DOM |
| AI.js + AI scripts | ai.rs + www/js/ai_bridge.js | ~40–60% |
| network.js + core/network | network.rs + peer_glue.js | ~40% (no F.Lobby) |
| effect + effects-pool | effect.rs, effects_pool.rs | ~70% |
| soundpack / background / scene / mechanics | present | ~70–85% |
| sprite-dom.js | — | **0%** (canvas only) |
| controller-changer.js | settings rebind | ~30% |
| controller-recorder.js | controller_recorder.rs | **added** (~60%) |
| loader.js | package.rs | ~70% |

## Implementation plan (this pass)

1. Document gaps (this file).  
2. Match **task queue** (`create_object` / `destroy` deferred like F.LF `tasks`).  
3. **controller_recorder** module.  
4. **peer_glue.js** for optional PeerJS.  
5. AI script names per character id + existing bridge.  
6. Build, deploy Pages, verify HTTP 200.

## Remaining for true Rust 1-on-1 (not claimed done)

1. Full **PeerJS + F.Lobby** protocol (not only BroadcastChannel / optional Peer).  
2. **sprite-dom** path or accept canvas-only forever.  
3. Full **AIin.frame()** cache and match.scene APIs so every LF2 AI script is correct.  
4. Every **character.js** state event (2086 LOC).  
5. Full **manager** DOM (1893 LOC) including controller-changer tables.  
6. Bit-identical TU ordering vs JS scheduler.

## How to play “complete 1-on-1”

→ https://sn99.github.io/flf-rust/game/game.html  

Rust experiment → https://sn99.github.io/flf-rust/rust/
