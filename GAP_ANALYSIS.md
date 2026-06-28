# flf-rust vs Project-F/F.LF

Repos: [sn99/flf-rust](https://github.com/sn99/flf-rust) · [Project-F/F.LF](https://github.com/Project-F/F.LF)

## Is Rust complete 1-on-1?

**Playable VS 1v1 port: Yes** (full engine surface in `/rust/`).

**Formal bit-identical certification vs stock F.LF under lockstep dumps: Not proven** — use `/game/game.html` for upstream JS fidelity.

## Ported (including this pass)

| Area | Status |
|------|--------|
| Character states + fall/heal/crouch/broken-defend | Deep handlers |
| **TU_trans order** | transit → tasks → TU → interactions → bg/sound/gameover → **AI end** (`pending_ai_keys`) |
| **Defend from front** | F.LF `(att.x > vic.x) === facing right` + injury factor / break |
| **Catch** | `caught_b` / `caught_throw` / `caught_release` / coincide snap |
| **hp_bound / heal / regen** | Present |
| Weapons / specials / **chase balls** / **leaving()** | Present |
| **Background parallax ratio** + timed layers | Present |
| AI AIin scripts + embedded sources | Present |
| Manager keychanger, F-keys, network F.Lobby client | Present |
| Canvas + DOM sprites (F11) | Present |
| Stage mode **shell** (4-COM start from frontpage hit) | Present |

## Two-tier hosting

| URL | Role |
|-----|------|
| `/game/game.html` | Stock F.LF JS + LF2_19 (upstream 1:1) |
| `/rust/` | Rust/WASM comprehensive port |

## Residual vs certified identity

- Fine-grained posteffect sound/VFX tables and some id-specific quirks
- Full Fsprite_dom + Fanimator menus (HTML + CSS idle bob stand-in)
- Full LF2 stage progression scripts (shell only)
- Green TU dump vs JS under identical recorded inputs
