# flf-rust vs Project-F/F.LF

Repos: [sn99/flf-rust](https://github.com/sn99/flf-rust) · [Project-F/F.LF](https://github.com/Project-F/F.LF)

## Is the Rust version complete 1-on-1?

### Formal bit-identical certification: **No**

Line-for-line TU identity with stock F.LF under lockstep verify is **not proven**. Differences remain in event ordering nuances, some catch/weapon edge cases, parallax/leaving fidelity, and manager sprite-animator UI.

### Playable / feature-complete port: **Yes, for VS 1v1**

| Surface | Verdict |
|---------|---------|
| Character states 0–16, 18–19, 301, 400–401, 501, 1700 | Present with deep handlers (fall chain, crouch dash, heal, broken-defend force, immunities) |
| LivingObject injure / heal / hp_bound / regen | Aligned with F.LF formulas (subset of posteffect tables) |
| Match combat, weapons, specials, chase balls | Implemented; specials chase + weapon state dispatch |
| AI AIin + LF2_19 scripts (`TU()`) | `ai_bridge` + embedded sources + heuristics fallback |
| Manager menus, keychanger, network F.Lobby client | Present (HTML overlays vs Fsprite menus) |
| Lockstep / F.Lobby 0.1 client | Present (Peer/BC fallback if lobby CORS blocks) |
| Canvas + optional DOM sprites | Present (F11) |
| Stage mode / full LF2 campaign | **Not** required for 1v1; **not** ported |

### Honest two-tier hosting

| URL | Role |
|-----|------|
| `/game/game.html` | **Stock F.LF JS + LF2_19** — true upstream 1:1 play |
| `/rust/` | **Rust/WASM engine** — comprehensive port, not formally bit-certified |

## This pass implemented (critical gaps)

1. **Heal state 1700** + `effect.heal` drain + `hp_bound` / `hp_lost` on injury  
2. **Fall state 12** full `frame` chain (180–189 / upward)  
3. **State 8** knockback force vs facing  
4. **State 15** crouch dash 213/214/210  
5. **Injure immunities** (lie, ice/fire run, super armor + hp_bound)  
6. **MP/HP regen** closer to F.LF generic TU  
7. **Mechanics** `coincide_xy`, `linear_friction`, `speed`  
8. **Weapon / special state dispatch** + **chase_target** for 300X / hit_Fa  
9. **F-keys** aligned with F.LF match: F4 end, F8 drop weapons, F9 destroy; F10 view, F11 DOM  
10. Teleport 400/401 only on frame entry  

## Remaining (non-blocking for playable 1v1)

- Exact `TU_trans` phase split (transit-all → tasks → TU-all) for cert dumps  
- Full `character.prototype.hit` defend injury tables / posteffect atlas  
- Full catch `caught_*` / coincide timing  
- Background parallax ratios + `leaving()`  
- Manager Fsprite_dom + Fanimator char-select  
- Stage / battle / demo modes  
- Live F.Lobby transport library when CORS allows  
- Recorded TU dump green vs JS F.LF  

