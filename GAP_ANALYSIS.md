# flf-rust vs Project-F/F.LF

## Is Rust complete 1-on-1 with F.LF?

**No — not formally certified** (F.Lobby protocol + bit-identical TU tests outstanding).

**Yes for playable complete game** when using **`/game/game.html`** (hosts F.LF JS + LF2_19).

Rust `/rust/` implements essentially the full engine surface: character state machine, LO/match/weapons/specials, effects pool, manager UX, AI scripts via AIin bridge, BroadcastChannel + PeerJS room lockstep. Remaining gaps vs upstream F.LF are **F.Lobby server handshake**, **DOM sprite backend** (optional; F.LF has canvas too), and **automated TU equivalence tests**.

## Plan (executed)

1. character_states full event matrix  
2. match tasks + APIs + combat  
3. AI full AIin.frame() in ai_bridge + package AI selection  
4. PeerJS CDN + room-based host id lockstep glue  
5. Manager full key rebind + summary/maximize/pause  
6. Continuous deploy  

## Remaining to flip formal checklist to all green

- Implement or document F.Lobby as out-of-scope if Peer room mode accepted  
- Optional sprite-dom  
- Headless TU dump compare vs JS F.LF  

