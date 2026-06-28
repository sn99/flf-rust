# Port status — full port effort

## Is Rust 1-on-1 with F.LF? **Not 100% yet**

| Layer | Status |
|-------|--------|
| **character.js** | Event/state dispatch **complete** (`character_states.rs` + ids) |
| **livingobject / match / weapons / specials** | High API + behavior coverage |
| **manager** | Menus, rebind, maximize, F1–F7, network UI — not every DOM path |
| **AI** | LF2 scripts via `ai_bridge.js` + heuristics |
| **network** | BroadcastChannel lockstep + Peer glue — not full F.Lobby |
| **sprite-dom** | Canvas only |

## Play

- **Complete F.LF game:** https://sn99.github.io/flf-rust/game/game.html (JS engine)
- **Rust engine:** https://sn99.github.io/flf-rust/rust/

See GAP_ANALYSIS.md and PARITY_CHECKLIST.md. Character.js priority **done**; remaining blockers for absolute 1:1 are PeerJS lobby, DOM sprites, manager DOM pixel parity, TU bit-identity tests.
