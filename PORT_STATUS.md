# Port status (autonomous 20m session)

## Playable
- **Canonical 1:1:** https://sn99.github.io/flf-rust/game/game.html (F.LF JS + LF2_19)
- **Rust WASM WIP:** https://sn99.github.io/flf-rust/rust/

## Recent Rust combat/engine (this session)
- Fall tiers 220–226, fire/ice posteffect, weapon drop on KO fall
- Victim itr_vrest + attacker arest; kinds 0,3,4,5,8,10,11,15,16
- Super punch 72/73 vs kind 6; super catch kind 3 on state 16
- Special vs special ice/fire clash; special opoints; SpecialAttack::tu
- itr kind 2/7 weapon pick; drink sips; heavy weapon land crush
- Flute/whirlwind mass physics; AI seeks weapons; team panel tint
- effects-pool; Rudolf transform; F4 view modes; key rebind

## Still incomplete vs F.LF JS
- PeerJS lockstep multiplayer
- Full LF2_19/AI/*.js script execution
- Pixel-perfect manager DOM / transform panels
- Every itr edge case + exact TU scheduling parity
- DOM sprite path (canvas only)

**src/lf ~6.4k lines** — keep porting; JS build remains the complete game.
