# Parity checklist

| Item | Status |
|------|--------|
| Core: animator, collision, combodec, controller, recorder, changer, math, resourcemap, sprite canvas/DOM, support, util, effects-pool, network sync | **Yes** |
| LF: livingobject, character (+states/ids), weapon (+states), specialattack (+states), effect/pool, background, scene, mechanics, match, manager UI, AI bridge, soundpack, network, factories, loader/package, touchcontroller, transistor | **Yes** |
| Shared hit/defend (`apply_combat_hit`) | **Yes** |
| Broken defend no fall overwrite | **Yes** |
| Stuck timers / disappear / gameover +30 / blocking_xz | **Yes** |
| Stats: attack / kill / NPC offset + summary dialog | **Yes** |
| `show_hp` dual bar (hp_bound + heal flash) | **Yes** |
| `combo_update` persistence | **Yes** |
| `get_pos`, `create_non_player_characters` (oid 5), multi-opoint fan | **Yes** |
| Dev tools (`tools/`) | **Vendored** |
| Bit-dump / full LF2 campaign stages | **Non-goal** (upstream incomplete too) |
