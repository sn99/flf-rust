# Port status (honest)

## A) Playable site = **complete F.LF** (not Rust)

**https://sn99.github.io/flf-rust/game/game.html** serves upstream **Project-F/F.LF** + **LF2_19** with fixed absolute asset URLs.

This **is** a 1-on-1 behavioural copy of https://project-f.github.io/F.LF/game/game.html for practical purposes (same JS engine + data). Verified UI assets load (dialog, panels, sprites).

## B) Rust/WASM rewrite = **not fully done**

Path: **https://sn99.github.io/flf-rust/rust/** and `src/` in this repo.

| Module | Rust | Notes |
|--------|------|--------|
| global, data, package | yes | LF2 JSON package |
| transistor (frame locks) | yes | F.LF-style authority |
| livingobject | partial | physics, force, injure, transit |
| character | partial | states 0–19+; not full id_update |
| weapon | partial | bounce, hold, hit |
| specialattack | partial | hit_a drain, despawn |
| effect | partial | blood/blast spawn |
| match | partial | hits, catch, throw, AI |
| manager | partial | menus, F-keys; not full DOM manager |
| soundpack | partial | audio sprite |
| background, scene, ai | partial | |
| sprite, touchcontroller, util_lf | yes (new) | canvas + on-screen pad |
| network, full sprite-dom | minimal / no | multiplayer not ported |
| loader (require plugin) | via package.rs | |

**Line scale:** ~4k Rust LF modules vs ~8k+ lines in key F.LF JS files alone (character.js 2k, manager 1.9k, …).

## Conclusion

- **Want the game to look/play like Project F today?** Use **`/game/game.html`** (done).
- **Want a finished pure-Rust 1:1 engine?** Still open work: finish character id blocks, all itr kinds, network, DOM sprite path parity, manager pixel-UI.

Continuing Rust work is iterative in `src/`; the hosted **canonical playable** is F.LF JS on Pages.
