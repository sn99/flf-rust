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
- **Character** states 0–19, 301, 400/401, 501, 1700; combos; weapons hold; catch/throw hooks
- **character_ids**: full **id_update** event matrix (state3_*, state15_crouch, disappear, rudolf_transform/revert, Louis hit_ja block, Davis hit_stop)
- **Rudolf transform**: swap object data + smoke 204; DJA revert
- **Catch/cpoint**: injury, vaction, taction/aaction/jaction, cover zz, throwinjury delayed land
- **Fall recovery** jump on 182/188; fall wait180; lying clears fall/bdefend
- **Disappear** blink counter (Rudolf 257 / 1280)
- **Teleport** 400/401 via scene query
- **Scene query** (distance sort for teleport/AI targets)
- **Weapons**: light/heavy land bounce, hold/drop, weapon→char hits, broken 320 debris
- **Specials**: hit_a HP drain, hit_j vz, off-stage despawn
- **Effects**: blood/blast by itr.effect; ice exit debris
- **Match**: hits + Davis hit_stop override, catches, throws, AI (chase/def/special/fall recover), sounds, panels, camera
- **Super punch** scope (frames 72/73 vs victim itr kind 6 → frame 70)
- **itr kind 16** whirlwind pull; **drink** weapon sip heal
- **effects-pool** circular reuse for blood/blast + legacy effects vec
- **Soundpack**: audio sprite from soundpack.json
- **Manager**: frontpage/char/COM/VS/settings/network UI; **click-to-rebind** keys; F2/F5–F7; demo; network connect log
- **Touch** on-screen pad; **sprite** helper module
- **Network**: session shell + input queue (no PeerJS lockstep)

### Still not equal to every F.LF line
- Pixel-perfect **Rudolf transform** panel + full create_transform_character lifecycle
- Full **arest/vrest** matrix and every itr kind edge case (14 ice column, etc.)
- **Peer multiplayer** lockstep (WebRTC/PeerJS)
- Pixel-perfect **manager** DOM (maximize/wide every path)
- **AI scripts** executed from LF2_19/AI/*.js (heuristics only, improved)
- DOM sprite path (canvas only in Rust)

**Honest summary:** Hosted **F.LF JS = complete game**. **Rust = advanced incomplete** rewrite; this pass closed major id_update / catch / transform gaps. Continue in `src/` for engine parity.
