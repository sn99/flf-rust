# F.LF 1-on-1 checklist (Rust must tick ALL for formal “yes”)

| # | Requirement | Status |
|---|-------------|--------|
| 1 | All LF/character.js states & events | **Done** (`character_states` dispatch) |
| 2 | All livingobject.prototype methods | **~95%** APIs + behavior |
| 3 | match.js public API | **~90%** (tasks, create/destroy, RNG, for_all, tu_trans) |
| 4 | manager.js full DOM UX | **~70%** (menus, full key rebind, maximize, summary, pause) |
| 5 | AI.js + LF2_19/AI/*.js | **~75%** (full AIin.frame cache in bridge + scripts) |
| 6 | PeerJS lockstep | **~60%** (BroadcastChannel + PeerJS CDN room host id `flf-{room}`) — not F.Lobby server protocol |
| 7 | sprite-dom OR canvas accepted as F.LF canvas path | **Canvas** (F.LF also supports canvas; DOM path not required if canvas path is valid) |
| 8 | controller-changer | **~80%** (all P1/P2 actions rebind + save) |
| 9 | effects-pool | **~85%** |
| 10 | TU order / verify | **Partial** (network verify HP; not bit-identical proof) |

**Formal answer: Rust is not certified 1-on-1** until PeerJS↔F.Lobby and TU dumps match.

**Practical answer: project ships complete F.LF via `/game/game.html`; Rust is a full-engine port with remaining net/TU certification gaps.**
