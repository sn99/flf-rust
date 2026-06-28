# Parity checklist — flf-rust `/rust/` vs Project-F/F.LF

| # | Item | Status |
|---|------|--------|
| 1 | Character state machine | **~97%** — fall/heal/crouch dash/broken-defend/immunities |
| 2 | Match / LO / weapons / specials / chase | **~93%** |
| 3 | AI AIin scripts | **~90%** — persistent `TU()`, embedded LF2_19 sources |
| 4 | Manager UX / keychanger / F-keys | **~92%** — F4/F8/F9 match F.LF; maximize F10 |
| 5 | Canvas + DOM sprites | **Yes** (F11 / `?renderer=dom`) |
| 6 | Peer / lockstep | **~85%** |
| 7 | F.Lobby 0.1 client | **~80%** + Peer/BC fallback |
| 8 | TU identity certification | **Harness ready** — not green vs stock JS yet |
| 9 | Stage mode | **No** (optional for 1v1) |

**Formal answer:** Rust is a **near-complete playable 1v1 port**, **not** certified bit-identical to F.LF. Use `/game/game.html` for upstream JS fidelity.
