# Parity checklist (flf-rust /rust/ vs F.LF)

| # | Item | Status |
|---|------|--------|
| 1 | Character state machine | **~95%** |
| 2 | Match / LO / weapons / specials / effects | **~90%** |
| 3 | AI AIin scripts | **~85%** (bridge + heuristics) |
| 4 | Manager UX / settings / F-keys | **~90%** |
| 5 | Canvas render | **Yes** (sprite-dom optional) |
| 6 | PeerJS lockstep | **~85%** BC + Peer + network_core frame buffer |
| 7 | F.Lobby 0.1 client | **~80%** protocol + iframe + start → setup; transport lib fallback |
| 8 | TU identity / game_state dumps | **Harness ready** — formal match pending recorded runs |

**Formal answer:** Rust is **not certified bit-identical** until TU dumps match JS F.LF under the harness. F.Lobby client and lockstep application layer are **implemented** in-tree.
