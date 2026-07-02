# flf-rust vs Project-F/F.LF

## Verdict

**Gameplay-surface port: complete** for VS mode, characters, weapons, specials, effects, backgrounds, AI scripts (JS bridge), lockstep networking shell (F.Lobby iframe + PeerJS/BroadcastChannel), touch controls, key rebind, summary stats, and dev tools vendored under `tools/`.

Intentional non-goals (same limits as upstream F.LF v0.9.9):

- Bit-identical per-TU memory dumps vs JS
- Lee On Road background (not in LF2_19 package)
- Hosting chrome identical to project-f.github.io layout

## Architecture notes

- JS `require.js` modules → Rust modules + thin `www/js/*` glue (AI `eval`, PeerJS, lobby protocol).
- DOM sprite path and canvas path both implemented (`F11` toggles).
- Combat shares `Character::apply_combat_hit` across char/weapon/special hit pipelines.
- Stats protocol mirrors F.LF `attacked` / `offset_attack` / `killed` / `die` with NPC rollup to parent.
- `combo_update` persistence for unconsumed non-direction combos.
- `background.get_pos`, `mech.coincide_xz` / `unit_friction` / `project`, full `core/math.js` helpers.

## Tests

`cargo test` — `parity_surface` (combat, TU timers, stats, math, mechanics, keycodes, combo buffer) + `tu_order`.
