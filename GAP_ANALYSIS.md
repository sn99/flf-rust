# flf-rust vs Project-F/F.LF — full analysis

**Repos:** [sn99/flf-rust](https://github.com/sn99/flf-rust) · [Project-F/F.LF](https://github.com/Project-F/F.LF)  
**Data package (shared):** [sn99/LF2_19](https://github.com/sn99/LF2_19)

## Verdict (current)

| Question | Answer |
|----------|--------|
| Is **Rust WASM** complete **1-on-1** with F.LF source? | **NO** |
| Does **flf-rust Pages** ship a complete playable F.LF+LF2 game? | **YES** → `/game/game.html` |
| Is Rust a near-complete *engine* rewrite effort? | **YES** (~8–9k LOC, most systems present) |

**1-on-1** requires behavioral parity with F.LF JS for every module, including DOM sprites, F.Lobby PeerJS lockstep, full manager UX, full AIin, and proven TU identity. That is **not** met by Rust alone.

## Architecture

| | F.LF | flf-rust Rust path | flf-rust game path |
|--|------|--------------------|--------------------|
| Engine | RequireJS `LF/` + `core/` | `src/lf` + `src/core_engine` → WASM | Same F.LF JS as upstream |
| Data | LF2_19 | LF2_19 / assets | LF2_19 absolute URLs |
| Render | DOM or canvas | **Canvas only** | DOM/canvas as F.LF |
| Net | PeerJS + F.Lobby | BroadcastChannel + PeerJS CDN glue | Full F.LF network |
| AI | AIin + LF2_19/AI/*.js | ai_bridge + heuristics | Full F.LF AI |

## Module parity (approx.)

| F.LF | Rust | Notes |
|------|------|--------|
| character.js | character + character_states + character_ids | State/event dispatch ~complete |
| livingobject.js | livingobject.rs | Most methods + physics |
| weapon / specialattack | + weapon_states / special_states | High |
| match.js | match_game.rs | Task queue, create_object, combat, camera |
| manager.js | manager.rs | Menus, rebind, maximize, F-keys, summary |
| AI + scripts | ai.rs + ai_bridge.js | Partial AIin; scripts best-effort |
| network | network.rs + peer_glue + Peer CDN | No F.Lobby protocol |
| effect / effects-pool | effect + effects_pool | Good |
| sprite-dom | — | **Missing** |
| loader | package.rs + loader.rs alias | Good |

## Plan executed (this continuity of work)

1. Analysis (this file + PARITY_CHECKLIST).  
2. Character event matrix (`character_states`).  
3. Match tasks, APIs, combat fidelity, F6/F7, overlays.  
4. AI package scripts + bridge.  
5. Local lockstep + PeerJS optional load.  
6. Manager maximize / pause / summary.  
7. Deploy continuous on `main` + `gh-pages`.

## Remaining for Rust answer = YES

1. sprite-dom **or** automated visual parity tests.  
2. Full PeerJS **F.Lobby** lockstep (not only BC + optional Peer).  
3. Complete AIin (frame cache, full match queries).  
4. Manager pixel/DOM parity with all dialogs.  
5. Regression suite vs F.LF TU dumps.

Until then, answer remains **NO** for pure Rust.

## Play complete 1-on-1 game

https://sn99.github.io/flf-rust/game/game.html  

Rust engine: https://sn99.github.io/flf-rust/rust/
