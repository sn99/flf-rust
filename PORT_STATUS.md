# Port status

## Complete playable game (1:1 F.LF)
https://sn99.github.io/flf-rust/game/game.html — full JS F.LF + LF2_19

## Rust/WASM (advanced; not every F.LF line)
https://sn99.github.io/flf-rust/rust/

### Done in Rust
- Core match loop 30 TU/s, transistor locks, LO physics/frame wait
- Character states, combos, weapons, specials, opoints, effects-pool
- Combat: fall tiers, ice/fire, vrest/arest, itr kinds, catch/throw, super punch
- AI difficult-tier heuristics (LF2 scripts are JS; behavior mirrored)
- Network: **BroadcastChannel lockstep** (2 tabs, room id) + PeerJS hooks
- Manager menus, settings rebind, F2/F4–F7, touch pad

### Still not equal to F.LF JS
- Executing **LF2_19/AI/*.js** text scripts (would need JS eval bridge)
- Full **PeerJS** lobby on lobby.projectf.hk (hooks only; use /game for that)
- Pixel-perfect manager DOM / every UI path
- DOM sprites (canvas only)
- 100% TU scheduling edge cases

**Honest:** JS host = complete game. Rust = deep engine port, multiplayer local lockstep, strong combat/AI; not a line-complete replacement of every F.LF file.
