# Status after “complete port” pass (2026-06-29)

## Canonical playable (1-on-1 with Project F)

**https://sn99.github.io/flf-rust/game/game.html**

Full **F.LF JavaScript engine** + **LF2_19** data. Same behavior as  
https://project-f.github.io/F.LF/game/game.html  
(asset URLs absolute + trailing slash fix for GitHub project Pages).

## Rust/WASM port

**https://sn99.github.io/flf-rust/rust/** · source `src/lf/`

### Implemented in this pass / overall
- Frame **transistor** (authority locks)
- **LivingObject**: physics, frame_force, injure/heal, effects stuck/create, transit
- **Character** states 0–19, 301, 400/401, 1700; combos; weapons hold; catch/throw hooks
- **character_ids**: Deep/Rudolf/John/Firen/Freeze/Davis-style TU hooks; **teleport 400/401**
- **Scene query** (distance sort for teleport/AI targets)
- **Weapons**: light/heavy land bounce, hold/drop, weapon→char hits
- **Specials**: hit_a HP drain, hit_j vz, off-stage despawn
- **Effects**: blood/blast by itr.effect
- **Match**: hits, catches, throws, AI (3-TU skip), sounds, panels, camera
- **Soundpack**: audio sprite from soundpack.json
- **Manager**: frontpage/char/COM/VS/settings/network UI; F5–F7; network connect log
- **Touch** on-screen pad; **sprite** helper module
- **Network**: session shell (no PeerJS lockstep)

### Still not equal to every F.LF line
- Full **id_update** (Rudolf transform, Louis, all Deep frames, etc.)
- **All itr kinds** and exact hit-stop / cpoint matrix
- **Peer multiplayer** lockstep
- Pixel-perfect **manager** DOM (key changer tables, maximize/wide every path)
- **AI scripts** executed from LF2_19/AI/*.js (heuristics only)
- Full **effects-pool** / **broken** fragments (320)
- DOM sprite path (canvas only in Rust)

**Honest summary:** Hosted **F.LF JS = complete game**. **Rust = advanced but incomplete** rewrite; continue in `src/` for true engine parity.
