# flf-rust vs Project-F/F.LF

## Is Rust complete 1-on-1 with F.LF?

**Playable complete game:** **Yes** via **`/game/game.html`** (hosts F.LF JS + LF2_19).

**Formal bit-identical certification for `/rust/` WASM:** **Partial** — F.Lobby 0.1 client + core/network lockstep + TU dump harness are implemented; green formal checklist requires a live F.Lobby peer session and matching TU dumps vs JS F.LF under identical inputs.

## Implemented toward parity

1. character_states event matrix, match tasks, combat, weapons/specials/effects  
2. AI AIin.frame via `ai_bridge.js` + package AI selection  
3. **F.Lobby 0.1 client** (`www/js/flobby.js`): GET `/protocol`, lobby iframe, `postMessage` protocol `F.Lobby 0.1`, start → network setup  
4. **core/network lockstep** (`www/js/network_core.js`): frame buffer `{f:{t,d}}`, transfer/messenger, LF control verify layer; PeerJS + BroadcastChannel transport fallback when lobby library unavailable  
5. PeerJS room glue + Rust `NetworkSession` BroadcastChannel  
6. **`match.game_state()`** F.LF shape `{ time, "i": [x,y,z,hp,mp] }` exposed as `window.__flf_game_state`  
7. **TU harness** (`www/js/tu_harness.js`, `www/tu_compare.html`): `?tu_dump=1` → `__flf_tu_download()` → compare dumps  
8. Manager key rebind, F1–F7, summary, maximize/pause  

## Remaining for formal all-green

- Run TU dumps vs stock F.LF under recorded inputs and fix divergences  
- Optional sprite-dom backend (F.LF also supports canvas)  
- Live F.Lobby server transport library when CORS/hosting allows  

## Honest two-tier

| URL | Role |
|-----|------|
| `/game/game.html` | Complete F.LF JS + LF2_19 (1:1 play) |
| `/rust/` | Advanced WASM port with F.Lobby client + lockstep + TU tools |
