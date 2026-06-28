# F.LF + LF2_19 vs flf-rust — completeness analysis

Sources:
- https://github.com/Project-F/F.LF (engine)
- https://github.com/sn99/LF2_19 (data package)
- https://github.com/sn99/flf-rust (this port)

## Architecture

| Layer | F.LF + LF2_19 | flf-rust |
|-------|---------------|----------|
| Engine | RequireJS AMD, `LF/*.js` + `core/*.js` (~9.2k LOC LF/) | Rust → WASM (`src/lf` + `src/core_engine` ~7.6k LOC) |
| Data | LF2_19: `data/`, `sprite/`, `bg/`, `sound/`, `AI/`, `UI/`, `manifest.js` | Same assets loaded via `Package::load` (JSON/data.js) |
| Render | DOM sprites (`sprite-dom`) **or** canvas | **Canvas only** |
| Entry | `game/game.html` + require.js | `rust/index.html` + wasm-bindgen |
| Multiplayer | PeerJS + `core/network` lockstep | BroadcastChannel + Peer hooks (not full lobby.projectf.hk) |

## Module mapping

| F.LF module | Rust | Parity |
|-------------|------|--------|
| livingobject.js | livingobject.rs | High (physics, injure, transistor, forces) |
| character.js | character.rs + character_ids.rs | High states; not every event branch |
| weapon.js | weapon.rs | Medium-high |
| specialattack.js | specialattack.rs | Medium-high |
| match.js | match_game.rs | High combat; create_object naming differs |
| manager.js | manager.rs | Medium (menus work; not pixel DOM) |
| AI.js + LF2_19/AI/*.js | ai.rs heuristics | **Low** — scripts not executed |
| network.js + core/network | network.rs | Medium — local lockstep only |
| effect.js + effects-pool | effect.rs + effects_pool.rs | Medium-high |
| soundpack.js | soundpack.rs | High |
| background.js | background.rs | Medium |
| scene.js | scene.rs | Medium (query subset) |
| mechanics.js | mechanics.rs | High volumes |
| sprite-dom.js | — | **Missing** (canvas only) |
| controller-changer.js | partial settings rebind | Low-medium |
| loader.js | package.rs | Medium |

## LF2_19 data

Package is **content**, not engine. Rust loads the same manifest/data/sprites/sounds when `asset_root` points at LF2_19 (as on Pages). AI `.js` files are **executable scripts** in F.LF; Rust does not run them unless bridged to JS.

## Verdict: **Not 1-on-1 complete** (Rust alone)

| Criterion | Result |
|-----------|--------|
| Same assets (LF2_19) on Pages | Yes (game/ + rust assets) |
| Same engine behavior line-by-line | **No** |
| Full playable LF2 experience | **Yes via /game/game.html (JS F.LF)** |
| Rust WASM alone = F.LF | **No** — advanced subset |

## Implementation plan (remaining)

1. **AI script bridge** — fetch LF2_19/AI/*.js, run via JS `Function` with AIin/AIcon shims fed from WASM snapshots each AI TU.
2. **Match API names** — `create_object`, `get_living_object`, `visualeffect_create` exposed for parity.
3. **Network** — keep BroadcastChannel; document PeerJS glue; optional peerjs CDN in rust/index.
4. **Manager** — maximize already partial; key table done; skip full controller-changer DOM unless time.
5. **Sprite-DOM** — out of scope for WASM canvas path; document as intentional.
6. **Verify** — build, deploy gh-pages, live 200.

