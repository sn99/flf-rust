# Port status (post full analysis)

See also [GAP_ANALYSIS.md](GAP_ANALYSIS.md).

## Verdict

| Target | 1-on-1? |
|--------|---------|
| **F.LF JS + LF2_19** at `/game/game.html` | **Yes** — complete playable package |
| **Rust WASM alone** at `/rust/` | **No** — deep engine port, not every F.LF line |

## Repos

- Engine reference: https://github.com/Project-F/F.LF
- Data package: https://github.com/sn99/LF2_19
- This port: https://github.com/sn99/flf-rust

## Implemented toward parity (latest)

- Combat, frames, weapons, specials, effects-pool, transform, manager UI
- **LF2_19 AI scripts** via `www/js/ai_bridge.js` + preload (fallback heuristics)
- **BroadcastChannel** lockstep multiplayer (2 tabs)
- `create_object` / visualeffect / brokeneffect pipelines

## Remaining vs true 1:1 Rust

- Full PeerJS lobby (projectf) — hooks only
- DOM sprite renderer
- Pixel-perfect manager / controller-changer
- Every TU edge case in 2k-line character.js
- AI scripts that need full AIin frame cache / match APIs may partially fail → heuristics

**Play complete game:** https://sn99.github.io/flf-rust/game/game.html  
**Rust WIP:** https://sn99.github.io/flf-rust/rust/
