//! Match host — LF/match.js
use crate::core_engine::controller::Controller;
use crate::core_engine::sprite::CanvasRenderer;
use crate::lf::background::Background;
use crate::lf::character::Character;
use crate::lf::data::ObjectData;
use crate::lf::global;
use crate::lf::mechanics::Mech;
use crate::lf::package::Package;
use crate::lf::specialattack::SpecialAttack;
use crate::lf::weapon::Weapon;
use serde_json::Value;
use std::cell::RefCell;
use std::rc::Rc;

/// Deferred work like F.LF match.tasks (create_object / destroy_object)
#[derive(Clone, Debug)]
pub enum MatchTask {
    CreateObject {
        oid: i32,
        team: i32,
        x: f64,
        y: f64,
        z: f64,
        facing: i32,
        action: i32,
        dvx: f64,
        dvy: f64,
    },
    DestroyUid(u32),
}

pub struct PlayerSetup {
    pub id: i32,
    pub team: i32,
    pub controller_index: Option<usize>,
    pub is_ai: bool,
    pub name: String,
}

pub struct Match {
    pub characters: Vec<Character>,
    pub weapons: Vec<Weapon>,
    pub specials: Vec<SpecialAttack>,
    pub effects: Vec<crate::lf::effect::EffectObj>,
    pub effects_pool: crate::lf::effects_pool::EffectsPool,
    pub background: Background,
    pub next_uid: u32,
    pub time: u32,
    pub game_over: bool,
    pub paused: bool,
    pub camera_x: f64,
    controllers: Rc<RefCell<Vec<Controller>>>,
    package_objects: std::collections::HashMap<i32, ObjectData>,
    pub sound: crate::lf::soundpack::Soundpack,
    pub ai_brains: Vec<crate::lf::ai::AiBrain>,
    pub properties: Value,
    pub ui_panel: Option<Value>,
    pub winner_team: Option<i32>,
    pub asset_root: String,
    pub tasks: Vec<MatchTask>,
    pub rng_state: u64,
    pub overlay_msg: String,
    pub overlay_ttl: i32,
    pub panel_remap: std::collections::HashMap<u32, u32>,
    /// F.LF F6 infinite mp mode
    pub f6_mode: bool,
    /// Basenames of AI scripts from LF2_19 AI list (e.g. dumbass, Crusher)
    pub ai_script_pool: Vec<String>,
}

impl Match {
    pub fn create(
        package: &Package,
        players: Vec<PlayerSetup>,
        background_id: i32,
        controllers: Rc<RefCell<Vec<Controller>>>,
        drop_weapons: bool,
    ) -> Result<Self, String> {
        let bg_val = package
            .backgrounds
            .get(&background_id)
            .cloned()
            .or_else(|| package.backgrounds.values().next().cloned())
            .unwrap_or(serde_json::json!({"width": 794, "zboundary": [316, 442], "name": "default"}));
        let background = Background::from_json(background_id, &bg_val);
        let (z0, z1) = background.zboundary;
        let mid_z = (z0 + z1) / 2.0;

        let mut characters = vec![];
        let mut next_uid = 1u32;
        let n = players.len().max(1) as f64;
        for (i, p) in players.iter().enumerate() {
            let data = package
                .objects
                .get(&p.id)
                .cloned()
                .ok_or_else(|| format!("character id {} not loaded", p.id))?;
            let x = 120.0 + (i as f64) * ((background.width - 240.0) / n.max(1.0));
            let mut ch = Character::new(next_uid, data, p.team, x, mid_z);
            next_uid += 1;
            ch.base.controller_index = p.controller_index;
            ch.base.ai = p.is_ai;
            if !p.name.is_empty() {
                ch.base.name = p.name.clone();
            }
            if i % 2 == 1 {
                ch.base.facing = -1;
            }
            characters.push(ch);
        }

        let mut weapons = vec![];
        if drop_weapons {
            for (k, wid) in [100i32, 101, 150, 151].iter().enumerate() {
                if let Some(data) = package.objects.get(wid).cloned() {
                    let w = Weapon::new(
                        next_uid,
                        data,
                        280.0 + k as f64 * 80.0,
                        mid_z + (k as f64 - 1.0) * 10.0,
                    );
                    next_uid += 1;
                    weapons.push(w);
                }
            }
        }

        for ch in &mut characters {
            ch.base.properties = package.properties.clone();
            let id = ch.base.id.to_string();
            if let Some(mass) = package.properties.get(&id).and_then(|p| p.get("mass")).and_then(|m| m.as_f64()) {
                ch.base.set_mass(mass);
            }
        }
        let pool = package
            .data_list
            .AI
            .iter()
            .filter_map(|e| {
                let f = e.file.rsplit('/').next().unwrap_or(&e.file);
                let name = f.trim_end_matches(".js").trim_end_matches(".as");
                if name.is_empty() { None } else { Some(name.to_string()) }
            })
            .collect::<Vec<_>>();
        let ai_brains = characters.iter().enumerate().map(|(i, c)| {
                let mut b = crate::lf::ai::AiBrain::default();
                let by_obj = package.object_entry(c.base.id).and_then(|e| e.AI).and_then(|ai_i| {
                    package.data_list.AI.get(ai_i as usize).map(|ae| {
                        let f = ae.file.rsplit('/').next().unwrap_or(&ae.file);
                        f.trim_end_matches(".js").trim_end_matches(".as").to_string()
                    })
                });
                b.script_name = by_obj.or_else(|| pool.get(i % pool.len().max(1)).cloned()).unwrap_or_else(|| {
                    match c.base.id {
                        1 => "Crusher".into(),
                        5 => "Ninja".into(),
                        9 => "Challangar".into(),
                        _ => "dumbass".into(),
                    }
                });
                b
            }).collect();
        let mut effects_pool = crate::lf::effects_pool::EffectsPool::new(64);
        // prefer blood 301 then 300 as pool template
        for tid in [301i32, 300, 302] {
            if let Some(d) = package.objects.get(&tid).cloned() {
                effects_pool.set_template(d);
                break;
            }
        }
        Ok(Self {
            characters,
            weapons,
            specials: vec![],
            effects: vec![],
            effects_pool,
            background,
            next_uid,
            time: 0,
            game_over: false,
            paused: false,
            camera_x: 0.0,
            controllers,
            package_objects: package.objects.clone(),
            sound: {
                let mut s = crate::lf::soundpack::Soundpack::new();
                s.set_root(&package.root);
                if !package.sound_meta.is_null() {
                    s.load_meta_json(&package.sound_meta);
                }
                s
            },
            ai_brains,
            properties: package.properties.clone(),
            ui_panel: package.ui.get("panel").cloned(),
            winner_team: None,
            asset_root: package.root.clone(),
            tasks: vec![],
            rng_state: 0xC0FFEE_u64,
            overlay_msg: String::new(),
            overlay_ttl: 0,
            panel_remap: std::collections::HashMap::new(),
            f6_mode: false,
            ai_script_pool: package
                .data_list
                .AI
                .iter()
                .filter_map(|e| {
                    let f = e.file.rsplit('/').next().unwrap_or(&e.file);
                    let name = f.trim_end_matches(".js").trim_end_matches(".as");
                    if name.is_empty() { None } else { Some(name.to_string()) }
                })
                .collect(),
        })
    }

    /// F.LF match.game_state — identity snapshot for lockstep verify / TU dumps
    /// Shape: `{ time, "0": [x,y,z,hp,mp], "1": [...], ... }`
    pub fn game_state(&self) -> serde_json::Value {
        let mut d = serde_json::Map::new();
        d.insert("time".into(), serde_json::json!(self.time));
        for (i, c) in self.characters.iter().enumerate() {
            d.insert(
                i.to_string(),
                serde_json::json!([
                    c.base.ps.x as i32,
                    c.base.ps.y as i32,
                    c.base.ps.z as i32,
                    c.base.hp as i32,
                    c.base.mp as i32
                ]),
            );
        }
        serde_json::Value::Object(d)
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    /// F.LF match.TU_trans phase 1: transit all living objects
    pub fn tu_trans(&mut self) {
        for ch in &mut self.characters {
            ch.transit_phase();
        }
        for w in &mut self.weapons {
            if w.base.trans.wait == 0 {
                let _ = w.base.apply_transit();
            }
        }
        for s in &mut self.specials {
            if s.base.trans.wait == 0 {
                let _ = s.base.apply_transit();
            }
        }
    }

    /// Full TU — F.LF order: transit → process_tasks → TU → interactions → bg/sound/hp/gameover → AI
    pub fn tu(&mut self) {
        if self.paused || self.game_over {
            return;
        }
        self.time += 1;
        let bg_z = self.background.zboundary;
        let bg_w = self.background.width;

        // --- F.LF TU_trans: transit ---
        self.tu_trans();
        // --- process_tasks (spawns before TU physics of new objects) ---
        self.process_tasks();

        // --- emit TU for characters (human / pending AI from last tick) ---
        let ctrls = self.controllers.borrow();
        for ch in self.characters.iter_mut() {
            if ch.base.ai {
                // pending_ai_keys applied inside Character::tu
                ch.tu(None, bg_z, bg_w);
            } else if let Some(ci) = ch.base.controller_index {
                ch.tu(ctrls.get(ci), bg_z, bg_w);
            } else {
                ch.tu(None, bg_z, bg_w);
            }
        }
        drop(ctrls);

        // Chase ball targeting (F.LF specialattack chase_target) before specials TU continues in interactions
        self.update_special_chase_targets();

        // Teleport 400/401 on frame entry only (character.js states 400/401 `frame`)
        let n = self.characters.len();
        for i in 0..n {
            if !self.characters[i].base.statemem_frame_tu {
                continue;
            }
            let st = self.characters[i].base.state();
            if st != 400 && st != 401 {
                continue;
            }
            let team = self.characters[i].base.team;
            let enemies = crate::lf::scene::query_characters(
                &self.characters,
                i,
                crate::lf::scene::QueryOpts {
                    team: None,
                    not_team: Some(team),
                    sort_distance: true,
                    reverse: false,
                },
            );
            let allies = crate::lf::scene::query_characters(
                &self.characters,
                i,
                crate::lf::scene::QueryOpts {
                    team: Some(team),
                    not_team: None,
                    sort_distance: true,
                    reverse: true,
                },
            );
            let ne = enemies.first().map(|&j| (self.characters[j].base.ps.x, self.characters[j].base.ps.z));
            let fa = allies.first().map(|&j| (self.characters[j].base.ps.x, self.characters[j].base.ps.z));
            self.characters[i].apply_teleport_targets(ne, fa);
        }

        // Rudolf transform / revert (approximate: swap object data id while keeping uid/team/hp)
        self.process_transforms();

        // pending ice broken effects
        let mut broken_spawns: Vec<(f64, f64, f64, i32)> = vec![];
        for ch in &mut self.characters {
            if ch.pending_broken_effect != 0 {
                broken_spawns.push((
                    ch.base.ps.x,
                    ch.base.ps.y - 30.0,
                    ch.base.ps.z,
                    ch.pending_broken_effect,
                ));
                ch.pending_broken_effect = 0;
            }
        }
        for (x, y, z, _fid) in broken_spawns {
            self.spawn_broken(x, y, z);
        }

        // frame sounds (sound.tu at end of TU_trans)
        for ch in &mut self.characters {
            if let Some(path) = ch.base.take_sound() {
                self.sound.play(&path);
            }
        }

        for w in &mut self.weapons {
            w.tu(bg_z, bg_w);
        }
        for s in &mut self.specials {
            s.tu(bg_z, bg_w);
        }
        self.specials.retain(|s| !s.base.removed && s.base.frame.n < 1000);
        for e in &mut self.effects {
            e.base.physics_tu(bg_z, bg_w);
        }
        self.effects.retain(|e| !e.base.removed && !e.base.dead);
        self.effects_pool.tu(bg_z, bg_w);

        self.super_punch_scope();
        self.process_hits();
        self.process_catches();
        self.process_throws();
        self.special_hits();
        self.special_vs_special();
        self.spawn_special_opoints();
        self.drink_weapons();
        self.whirlwind_itr();
        self.itr_kind2_pick();
        self.burn_broken_fx();
        self.bpoint_blood();
        self.weapon_hits();
        self.weapon_land_crush();
        self.char_hits_specials();
        self.spawn_opoints();
        self.pick_weapons();
        self.flush_broken_effects();
        self.flush_visual_effects();
        // late tasks from opoints during TU
        self.process_tasks();

        // background / sound / gameover (F.LF TU_trans tail)
        self.update_camera();
        self.background.tu(self.time);
        self.sound.tu();
        self.check_game_over();
        self.apply_leaving();

        // AI at end of TU (F.LF) — keys apply next TU via pending_ai_keys
        self.run_ai_end_of_tu(bg_z, bg_w);
    }

    /// F.LF AI_frameskip=3 at end of TU_trans
    fn run_ai_end_of_tu(&mut self, bg_z: (f64, f64), bg_w: f64) {
        if self.time % 3 != 0 {
            return;
        }
        while self.ai_brains.len() < self.characters.len() {
            self.ai_brains.push(crate::lf::ai::AiBrain::default());
        }
        let snapshot: Vec<(i32, f64, f64, bool)> = self
            .characters
            .iter()
            .map(|c| {
                (
                    c.base.team,
                    c.base.ps.x,
                    c.base.ps.z,
                    c.base.removed || c.base.hp <= 0.0,
                )
            })
            .collect();
        let enemy_snap: Vec<(u32, f64, f64, f64, i32)> = self
            .characters
            .iter()
            .map(|c| {
                (
                    c.base.uid,
                    c.base.ps.x,
                    c.base.ps.z,
                    c.base.ps.y,
                    c.base.state(),
                )
            })
            .collect();
        let weapon_snap: Vec<(u32, f64, f64, bool, bool)> = self
            .weapons
            .iter()
            .map(|w| {
                (
                    w.base.uid,
                    w.base.ps.x,
                    w.base.ps.z,
                    w.held || w.base.removed,
                    w.base.obj_type == "drink",
                )
            })
            .collect();
        let time = self.time;
        let asset_root = self.asset_root.clone();
        let bg_w_val = bg_w;
        for i in 0..self.characters.len() {
            if !self.characters[i].base.ai || self.characters[i].base.removed {
                continue;
            }
            let my_team = self.characters[i].base.team;
            let holding = self.characters[i].hold_weapon.is_some();
            let enemies: Vec<(u32, f64, f64, f64, i32)> = enemy_snap
                .iter()
                .enumerate()
                .filter(|(j, e)| {
                    *j < snapshot.len()
                        && snapshot[*j].0 != my_team
                        && !snapshot[*j].3
                        && e.0 != self.characters[i].base.uid
                })
                .map(|(_, e)| *e)
                .collect();
            let hold_heavy = holding && self.characters[i].base.hold_type == "heavyweapon";
            let hold_drink = holding && self.characters[i].base.hold_type == "drink";
            let hold_uid = self.characters[i]
                .hold_weapon
                .map(|u| u as i32)
                .unwrap_or(-1);
            let mut keys = vec![];
            let mut used_script = false;
            if self.ai_brains[i].prefer_script {
                let self_json = crate::lf::ai::snapshot_json(
                    &self.characters[i].base,
                    &self.characters[i].base.hold_type,
                    self.characters[i].hold_weapon_oid,
                    hold_uid,
                    self.characters[i].catch_counter,
                    bg_w_val,
                    bg_z.0,
                    bg_z.1,
                );
                let others_json: Vec<serde_json::Value> = enemies
                    .iter()
                    .take(8)
                    .map(|e| {
                        serde_json::json!({
                            "x": e.1, "y": e.3, "z": e.2, "vx": 0, "vy": 0, "vz": 0,
                            "facing": 1, "hp": 500, "mp": 200, "fall": 0,
                            "team": 0, "id": 0, "uid": e.0, "state": e.4, "frame": 0,
                            "hold_type": "", "hold_oid": 0, "hold_uid": -1,
                            "blink": false, "effect_timeout": 0, "catch_counter": 0,
                            "bg_w": bg_w_val, "bg_z": [bg_z.0, bg_z.1],
                        })
                    })
                    .collect();
                let oj = serde_json::Value::Array(others_json).to_string();
                let script = self.ai_brains[i].script_name.clone();
                if let Some(k) =
                    crate::lf::ai::try_script_keys_sync(&asset_root, &script, &self_json, &oj)
                {
                    keys = k;
                    used_script = true;
                }
            }
            if !used_script {
                let mut cfg = std::collections::HashMap::new();
                for k in ["up", "down", "left", "right", "def", "jump", "att"] {
                    cfg.insert(k.to_string(), k.to_string());
                }
                let mut ac = Controller::new_keyboard(cfg);
                crate::lf::ai::ai_fill(
                    &mut self.ai_brains[i],
                    &self.characters[i].base,
                    &enemies,
                    &weapon_snap,
                    holding,
                    hold_heavy,
                    hold_drink,
                    &mut ac,
                    time,
                );
                for a in ["up", "down", "left", "right", "def", "jump", "att"] {
                    if ac.is_pressed(a) {
                        keys.push(a.to_string());
                    }
                }
            }
            self.characters[i].queue_ai_keys(keys);
        }
    }

    /// F.LF background.leaving — destroy specials/weapons far off-screen
    fn apply_leaving(&mut self) {
        let w = self.background.width;
        let margin = 200.0;
        for s in &mut self.specials {
            if self.background.leaving(s.base.ps.x, margin) {
                s.base.trans_frame(1000, 5);
                s.base.removed = true;
            }
        }
        for wp in &mut self.weapons {
            if !wp.held && (wp.base.ps.x < -margin || wp.base.ps.x > w + margin) {
                wp.base.removed = true;
            }
        }
        let _ = w;
    }

    fn flush_visual_effects(&mut self) {
        let mut pts = vec![];
        for ch in &mut self.characters {
            if ch.base.pending_vfx_num != 0 {
                let n = ch.base.pending_vfx_num;
                ch.base.pending_vfx_num = 0;
                pts.push((ch.base.ps.x, ch.base.ps.y - 40.0, ch.base.ps.z, 300 + n));
            }
        }
        for (x, y, z, oid) in pts {
            if let Some(data) = self.package_objects.get(&oid).cloned().or_else(|| self.package_objects.get(&301).cloned()) {
                let eo = crate::lf::effect::EffectObj::new(self.next_uid, data, x, y, z);
                self.next_uid += 1;
                self.effects.push(eo);
            }
        }
    }

    /// Drain LO.pending_broken_* into 320 debris (brokeneffect_create)
    fn flush_broken_effects(&mut self) {
        let mut jobs: Vec<(f64, f64, f64, i32, i32)> = vec![];
        for ch in &mut self.characters {
            if ch.base.pending_broken_num > 0 {
                jobs.push((
                    ch.base.ps.x,
                    ch.base.ps.y,
                    ch.base.ps.z,
                    ch.base.pending_broken_id,
                    ch.base.pending_broken_num,
                ));
                ch.base.pending_broken_id = 0;
                ch.base.pending_broken_num = 0;
            }
        }
        for (x, y, z, _fid, num) in jobs {
            let n = num.clamp(1, 12);
            for i in 0..n {
                if let Some(data) = self.package_objects.get(&320).cloned() {
                    let mut eo = crate::lf::effect::EffectObj::new(
                        self.next_uid,
                        data,
                        x + (i as f64 - n as f64 * 0.5) * 6.0,
                        y - 20.0 + (i % 3) as f64 * 4.0,
                        z,
                    );
                    self.next_uid += 1;
                    eo.base.ps.vx = (i as f64 - 4.0) * 1.5;
                    eo.base.ps.vy = -2.0 - (i % 4) as f64;
                    self.effects.push(eo);
                } else {
                    self.spawn_broken(x, y, z);
                    break;
                }
            }
        }
    }

    fn process_hits(&mut self) {
        let n = self.characters.len();
        // i, j, injury, fall, dvx, dvy, arest, effect, kind, vrest
        let mut events: Vec<(usize, usize, f64, f64, f64, f64, i32, i32, i32, i32, f64)> = vec![];

        for i in 0..n {
            if self.characters[i].base.removed || self.characters[i].base.arest > 0 {
                continue;
            }
            let Some(frame) = self.characters[i].base.frame_data().cloned() else { continue };
            let itrs = Mech::itr_volumes(
                &self.characters[i].base.ps,
                self.characters[i].base.facing,
                &frame,
            );
            for j in 0..n {
                if i == j || self.characters[j].base.removed {
                    continue;
                }
                if self.characters[i].base.team == self.characters[j].base.team
                    && self.characters[i].base.team != 0
                {
                    continue;
                }
                // victim-side vrest: cannot be hit again by this attacker uid yet
                let att_uid = self.characters[i].base.uid;
                if !self.characters[j].base.itr_vrest_test(att_uid) {
                    continue;
                }
                let Some(vframe) = self.characters[j].base.frame_data().cloned() else { continue };
                let bdys = Mech::body_volumes(
                    &self.characters[j].base.ps,
                    self.characters[j].base.facing,
                    &vframe,
                );
                for (vol, itr) in &itrs {
                    // Combat itr kinds; 1 catch / 2 pick handled elsewhere; 9 shield on specials
                    if !matches!(itr.kind, 0 | 3 | 4 | 5 | 8 | 10 | 11 | 15 | 16) {
                        continue;
                    }
                    for b in &bdys {
                        if vol.intersects(b) {
                            let injury = if itr.injury != 0.0 { itr.injury } else { 20.0 };
                            let fall = if itr.fall != 0.0 { itr.fall } else { global::DEFAULT_FALL };
                            let vrest = if itr.vrest != 0 { itr.vrest } else { global::DEFAULT_VREST };
                            events.push((
                                i,
                                j,
                                injury,
                                fall,
                                vol.vx,
                                itr.dvy,
                                itr.arest,
                                itr.effect,
                                itr.kind,
                                vrest,
                                itr.bdefend,
                            ));
                        }
                    }
                }
            }
        }

        let mut drops: Vec<(u32, f64, f64)> = vec![]; // char uid, vx, vy for weapon drop
        for (i, j, injury, fall, dvx, dvy, arest, eff, ikind, vrest, bdef) in events {
            let att_uid = self.characters[i].base.uid;
            let att_x = self.characters[i].base.ps.x;
            let facing = self.characters[i].base.facing;

            self.characters[i].base.itr_arest_update(arest);
            if !crate::lf::character_ids::state3_hit_stop(&mut self.characters[i]) {
                let hs = global::DEFAULT_HIT_STOP;
                self.characters[i].base.trans.inc_wait(hs, 20, 1);
                self.characters[i].base.trans.wait = self.characters[i].base.trans.wait.max(hs);
            }

            // attacker tracks victim (legacy) + victim tracks attacker (F.LF itr_vrest)
            let vic_uid = self.characters[j].base.uid;
            self.characters[i].base.vrest.insert(vic_uid, vrest);
            self.characters[j].base.itr_vrest_update(att_uid, vrest);

            // F.LF: defend only if attacked from front:
            // (att.x > vic.x) === (vic facing right)
            let defending = self.characters[j].base.state() == 7;
            let mut inj = injury;
            let vic_x = self.characters[j].base.ps.x;
            let vic_face_right = self.characters[j].base.facing > 0;
            let attacked_from_front = (att_x > vic_x) == vic_face_right;
            if defending && attacked_from_front {
                inj *= global::DEFEND_INJURY_FACTOR;
                self.characters[j].base.bdefend += if bdef != 0.0 { bdef } else { injury };
                // absorb some dvx (handled by reduced fall via continue on effective defend)
                if self.characters[j].base.bdefend <= global::DEFEND_BREAK_LIMIT {
                    self.characters[j].base.trans_frame(111, 8);
                    self.characters[j]
                        .base
                        .effect_create(0, 3, dvx * 0.3, 0.0);
                    self.sound.play("1/002");
                    // still apply reduced injury
                    let (_d, _) = self.characters[j].base.injure(
                        inj,
                        0.0,
                        dvx * 0.2 * facing as f64,
                        0.0,
                        att_x,
                        0,
                        ikind,
                    );
                    let _ = facing;
                    continue;
                }
                self.characters[j].base.trans_frame(112, 12);
            } else if defending {
                // hit from behind — full injury, lose defend
                self.characters[j].base.bdefend = 45.0;
            }

            let dvy_use = if dvy != 0.0 { dvy } else { global::DEFAULT_FALL_DVY };
            let mut inj2 = inj;
            let mut eff2 = eff;
            if ikind == 8 {
                inj2 = -inj.abs().max(10.0);
            }
            // kind 4 / effect ice mapping
            if ikind == 4 && eff2 <= 0 {
                eff2 = 3;
            }
            if (ikind == 3 || ikind == 5) && eff2 <= 0 {
                eff2 = 2;
            }
            // kind 15 whirlwind_force
            if ikind == 15 {
                let az = self.characters[i].base.ps.z;
                self.characters[j].base.whirlwind_force(att_x, az);
                inj2 = inj2.max(5.0);
            }
            // kind 10/11 flute
            if ikind == 10 || ikind == 11 {
                self.characters[j].base.flute_force();
            }

            let (drop_w, snd_eff) = self.characters[j].base.injure(
                inj2,
                if ikind == 8 { 0.0 } else { fall },
                dvx * facing as f64,
                if ikind == 8 { 0.0 } else { dvy_use },
                att_x,
                eff2,
                ikind,
            );
            if drop_w {
                let uid = self.characters[j].base.uid;
                drops.push((uid, dvx * facing as f64, dvy_use));
            }
            if self.characters[j].base.hp <= 0.0 && !self.characters[j].base.dead {
                self.characters[i].base.kills += 1;
            }
            if bdef > 0.0 {
                self.characters[j].base.bdefend += bdef;
            }
            match snd_eff {
                2 | 20 | 21 | 22 | 23 => self.sound.play("1/070"),
                3 | 30 => {
                    if self.characters[j].base.state() == 13 {
                        self.sound.play("1/066");
                    } else {
                        self.sound.play("1/065");
                    }
                }
                0 | 1 => self.sound.play("1/002"),
                _ => {}
            }

            let (bx, by, bz) = (
                self.characters[j].base.ps.x,
                self.characters[j].base.ps.y - 40.0,
                self.characters[j].base.ps.z,
            );
            let mut try_ids = vec![301i32, 300];
            if eff2 == 3 || ikind == 4 || ikind == 16 {
                try_ids.insert(0, 302);
            }
            if eff2 == 2 || ikind == 3 || ikind == 5 {
                try_ids.insert(0, 303);
            }
            if eff2 > 0 {
                try_ids.insert(0, crate::lf::effect::effect_id_from_num(eff2));
            }
            let mut spawned = self.effects_pool.create(bx, by, bz, 0);
            if !spawned {
                for try_id in try_ids {
                    if let Some(data) = self.package_objects.get(&try_id).cloned() {
                        let eo = crate::lf::effect::EffectObj::new(self.next_uid, data, bx, by, bz);
                        self.next_uid += 1;
                        self.effects.push(eo);
                        spawned = true;
                        break;
                    }
                }
            }
            let _ = spawned;
        }
        for (cuid, vx, vy) in drops {
            self.drop_char_weapon(cuid, vx, vy, 0.0);
        }
    }

    fn drop_char_weapon(&mut self, char_uid: u32, vx: f64, vy: f64, vz: f64) {
        let Some(ch) = self.characters.iter_mut().find(|c| c.base.uid == char_uid) else {
            return;
        };
        let Some(wid) = ch.hold_weapon.take() else {
            return;
        };
        ch.hold_weapon_oid = 0;
        ch.base.hold_type.clear();
        if ch.base.holding_uid == Some(wid) {
            ch.base.holding_uid = None;
        }
        if let Some(w) = self.weapons.iter_mut().find(|w| w.base.uid == wid) {
            w.drop(vx, vy, vz);
        }
    }

    /// Super punch scope: if stand/walk att would hit a victim whose attack frames expose itr kind 6
    fn super_punch_scope(&mut self) {
        let n = self.characters.len();
        for i in 0..n {
            if self.characters[i].base.removed {
                continue;
            }
            if !matches!(self.characters[i].base.state(), 0 | 1) {
                continue;
            }
            // probe frames 72 and 73 itr volumes (LF character.js)
            let mut scope_hit = false;
            for probe in [72i32, 73] {
                let Some(frame) = self.characters[i].base.data.frames.get(&probe).cloned() else {
                    continue;
                };
                let itrs = Mech::itr_volumes(
                    &self.characters[i].base.ps,
                    self.characters[i].base.facing,
                    &frame,
                );
                for j in 0..n {
                    if i == j || self.characters[j].base.removed {
                        continue;
                    }
                    if self.characters[i].base.team == self.characters[j].base.team
                        && self.characters[i].base.team != 0
                    {
                        continue;
                    }
                    // victim currently has itr kind 6 on active frame (dance of pain / special hurtbox)
                    let Some(vframe) = self.characters[j].base.frame_data().cloned() else {
                        continue;
                    };
                    let has_k6 = vframe.itr.iter().any(|it| it.kind == 6);
                    if !has_k6 {
                        continue;
                    }
                    let bdys = Mech::body_volumes(
                        &self.characters[j].base.ps,
                        self.characters[j].base.facing,
                        &vframe,
                    );
                    for (vol, _) in &itrs {
                        for b in &bdys {
                            if vol.intersects(b) {
                                scope_hit = true;
                                break;
                            }
                        }
                    }
                }
            }
            self.characters[i].want_super_punch = scope_hit;
        }
    }

    /// itr kind 16 whirlwind — pull victims slightly toward attacker special/character
    fn whirlwind_itr(&mut self) {
        let n = self.characters.len();
        let mut pulls: Vec<(usize, f64, f64)> = vec![];
        for i in 0..n {
            if self.characters[i].base.removed {
                continue;
            }
            let Some(frame) = self.characters[i].base.frame_data().cloned() else {
                continue;
            };
            let itrs = Mech::itr_volumes(
                &self.characters[i].base.ps,
                self.characters[i].base.facing,
                &frame,
            );
            for j in 0..n {
                if i == j || self.characters[j].base.removed {
                    continue;
                }
                let Some(vframe) = self.characters[j].base.frame_data().cloned() else {
                    continue;
                };
                let bdys = Mech::body_volumes(
                    &self.characters[j].base.ps,
                    self.characters[j].base.facing,
                    &vframe,
                );
                for (vol, itr) in &itrs {
                    if itr.kind != 16 {
                        continue;
                    }
                    for b in &bdys {
                        if vol.intersects(b) {
                            let ax = self.characters[i].base.ps.x;
                            let az = self.characters[i].base.ps.z;
                            let dx = (ax - self.characters[j].base.ps.x).signum() * 2.5;
                            let dz = (az - self.characters[j].base.ps.z).signum() * 1.2;
                            pulls.push((j, dx, dz));
                        }
                    }
                }
            }
        }
        // specials whirlwind
        for sp in &self.specials {
            if sp.base.removed {
                continue;
            }
            let Some(frame) = sp.base.frame_data().cloned() else {
                continue;
            };
            let itrs = Mech::itr_volumes(&sp.base.ps, sp.base.facing, &frame);
            for j in 0..n {
                if self.characters[j].base.removed {
                    continue;
                }
                let Some(vframe) = self.characters[j].base.frame_data().cloned() else {
                    continue;
                };
                let bdys = Mech::body_volumes(
                    &self.characters[j].base.ps,
                    self.characters[j].base.facing,
                    &vframe,
                );
                for (vol, itr) in &itrs {
                    if itr.kind != 16 {
                        continue;
                    }
                    for b in &bdys {
                        if vol.intersects(b) {
                            let dx = (sp.base.ps.x - self.characters[j].base.ps.x).signum() * 3.0;
                            let dz = (sp.base.ps.z - self.characters[j].base.ps.z).signum() * 1.5;
                            pulls.push((j, dx, dz));
                        }
                    }
                }
            }
        }
        for (j, dx, dz) in pulls {
            self.characters[j].base.ps.x += dx;
            self.characters[j].base.ps.z += dz;
            self.characters[j].base.ps.vx *= 0.85;
        }
    }

    /// Drink weapon (type drink): periodic heal while held on sip frames
    fn drink_weapons(&mut self) {
        let mut heals: Vec<(usize, f64)> = vec![];
        for (ci, ch) in self.characters.iter().enumerate() {
            if ch.base.hold_type != "drink" {
                continue;
            }
            let Some(wid) = ch.hold_weapon else {
                continue;
            };
            // sip animation frames often 55-58 or weaponact on light drink
            if matches!(ch.base.frame.n, 55 | 56 | 57 | 58 | 110) || ch.base.state() == 7 {
                heals.push((ci, 3.0));
            }
            let _ = wid;
        }
        for (ci, amt) in heals {
            let ch = &mut self.characters[ci];
            ch.drink_sips += 1;
            if ch.drink_sips % 4 == 0 {
                ch.base.hp = (ch.base.hp + amt).min(ch.base.hp_full);
                ch.base.mp = (ch.base.mp + amt * 0.5).min(ch.base.mp_full);
            }
        }
    }


    /// itr kind 1 — catch (walking frames)
    fn process_catches(&mut self) {
        let n = self.characters.len();
        let mut catches: Vec<(usize, usize, i32, i32)> = vec![]; // att, vic, catchingact, caughtact
        for i in 0..n {
            if self.characters[i].base.removed { continue; }
            let Some(frame) = self.characters[i].base.frame_data().cloned() else { continue };
            let itrs = Mech::itr_volumes(&self.characters[i].base.ps, self.characters[i].base.facing, &frame);
            for j in 0..n {
                if i == j || self.characters[j].base.removed { continue; }
                if self.characters[i].base.team == self.characters[j].base.team && self.characters[i].base.team != 0 { continue; }
                let Some(vframe) = self.characters[j].base.frame_data().cloned() else { continue };
                let vstate = self.characters[j].base.state();
                let bdys = Mech::body_volumes(&self.characters[j].base.ps, self.characters[j].base.facing, &vframe);
                for (vol, itr) in &itrs {
                    // kind 1 normal catch; kind 3 super catch (dance of pain / special)
                    if itr.kind != 1 && itr.kind != 3 {
                        continue;
                    }
                    if itr.kind == 1 && !matches!(vstate, 0 | 1 | 8 | 11 | 12 | 16) {
                        continue;
                    }
                    if itr.kind == 3 && vstate != 16 && vstate != 12 {
                        continue;
                    }
                    for b in &bdys {
                        if vol.intersects(b) {
                            let ca = itr.catchingact.first().copied().unwrap_or(120);
                            let co = itr.caughtact.first().copied().unwrap_or(130);
                            catches.push((i, j, ca, co));
                        }
                    }
                }
            }
        }
        for (i, j, ca, co) in catches {
            self.characters[i].base.trans_frame(ca, 15);
            self.characters[j].base.trans_frame(co, 15);
            self.characters[i].base.holding_uid = Some(self.characters[j].base.uid);
            self.characters[j].base.held_by = Some(self.characters[i].base.uid);
            self.characters[i].catch_injury_pending = true;
            // Rudolf: remember caught character id for transform
            self.characters[i].transform_caught_id = self.characters[j].base.id;
            self.characters[i].transform_target_uid = Some(self.characters[j].base.uid);
            self.sound.play("1/021");
        }
        // sync caught position to catcher cpoint; apply cpoint injury once per catch TU
        let mut injuries: Vec<(u32, f64)> = vec![];
        let mut vactions: Vec<(u32, i32)> = vec![];
        for i in 0..n {
            if let Some(vid) = self.characters[i].base.holding_uid {
                let facing = self.characters[i].base.facing;
                let (x, y, z, injury, vaction, apply_inj) = {
                    if let Some(fd) = self.characters[i].base.frame_data() {
                        if let Some(cp) = &fd.cpoint {
                            let x = self.characters[i].base.ps.x
                                + (cp.x - fd.centerx) * facing as f64;
                            let y = self.characters[i].base.ps.y + (cp.y - fd.centery);
                            let z = self.characters[i].base.ps.z;
                            let apply = cp.injury > 0.0
                                && self.characters[i].base.frame.wait_left
                                    == self.characters[i].base.frame_data().map(|f| f.wait).unwrap_or(-1);
                            (x, y, z, cp.injury, cp.vaction, apply)
                        } else {
                            continue;
                        }
                    } else {
                        continue;
                    }
                };
                if apply_inj {
                    self.characters[i].catch_injury_pending = false;
                    self.characters[i].base.trans.inc_wait(1, 10, 99);
                    injuries.push((vid, injury));
                }
                if vaction != 0 {
                    vactions.push((vid, vaction));
                }
                if let Some(ch) = self.characters.iter_mut().find(|c| c.base.uid == vid) {
                    // F.LF caught_b then coincide
                    ch.caught_b(x, y, facing, 0.0);
                    ch.base.ps.x = x;
                    ch.base.ps.y = y;
                    ch.base.ps.z = z;
                    ch.base.ps.vx = 0.0;
                    ch.base.ps.vy = 0.0;
                    ch.base.ps.vz = 0.0;
                    ch.base.facing = -facing;
                }
                let fn_ = self.characters[i].base.frame.n;
                if !(120..=150).contains(&fn_) && self.characters[i].base.state() != 9 {
                    let vid = self.characters[i].base.holding_uid.take();
                    if let Some(vid) = vid {
                        if let Some(ch) = self.characters.iter_mut().find(|c| c.base.uid == vid) {
                            ch.caught_release();
                        }
                    }
                }
            }
        }
        for (vid, inj) in injuries {
            if let Some(ch) = self.characters.iter_mut().find(|c| c.base.uid == vid) {
                ch.base.hp -= inj;
                ch.base.injury_total += inj;
            }
        }
        for (vid, va) in vactions {
            if let Some(ch) = self.characters.iter_mut().find(|c| c.base.uid == vid) {
                ch.base.trans_frame(va, 22);
            }
        }
    }


    fn process_throws(&mut self) {
        let ctrls = self.controllers.borrow();
        let mut releases: Vec<(u32, u32, f64, f64, f64)> = vec![]; // catcher, victim, vx,vy,vz
        for ch in &self.characters {
            if ch.base.removed { continue; }
            let Some(vid) = ch.base.holding_uid else { continue };
            // throw frames 121-122 or att while catching
            let throwing = matches!(ch.base.frame.n, 121 | 122 | 123 | 124 | 125)
                || (ch.base.state() == 9 && ch.base.controller_index.and_then(|i| ctrls.get(i)).map(|c| c.is_pressed("att")).unwrap_or(false));
            if !throwing { continue; }
            if let Some(fd) = ch.base.frame_data() {
                if let Some(cp) = &fd.cpoint {
                    let vx = if cp.throwvx != 0.0 { cp.throwvx * ch.base.facing as f64 } else { 12.0 * ch.base.facing as f64 };
                    let vy = if cp.throwvy != 0.0 { cp.throwvy } else { -8.0 };
                    let vz = cp.throwvz;
                    releases.push((ch.base.uid, vid, vx, vy, vz));
                    if cp.throwinjury > 0.0 {
                        // applied below
                    }
                } else {
                    releases.push((ch.base.uid, vid, 12.0 * ch.base.facing as f64, -8.0, 0.0));
                }
            }
        }
        drop(ctrls);
        for (cuid, vid, vx, vy, vz) in releases {
            let mut throw_inj = global::DEFAULT_THROW_INJURY;
            if let Some(ch) = self.characters.iter_mut().find(|c| c.base.uid == cuid) {
                if let Some(fd) = ch.base.frame_data() {
                    if let Some(cp) = &fd.cpoint {
                        if cp.throwinjury > 0.0 {
                            throw_inj = cp.throwinjury;
                        }
                    }
                }
                ch.base.holding_uid = None;
            }
            if let Some(vic) = self.characters.iter_mut().find(|c| c.base.uid == vid) {
                vic.base.held_by = None;
                vic.base.ps.vx = vx;
                vic.base.ps.vy = vy;
                vic.base.ps.vz = vz;
                // impulse: move one step
                vic.base.ps.x += vx;
                vic.base.ps.y += vy * 2.0;
                vic.base.ps.z += vz;
                vic.base.trans_frame(180, 15); // fall
                vic.caught_throwinjury = throw_inj;
            }
        }
    }

    fn special_hits(&mut self) {
        // specials hit characters
        let mut events = vec![];
        for (si, sp) in self.specials.iter().enumerate() {
            if sp.base.removed { continue; }
            let Some(frame) = sp.base.frame_data().cloned() else { continue };
            let itrs = Mech::itr_volumes(&sp.base.ps, sp.base.facing, &frame);
            for (ci, ch) in self.characters.iter().enumerate() {
                if ch.base.removed { continue; }
                if sp.base.team != 0 && ch.base.team == sp.base.team { continue; }
                let Some(vf) = ch.base.frame_data().cloned() else { continue };
                let bdys = Mech::body_volumes(&ch.base.ps, ch.base.facing, &vf);
                for (vol, itr) in &itrs {
                    if itr.kind == 9 {
                        // john shield etc depletes type3 instantly when applied TO special — handled inverse
                        continue;
                    }
                    if itr.kind != 0 && itr.kind != 3 { continue; }
                    for b in &bdys {
                        if vol.intersects(b) {
                            events.push((si, ci, itr.injury.max(15.0), itr.fall, vol.vx, itr.dvy, itr.effect, itr.kind));
                        }
                    }
                }
            }
        }
        let mut drops = vec![];
        for (si, ci, inj, fall, dvx, dvy, eff, ikind) in events {
            let facing = self.specials[si].base.facing;
            let att_x = self.specials[si].base.ps.x;
            // defend reduces special damage when facing the projectile
            if self.characters[ci].base.state() == 7 {
                let face_ball = (att_x > self.characters[ci].base.ps.x) == (self.characters[ci].base.facing < 0)
                    || (att_x < self.characters[ci].base.ps.x) == (self.characters[ci].base.facing > 0);
                // facing toward attacker x
                let toward = (self.characters[ci].base.facing > 0 && att_x > self.characters[ci].base.ps.x)
                    || (self.characters[ci].base.facing < 0 && att_x < self.characters[ci].base.ps.x);
                if toward {
                    self.characters[ci].base.bdefend += inj * 0.5;
                    if self.characters[ci].base.bdefend < global::DEFEND_BREAK_LIMIT {
                        self.specials[si].base.trans_frame(1000, 5);
                        continue;
                    }
                }
                let _ = face_ball;
            }
            if self.characters[ci].base.state() == 7 {
                let toward = (self.characters[ci].base.facing > 0
                    && att_x > self.characters[ci].base.ps.x)
                    || (self.characters[ci].base.facing < 0
                        && att_x < self.characters[ci].base.ps.x);
                if toward {
                    self.characters[ci].base.bdefend += inj * 0.5;
                    if self.characters[ci].base.bdefend < global::DEFEND_BREAK_LIMIT {
                        self.specials[si].base.trans_frame(1000, 5);
                        continue;
                    }
                }
            }
            let next_frame = self.specials[si]
                .base
                .frame_data()
                .map(|f| if f.next > 0 { f.next } else { 1000 })
                .unwrap_or(1000);
            let (drop_w, _) = self.characters[ci].base.injure(
                inj,
                fall,
                dvx * facing as f64,
                if dvy != 0.0 { dvy } else { -3.0 },
                att_x,
                eff,
                ikind,
            );
            if drop_w {
                drops.push(self.characters[ci].base.uid);
            }
            self.specials[si].base.trans_frame(next_frame, 5);
        }
        for uid in drops {
            self.drop_char_weapon(uid, 4.0, -3.0, 0.0);
        }
    }


    fn weapon_hits(&mut self) {
        let mut ev = vec![];
        for (wi, w) in self.weapons.iter().enumerate() {
            if w.held || w.base.removed { continue; }
            // only when moving fast in air / thrown
            let spd = (w.base.ps.vx*w.base.ps.vx + w.base.ps.vy*w.base.ps.vy).sqrt();
            if spd < 3.0 && w.base.ps.y >= 0.0 { continue; }
            let Some(frame) = w.base.frame_data().cloned() else { continue };
            let itrs = Mech::itr_volumes(&w.base.ps, w.base.facing, &frame);
            for (ci, ch) in self.characters.iter().enumerate() {
                if ch.base.removed { continue; }
                if w.base.team != 0 && w.base.team == ch.base.team { continue; }
                let Some(vf) = ch.base.frame_data().cloned() else { continue };
                let bdys = Mech::body_volumes(&ch.base.ps, ch.base.facing, &vf);
                for (vol, itr) in &itrs {
                    if itr.kind != 0 && itr.kind != 3 { continue; }
                    for b in &bdys {
                        if vol.intersects(b) {
                            ev.push((wi, ci, itr.injury.max(15.0), itr.fall, vol.vx, itr.dvy));
                        }
                    }
                }
            }
        }
        let mut drops = vec![];
        for (wi, ci, inj, fall, dvx, dvy) in ev {
            let facing = self.weapons[wi].base.facing;
            let att_x = self.weapons[wi].base.ps.x;
            let (drop_w, _) = self.characters[ci].base.injure(
                inj,
                fall,
                dvx * facing as f64,
                if dvy != 0.0 { dvy } else { -4.0 },
                att_x,
                0,
                0,
            );
            if drop_w {
                drops.push(self.characters[ci].base.uid);
            }
            // weapon rebound
            self.weapons[wi].base.ps.vx *= -0.4;
            self.weapons[wi].base.ps.vy = -2.0;
            // chance to break light weapons on hard hit
            if self.weapons[wi].light && inj > 30.0 {
                let (x, y, z) = (
                    self.weapons[wi].base.ps.x,
                    self.weapons[wi].base.ps.y,
                    self.weapons[wi].base.ps.z,
                );
                self.weapons[wi].die();
                self.spawn_broken(x, y, z);
            }
        }
        for uid in drops {
            self.drop_char_weapon(uid, 3.0, -2.0, 0.0);
        }
    }

    /// Heavy weapon falling onto a character (weapon_drop_hurt lite)
    fn weapon_land_crush(&mut self) {
        let mut hits: Vec<(usize, f64)> = vec![];
        for w in &self.weapons {
            if w.held || w.base.removed || !w.heavy {
                continue;
            }
            if w.base.ps.y >= -8.0 && w.base.ps.vy > 2.0 {
                for (ci, ch) in self.characters.iter().enumerate() {
                    if ch.base.removed {
                        continue;
                    }
                    let dx = (w.base.ps.x - ch.base.ps.x).abs();
                    let dz = (w.base.ps.z - ch.base.ps.z).abs();
                    if dx < 28.0 && dz < 14.0 && ch.base.ps.y >= -5.0 {
                        hits.push((ci, 15.0));
                    }
                }
            }
        }
        for (ci, inj) in hits {
            let ax = self.characters[ci].base.ps.x;
            let _ = self.characters[ci]
                .base
                .injure(inj, 10.0, 0.0, -2.0, ax, 0, 0);
        }
    }

    /// F.LF create_transform_character — queue data swap (process_transforms applies)
    pub fn create_transform_character(&mut self, idx: usize, new_id: i32) {
        if idx < self.characters.len() {
            self.characters[idx].transform_target_id = new_id;
            self.characters[idx].pending_transform = true;
        }
    }

    /// Swap character data to transform target id (Rudolf) or back to 5.
    fn process_transforms(&mut self) {
        let mut jobs: Vec<(usize, i32, bool)> = vec![]; // idx, new_id, is_revert
        for (i, ch) in self.characters.iter().enumerate() {
            if ch.pending_revert_transform {
                jobs.push((i, 5, true));
            } else if ch.pending_transform && ch.transform_target_id != 0 {
                jobs.push((i, ch.transform_target_id, false));
            } else if ch.pending_transform && ch.transform_caught_id != 0 {
                jobs.push((i, ch.transform_caught_id, false));
            }
        }
        for (i, new_id, is_revert) in jobs {
            let Some(data) = self.package_objects.get(&new_id).cloned() else {
                self.characters[i].pending_transform = false;
                self.characters[i].pending_revert_transform = false;
                continue;
            };
            let ch = &mut self.characters[i];
            let hp = ch.base.hp;
            let mp = ch.base.mp;
            let team = ch.base.team;
            let uid = ch.base.uid;
            let x = ch.base.ps.x;
            let y = ch.base.ps.y;
            let z = ch.base.ps.z;
            let facing = ch.base.facing;
            let ai = ch.base.ai;
            let ci = ch.base.controller_index;
            let name = ch.base.name.clone();
            let was_transform = ch.is_rudolf_transform;
            let t_uid = ch.transform_target_uid;
            let t_id = if is_revert { 0 } else { new_id };
            let caught_id = ch.transform_caught_id;

            let mut neu = Character::new(uid, data, team, x, z);
            neu.base.ps.y = y;
            neu.base.hp = hp;
            neu.base.mp = mp;
            neu.base.facing = facing;
            neu.base.ai = ai;
            neu.base.controller_index = ci;
            neu.base.name = name;
            neu.base.effect.blink = true;
            neu.base.effect.super_armor = true;
            neu.base.effect.timeout = 20;
            if is_revert {
                neu.is_rudolf_transform = false;
                neu.transform_target_id = 0;
                neu.transform_caught_id = 0;
                neu.transform_target_uid = None;
            } else {
                neu.is_rudolf_transform = true;
                neu.transform_target_id = t_id;
                neu.transform_caught_id = caught_id;
                neu.transform_target_uid = t_uid;
                // release held victim after transform
                if let Some(vid) = ch.base.holding_uid {
                    // clear victim held_by below
                    let _ = vid;
                }
            }
            let hold = ch.base.holding_uid.take();
            *ch = neu;
            if let Some(vid) = hold {
                if let Some(vic) = self.characters.iter_mut().find(|c| c.base.uid == vid) {
                    vic.base.held_by = None;
                    vic.base.trans_frame(212, 10);
                }
            }
            let _ = was_transform;
            // smoke opoint 204 if available
            if let Some(data) = self.package_objects.get(&204).cloned() {
                let eo = crate::lf::effect::EffectObj::with_frame(
                    self.next_uid,
                    data,
                    x,
                    y - 70.0,
                    z,
                    70,
                );
                self.next_uid += 1;
                self.effects.push(eo);
            }
        }
    }

    fn spawn_broken(&mut self, x: f64, y: f64, z: f64) {
        if let Some(data) = self.package_objects.get(&320).cloned() {
            for i in 0..4 {
                let mut w = crate::lf::weapon::Weapon::new(self.next_uid, data.clone(), x + i as f64 * 3.0, z);
                self.next_uid += 1;
                w.base.ps.y = y;
                w.base.ps.vx = (i as f64 - 1.5) * 4.0;
                w.base.ps.vy = -5.0 - i as f64;
                w.base.trans_frame(0, 0);
                // use special as debris if weapon type wrong — use effect object
            }
        }
        if let Some(data) = self.package_objects.get(&320).cloned() {
            for i in 0..6 {
                let mut eo = crate::lf::effect::EffectObj::new(
                    self.next_uid,
                    data.clone(),
                    x + (i as f64 - 3.0) * 8.0,
                    y,
                    z,
                );
                self.next_uid += 1;
                eo.base.ps.vx = (i as f64 - 3.0) * 2.0;
                eo.base.ps.vy = -3.0;
                self.effects.push(eo);
            }
        }
    }


    fn char_hits_specials(&mut self) {
        let mut kill = vec![];
        for (ci, ch) in self.characters.iter().enumerate() {
            if ch.base.removed || ch.base.arest > 0 { continue; }
            let Some(frame) = ch.base.frame_data().cloned() else { continue };
            let itrs = Mech::itr_volumes(&ch.base.ps, ch.base.facing, &frame);
            for (si, sp) in self.specials.iter().enumerate() {
                if sp.base.removed { continue; }
                if sp.base.team == ch.base.team && ch.base.team != 0 { continue; }
                let Some(sf) = sp.base.frame_data().cloned() else { continue };
                let bdys = Mech::body_volumes(&sp.base.ps, sp.base.facing, &sf);
                for (vol, itr) in &itrs {
                    for b in &bdys {
                        if !vol.intersects(b) { continue; }
                        if itr.kind == 9 {
                            kill.push(si);
                        } else if itr.kind == 14 {
                            // ice column — break immediately
                            kill.push(si);
                        } else if itr.kind == 0 {
                            // damage type3
                            kill.push(si); // destroy ball on hit often
                        }
                    }
                }
            }
            let _ = ci;
        }
        kill.sort_unstable();
        kill.dedup();
        for si in kill.into_iter().rev() {
            if si < self.specials.len() {
                self.specials[si].base.hp = 0.0;
                self.specials[si].base.trans_frame(1000, 5);
            }
        }
    }

    fn spawn_opoints(&mut self) {
        let mut spawns = vec![];
        let mut spawned_uids = vec![];
        for ch in &self.characters {
            if let Some(fd) = ch.base.frame_data() {
                if let Some(op) = &fd.opoint {
                    // spawn on first TU of frame (wait == original wait)
                    if !ch.base.opoint_spawned && ch.base.frame.wait_left == fd.wait && op.oid != 0 {
                        let x = ch.base.ps.x + (op.x - fd.centerx) * ch.base.facing as f64;
                        let y = ch.base.ps.y + (op.y - fd.centery);
                        spawns.push((op.oid, ch.base.team, x, y, ch.base.ps.z, ch.base.facing, op.action, op.dvx, op.dvy, op.kind));
                        spawned_uids.push(ch.base.uid);
                    }
                }
            }
        }
        for uid in spawned_uids {
            if let Some(ch) = self.characters.iter_mut().find(|c| c.base.uid == uid) {
                ch.base.opoint_spawned = true;
            }
        }
        for (oid, team, x, y, z, facing, action, dvx, dvy, okind) in spawns {
            if let Some(data) = self.package_objects.get(&oid).cloned() {
                let ty = data.obj_type.as_str();
                if okind == 1 && (oid >= 300 || ty == "effect" || ty.is_empty() && oid >= 300) {
                    let mut eo = crate::lf::effect::EffectObj::new(self.next_uid, data, x, y, z);
                    self.next_uid += 1;
                    if action != 0 { eo.base.trans_frame(action, 0); }
                    self.effects.push(eo);
                    continue;
                }
                if ty == "lightweapon" || ty == "heavyweapon" || ty == "drink" {
                    let mut w = crate::lf::weapon::Weapon::new(self.next_uid, data, x, z);
                    self.next_uid += 1;
                    w.base.ps.y = y;
                    w.base.team = team;
                    w.base.facing = facing;
                    w.base.ps.vx = dvx * facing as f64;
                    w.base.ps.vy = dvy;
                    self.weapons.push(w);
                } else {
                    let mut s = SpecialAttack::new(self.next_uid, data, team, x, y, z, facing)
                        .with_velocity(dvx, dvy);
                    self.next_uid += 1;
                    if action != 0 {
                        s.base.trans_frame(action, 0);
                    }
                    self.specials.push(s);
                }
            }
        }
    }

    /// Opoints on special frames (balls spawning sub-effects)
    fn spawn_special_opoints(&mut self) {
        let mut spawns = vec![];
        for (si, sp) in self.specials.iter().enumerate() {
            if sp.base.removed || sp.opoint_done {
                continue;
            }
            let Some(fd) = sp.base.frame_data() else {
                continue;
            };
            let Some(op) = &fd.opoint else {
                continue;
            };
            if op.oid == 0 {
                continue;
            }
            if sp.base.frame.wait_left != fd.wait {
                continue;
            }
            let x = sp.base.ps.x + (op.x - fd.centerx) * sp.base.facing as f64;
            let y = sp.base.ps.y + (op.y - fd.centery);
            spawns.push((
                si,
                op.oid,
                sp.base.team,
                x,
                y,
                sp.base.ps.z,
                sp.base.facing,
                op.action,
                op.dvx,
                op.dvy,
            ));
        }
        for (si, oid, team, x, y, z, facing, action, dvx, dvy) in spawns {
            if si < self.specials.len() {
                self.specials[si].opoint_done = true;
            }
            if let Some(data) = self.package_objects.get(&oid).cloned() {
                let ty = data.obj_type.clone();
                if ty == "specialattack" || ty.is_empty() || data.id >= 200 && data.id < 300 {
                    let mut s = SpecialAttack::new(self.next_uid, data, team, x, y, z, facing)
                        .with_velocity(dvx, dvy);
                    self.next_uid += 1;
                    if action != 0 {
                        s.base.trans_frame(action, 0);
                    }
                    self.specials.push(s);
                } else {
                    let mut eo = crate::lf::effect::EffectObj::new(self.next_uid, data, x, y, z);
                    self.next_uid += 1;
                    if action != 0 {
                        eo.base.trans_frame(action, 0);
                    }
                    self.effects.push(eo);
                }
            }
        }
    }

    /// Ball vs ball: ice/fire clash from specialattack.js
    fn special_vs_special(&mut self) {
        let n = self.specials.len();
        let mut die: Vec<usize> = vec![];
        let mut shards: Vec<(f64, f64, f64)> = vec![];
        for a in 0..n {
            if self.specials[a].base.removed {
                continue;
            }
            let Some(fa) = self.specials[a].base.frame_data().cloned() else {
                continue;
            };
            let itrs_a = Mech::itr_volumes(
                &self.specials[a].base.ps,
                self.specials[a].base.facing,
                &fa,
            );
            for b in (a + 1)..n {
                if self.specials[b].base.removed {
                    continue;
                }
                // same team head-on only cancel if opposing dirs
                let same_team = self.specials[a].base.team == self.specials[b].base.team
                    && self.specials[a].base.team != 0;
                if same_team && self.specials[a].base.facing == self.specials[b].base.facing {
                    continue;
                }
                let Some(fb) = self.specials[b].base.frame_data().cloned() else {
                    continue;
                };
                let bdys_b = Mech::body_volumes(
                    &self.specials[b].base.ps,
                    self.specials[b].base.facing,
                    &fb,
                );
                // also itr vs itr / body
                let itrs_b = Mech::itr_volumes(
                    &self.specials[b].base.ps,
                    self.specials[b].base.facing,
                    &fb,
                );
                let mut hit = false;
                for (va, _) in &itrs_a {
                    for (vb, _) in &itrs_b {
                        if va.intersects(vb) {
                            hit = true;
                            break;
                        }
                    }
                    for bb in &bdys_b {
                        if va.intersects(bb) {
                            hit = true;
                            break;
                        }
                    }
                }
                if !hit {
                    continue;
                }
                let a_ice = self.specials[a].is_ice_ball();
                let b_ice = self.specials[b].is_ice_ball();
                let a_fire = self.specials[a].is_fire_ball();
                let b_fire = self.specials[b].is_fire_ball();
                // non-ice hit by ice → destroy non-ice, spawn 209
                if a_ice && !b_ice {
                    die.push(b);
                    shards.push((
                        self.specials[b].base.ps.x,
                        self.specials[b].base.ps.y,
                        self.specials[b].base.ps.z,
                    ));
                } else if b_ice && !a_ice {
                    die.push(a);
                    shards.push((
                        self.specials[a].base.ps.x,
                        self.specials[a].base.ps.y,
                        self.specials[a].base.ps.z,
                    ));
                } else if a_fire && b_fire || a_ice && b_ice || (!a_ice && !b_ice) {
                    // mutual destroy
                    die.push(a);
                    die.push(b);
                } else {
                    die.push(a);
                    die.push(b);
                }
            }
        }
        die.sort_unstable();
        die.dedup();
        for si in die.into_iter().rev() {
            if si < self.specials.len() {
                self.specials[si].mark_die(1000);
            }
        }
        for (x, y, z) in shards {
            // ice shatter effect 209 if present else 302
            for oid in [209i32, 302, 300] {
                if let Some(data) = self.package_objects.get(&oid).cloned() {
                    let eo = crate::lf::effect::EffectObj::new(self.next_uid, data, x, y - 20.0, z);
                    self.next_uid += 1;
                    self.effects.push(eo);
                    break;
                }
            }
        }
    }

    /// itr kind 2 on character frames — pick light weapons in volume
    fn itr_kind2_pick(&mut self) {
        let n = self.characters.len();
        let mut picks: Vec<(usize, usize)> = vec![];
        for i in 0..n {
            if self.characters[i].base.removed || self.characters[i].hold_weapon.is_some() {
                continue;
            }
            if self.characters[i].base.arest > 0 {
                continue;
            }
            let Some(frame) = self.characters[i].base.frame_data().cloned() else {
                continue;
            };
            let has_k2 = frame.itr.iter().any(|it| it.kind == 2 || it.kind == 7);
            if !has_k2 {
                continue;
            }
            let itrs = Mech::itr_volumes(
                &self.characters[i].base.ps,
                self.characters[i].base.facing,
                &frame,
            );
            for (wi, w) in self.weapons.iter().enumerate() {
                if w.held || w.base.removed {
                    continue;
                }
                // kind 7 cannot pick heavy
                let only_light = frame.itr.iter().any(|it| it.kind == 7);
                if only_light && w.heavy {
                    continue;
                }
                let Some(wf) = w.base.frame_data().cloned() else {
                    continue;
                };
                let bdys = Mech::body_volumes(&w.base.ps, w.base.facing, &wf);
                for (vol, itr) in &itrs {
                    if itr.kind != 2 && itr.kind != 7 {
                        continue;
                    }
                    if itr.kind == 7 && w.heavy {
                        continue;
                    }
                    for b in &bdys {
                        if vol.intersects(b) {
                            picks.push((i, wi));
                        }
                    }
                }
            }
        }
        for (ci, wi) in picks {
            if self.characters[ci].hold_weapon.is_some() || self.weapons[wi].held {
                continue;
            }
            let uid = self.weapons[wi].base.uid;
            let woid = self.weapons[wi].base.id;
            let wtype = self.weapons[wi].base.obj_type.clone();
            self.weapons[wi].held = true;
            self.weapons[wi].holder_uid = Some(self.characters[ci].base.uid);
            self.weapons[wi].base.team = self.characters[ci].base.team;
            self.characters[ci].hold_weapon = Some(uid);
            self.characters[ci].hold_weapon_oid = woid;
            self.characters[ci].base.holding_uid = Some(uid);
            self.characters[ci].base.hold_type = wtype;
            self.characters[ci].base.itr_arest_update(3);
            self.sound.play("1/020");
        }
    }

    /// bpoint on frames — spawn blood effect at bleeding point
    fn bpoint_blood(&mut self) {
        let mut pts = vec![];
        for ch in &self.characters {
            if ch.base.removed || !matches!(ch.base.state(), 11 | 12 | 16) {
                continue;
            }
            let Some(fd) = ch.base.frame_data() else {
                continue;
            };
            let Some((bx, by)) = fd.bpoint else {
                continue;
            };
            if ch.base.frame.wait_left != fd.wait {
                continue;
            }
            let x = if ch.base.facing >= 0 {
                ch.base.ps.x - fd.centerx + bx
            } else {
                ch.base.ps.x + fd.centerx - bx
            };
            let y = ch.base.ps.y + (by - fd.centery);
            pts.push((x, y, ch.base.ps.z));
        }
        for (x, y, z) in pts {
            if !self.effects_pool.create(x, y, z, 0) {
                if let Some(data) = self.package_objects.get(&301).cloned() {
                    let eo = crate::lf::effect::EffectObj::new(self.next_uid, data, x, y, z);
                    self.next_uid += 1;
                    self.effects.push(eo);
                }
            }
        }
    }

    /// Burning state 18 — broken effect sparks (character.js brokeneffect 302)
    fn burn_broken_fx(&mut self) {
        let mut sparks = vec![];
        for ch in &self.characters {
            if ch.base.removed {
                continue;
            }
            let st = ch.base.state();
            if st == 18 && ch.base.frame.wait_left == ch.base.frame_data().map(|f| f.wait).unwrap_or(0) {
                sparks.push((ch.base.ps.x, ch.base.ps.y - 30.0, ch.base.ps.z));
            }
        }
        for (x, y, z) in sparks {
            if self.effects_pool.create(x, y, z, 0) {
                continue;
            }
            if let Some(data) = self.package_objects.get(&302).cloned() {
                let eo = crate::lf::effect::EffectObj::new(self.next_uid, data, x, y, z);
                self.next_uid += 1;
                self.effects.push(eo);
            }
        }
    }

    fn pick_weapons(&mut self) {
        // itr kind 1 on walking frames = catch weapon scope — approximate: stand/walk near weapon + att
        let mut picks: Vec<(usize, usize)> = vec![]; // char idx, weapon idx
        let mut throws: Vec<(usize, f64, f64, f64)> = vec![]; // weapon idx after drop from char

        for (ci, ch) in self.characters.iter().enumerate() {
            if ch.base.removed { continue; }
            // throw frames 45, 50 — release weapon
            let fr = ch.base.frame.n;
            if ch.hold_weapon.is_some() && matches!(fr, 45 | 46 | 47 | 50 | 51 | 52) {
                if let Some(fd) = ch.base.frame_data() {
                    if let Some(wp) = &fd.wpoint {
                        if wp.kind == 2 || wp.attacking > 0 || fr == 50 {
                            let facing = ch.base.facing as f64;
                            throws.push((ci, 8.0 * facing, -4.0, 0.0));
                        }
                    }
                }
            }
        }
        for (ci, vx, vy, vz) in throws {
            if let Some(wid) = self.characters[ci].hold_weapon.take() {
                self.characters[ci].hold_weapon_oid = 0;
                self.characters[ci].base.holding_uid = None;
                self.characters[ci].base.hold_type.clear();
                if let Some(w) = self.weapons.iter_mut().find(|w| w.base.uid == wid) {
                    w.drop(vx, vy, vz);
                }
            }
        }

        // pick up: character in state 0/1, att edge, close to weapon
        for (ci, ch) in self.characters.iter().enumerate() {
            if ch.base.removed || ch.hold_weapon.is_some() { continue; }
            if !matches!(ch.base.state(), 0 | 1) { continue; }
            // use arest==0 and attack intent: frame just entered 60 or att — simpler distance check each TU if very close
            for (wi, w) in self.weapons.iter().enumerate() {
                if w.held { continue; }
                let dx = (w.base.ps.x - ch.base.ps.x).abs();
                let dz = (w.base.ps.z - ch.base.ps.z).abs();
                let dy = (w.base.ps.y - ch.base.ps.y).abs();
                if dx < 35.0 && dz < 12.0 && dy < 20.0 && w.base.ps.y >= -5.0 {
                    // pick when character attacks or walks over with att held — use frame 60 start
                    if matches!(ch.base.frame.n, 60 | 65 | 20 | 25) || ch.base.state() == 1 {
                        picks.push((ci, wi));
                        break;
                    }
                }
            }
        }
        for (ci, wi) in picks {
            if self.characters[ci].hold_weapon.is_some() { continue; }
            if self.weapons[wi].held { continue; }
            let uid = self.weapons[wi].base.uid;
            let woid = self.weapons[wi].base.id;
            let wtype = self.weapons[wi].base.obj_type.clone();
            self.weapons[wi].held = true;
            self.weapons[wi].holder_uid = Some(self.characters[ci].base.uid);
            self.weapons[wi].base.team = self.characters[ci].base.team;
            self.characters[ci].hold_weapon = Some(uid);
            self.characters[ci].hold_weapon_oid = woid;
            self.characters[ci].base.holding_uid = Some(uid);
            self.characters[ci].base.hold_type = wtype;
        }

        // sync held weapons to wpoint
        for ch in &self.characters {
            if let Some(wid) = ch.hold_weapon {
                if let Some((x, y, z, facing, wact)) = ch.wpoint_world() {
                    if let Some(w) = self.weapons.iter_mut().find(|w| w.base.uid == wid) {
                        w.attach_to(ch.base.uid, x, y, z, facing);
                        w.base.team = ch.base.team;
                        if wact > 0 {
                            w.base.trans_frame(wact, 2);
                        }
                    }
                }
            }
        }
    }


    /// Queue create (F.LF pushes task; applied in process_tasks same TU end)
    pub fn create_object(&mut self, oid: i32, team: i32, x: f64, y: f64, z: f64, facing: i32, action: i32, dvx: f64, dvy: f64) {
        self.tasks.push(MatchTask::CreateObject { oid, team, x, y, z, facing, action, dvx, dvy });
    }

    pub fn destroy_object_uid(&mut self, uid: u32) {
        self.tasks.push(MatchTask::DestroyUid(uid));
    }

    fn process_tasks(&mut self) {
        let tasks = std::mem::take(&mut self.tasks);
        for task in tasks {
            match task {
                MatchTask::CreateObject { oid, team, x, y, z, facing, action, dvx, dvy } => {
                    self.spawn_object_now(oid, team, x, y, z, facing, action, dvx, dvy);
                }
                MatchTask::DestroyUid(uid) => {
                    if let Some(ch) = self.characters.iter_mut().find(|c| c.base.uid == uid) {
                        ch.base.removed = true;
                        ch.base.hp = 0.0;
                    }
                    if let Some(w) = self.weapons.iter_mut().find(|w| w.base.uid == uid) {
                        w.die();
                    }
                    if let Some(s) = self.specials.iter_mut().find(|s| s.base.uid == uid) {
                        s.mark_die(1000);
                    }
                }
            }
        }
    }

    fn spawn_object_now(&mut self, oid: i32, team: i32, x: f64, y: f64, z: f64, facing: i32, action: i32, dvx: f64, dvy: f64) {
        if let Some(data) = self.package_objects.get(&oid).cloned() {
            let ty = data.obj_type.as_str();
            if ty == "lightweapon" || ty == "heavyweapon" || ty == "drink" {
                let mut w = crate::lf::weapon::Weapon::new(self.next_uid, data, x, z);
                self.next_uid += 1;
                w.base.ps.y = y;
                w.base.team = team;
                w.base.facing = facing;
                w.base.ps.vx = dvx * facing as f64;
                w.base.ps.vy = dvy;
                self.weapons.push(w);
            } else if oid >= 300 && oid < 400 {
                let mut eo = crate::lf::effect::EffectObj::new(self.next_uid, data, x, y, z);
                self.next_uid += 1;
                if action != 0 { eo.base.trans_frame(action, 0); }
                self.effects.push(eo);
            } else {
                let mut s = SpecialAttack::new(self.next_uid, data, team, x, y, z, facing).with_velocity(dvx, dvy);
                self.next_uid += 1;
                if action != 0 { s.base.trans_frame(action, 0); }
                self.specials.push(s);
            }
        }
    }

    /// LF match.get_living_object list of character uids still alive
    pub fn get_living_object_uids(&self) -> Vec<u32> {
        self.characters.iter().filter(|c| !c.base.removed && c.base.hp > 0.0).map(|c| c.base.uid).collect()
    }


    /// F.LF create_multiple_objects — fan vz
    pub fn create_multiple_objects(&mut self, oid: i32, team: i32, x: f64, y: f64, z: f64, facing: i32, action: i32, number: i32, vz_step: f64) {
        let n = number.max(1) as i32;
        for i in 0..n {
            let vz = (i as f64 - (n as f64 - 1.0) / 2.0) * vz_step;
            self.tasks.push(MatchTask::CreateObject {
                oid, team, x, y, z: z + vz * 0.01, facing, action, dvx: 0.0, dvy: 0.0,
            });
        }
    }

    pub fn create_weapon(&mut self, id: i32, x: f64, z: f64) {
        self.create_object(id, 0, x, 0.0, z, 1, 0, 0.0, 0.0);
    }

    pub fn drop_weapons_setup(&mut self, positions: &[(f64, f64)]) {
        let ids = [100i32, 101, 150, 151];
        for (i, &(x, z)) in positions.iter().enumerate() {
            let id = ids[i % ids.len()];
            self.create_weapon(id, x, z);
        }
    }

    /// Deterministic RNG like match.random / new_randomseed
    pub fn new_randomseed(&mut self, seed: u64) {
        self.rng_state = seed;
    }
    pub fn random_f(&mut self) -> f64 {
        // xorshift
        let mut x = self.rng_state;
        if x == 0 { x = 0x1234_5678_9ABC_DEF0; }
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.rng_state = x;
        (x as f64) / (u64::MAX as f64)
    }

    pub fn overlay_message(&mut self, mess: &str) {
        self.overlay_msg = mess.to_string();
        self.overlay_ttl = 90;
    }

    pub fn transform_panel(&mut self, from_uid: u32, to_uid: Option<u32>) {
        // UI panels use character names — mark for render
        self.panel_remap.insert(from_uid, to_uid.unwrap_or(from_uid));
    }

    /// F.LF match.for_all on characters
    pub fn for_all_characters<F: FnMut(&mut Character)>(&mut self, mut f: F) {
        for ch in &mut self.characters {
            if !ch.base.removed {
                f(ch);
            }
        }
    }

    pub fn F7_refill(&mut self) {
        for ch in &mut self.characters {
            ch.base.hp = ch.base.hp_full;
            ch.base.hp_bound = ch.base.hp_full;
            ch.base.mp = ch.base.mp_full;
            ch.base.dead = false;
            ch.base.removed = false;
        }
    }

    /// F.LF match.F4 — end match / destroy
    pub fn f4_end_match(&mut self) {
        self.game_over = true;
        self.overlay_message("MATCH END (F4)");
        self.overlay_ttl = 9999;
    }

    /// F.LF match.drop_weapons — spawn random light weapons on field
    pub fn drop_weapons(&mut self) {
        let positions: Vec<(f64, f64)> = self
            .characters
            .iter()
            .filter(|c| !c.base.removed)
            .map(|c| (c.base.ps.x + 40.0, c.base.ps.z))
            .collect();
        if positions.is_empty() {
            let mid = self.background.width / 2.0;
            let z = (self.background.zboundary.0 + self.background.zboundary.1) / 2.0;
            self.drop_weapons_setup(&[(mid, z), (mid + 80.0, z)]);
        } else {
            self.drop_weapons_setup(&positions);
        }
        self.overlay_message("F8 drop weapons");
    }

    /// F.LF match.destroy_weapons
    pub fn destroy_weapons(&mut self) {
        for w in &mut self.weapons {
            w.base.hp = 0.0;
            w.held = false;
            w.base.removed = true;
            w.die();
        }
        for ch in &mut self.characters {
            ch.hold_weapon = None;
            ch.base.holding_uid = None;
            ch.base.hold_type.clear();
        }
        self.overlay_message("F9 destroy weapons");
    }

    /// Update chase ball targets (nearest enemy to ball team)
    pub fn update_special_chase_targets(&mut self) {
        for sp in &mut self.specials {
            if sp.base.removed {
                continue;
            }
            let st = sp.base.state();
            let hit_fa = sp
                .base
                .frame_data()
                .map(|f| f.hit_Fa)
                .unwrap_or(0);
            if hit_fa == 0 && !(3002..=3010).contains(&st) {
                continue;
            }
            let team = sp.base.team;
            let sx = sp.base.ps.x;
            let sz = sp.base.ps.z;
            let mut best = None;
            let mut best_d = f64::MAX;
            for ch in &self.characters {
                if ch.base.removed || ch.base.hp <= 0.0 || ch.base.team == team {
                    continue;
                }
                let d = (ch.base.ps.x - sx).hypot(ch.base.ps.z - sz);
                if d < best_d {
                    best_d = d;
                    best = Some((ch.base.ps.x, ch.base.ps.z));
                }
            }
            if let Some((x, z)) = best {
                sp.chase_x = x;
                sp.chase_z = z;
            }
        }
    }

    fn update_camera(&mut self) {
        let mut min_x = f64::MAX;
        let mut max_x = f64::MIN;
        let mut any = false;
        for ch in &self.characters {
            if ch.base.removed {
                continue;
            }
            any = true;
            min_x = min_x.min(ch.base.ps.x);
            max_x = max_x.max(ch.base.ps.x);
        }
        if !any {
            return;
        }
        let mid = (min_x + max_x) / 2.0;
        let target = (mid - global::WINDOW_WIDTH / 2.0)
            .max(0.0)
            .min((self.background.width - global::WINDOW_WIDTH).max(0.0));
        self.camera_x += (target - self.camera_x) * global::CAMERA_SPEED_FACTOR * 4.0;
    }

    fn check_game_over(&mut self) {
        let mut teams = std::collections::HashSet::new();
        for ch in &self.characters {
            if !ch.base.removed && ch.base.hp > 0.0 {
                teams.insert(ch.base.team);
            }
        }
        if teams.len() <= 1 && self.time > 90 {
            self.game_over = true;
            self.winner_team = teams.into_iter().next();
            let mut msg = "GAME OVER".to_string();
            if let Some(w) = self.winner_team {
                msg = format!("GAME OVER — team {} wins", w);
            }
            self.overlay_message(&msg);
            self.overlay_ttl = 9999;
        }
    }

    pub fn render(&mut self, ren: &mut CanvasRenderer) {
        ren.cam_x = self.camera_x;
        ren.cam_y = 0.0;
        // gameplay viewer is below panels in original — full canvas here
        self.background.draw(ren, self.time);

        // shadows
        for ch in &self.characters {
            if ch.base.removed || !ch.base.sp.visible {
                continue;
            }
            self.background
                .draw_shadow(ren, ch.base.ps.x, ch.base.ps.z);
        }

        let mut items: Vec<(f64, i32, SpriteDraw)> = vec![];
        for ch in &self.characters {
            if ch.base.removed {
                continue;
            }
            if let Some(fd) = ch.base.frame_data() {
                items.push((
                    ch.base.ps.z,
                    ch.base.uid as i32,
                    SpriteDraw {
                        sp: ch.base.sp.clone(),
                        cx: fd.centerx,
                        cy: fd.centery,
                    },
                ));
            }
        }
        for w in &self.weapons {
            if let Some(fd) = w.base.frame_data() {
                items.push((
                    w.base.ps.z,
                    w.base.uid as i32,
                    SpriteDraw {
                        sp: w.base.sp.clone(),
                        cx: fd.centerx,
                        cy: fd.centery,
                    },
                ));
            }
        }
        for e in &self.effects {
            if let Some(fd) = e.base.frame_data() {
                items.push((
                    e.base.ps.z,
                    e.base.uid as i32,
                    SpriteDraw {
                        sp: e.base.sp.clone(),
                        cx: fd.centerx,
                        cy: fd.centery,
                    },
                ));
            }
        }
        for e in self.effects_pool.drain_live_refs() {
            if let Some(fd) = e.base.frame_data() {
                items.push((
                    e.base.ps.z,
                    e.base.uid as i32,
                    SpriteDraw {
                        sp: e.base.sp.clone(),
                        cx: fd.centerx,
                        cy: fd.centery,
                    },
                ));
            }
        }
        for s in &self.specials {
            if let Some(fd) = s.base.frame_data() {
                items.push((
                    s.base.ps.z,
                    s.base.uid as i32,
                    SpriteDraw {
                        sp: s.base.sp.clone(),
                        cx: fd.centerx,
                        cy: fd.centery,
                    },
                ));
            }
        }
        items.sort_by(|a, b| {
            a.0.partial_cmp(&b.0)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.1.cmp(&b.1))
        });
        // fix Equal
        let _ = 0i32;

        for (_, _, d) in &items {
            ren.draw_sprite(&d.sp, d.cx, d.cy);
        }

        self.draw_panels(ren);
        if self.overlay_ttl > 0 {
            ren.fill_text(&self.overlay_msg, 280.0, 120.0, "#fff", "bold 20px sans-serif");
            self.overlay_ttl -= 1;
        }

        if self.paused {
            ren.fill_rect_color(0.0, 180.0, ren.width, 40.0, "rgba(0,0,0,0.5)");
            ren.fill_text("PAUSED", 360.0, 208.0, "#fff", "bold 28px sans-serif");
        }
        if self.game_over {
            ren.fill_rect_color(200.0, 150.0, 400.0, 100.0, "rgba(0,0,0,0.75)");
            ren.fill_text("GAME OVER", 300.0, 195.0, "#ff0", "bold 32px sans-serif");
            if let Some(wt) = self.winner_team {
                ren.fill_text(&format!("Winner team {}", wt), 320.0, 175.0, "#0f0", "16px sans-serif");
            }
            let mut ky = 220.0;
            for ch in &self.characters {
                if ch.base.kills > 0 || !ch.base.removed {
                    ren.fill_text(
                        &format!("{}  K:{} HP:{:.0}", ch.base.name, ch.base.kills, ch.base.hp.max(0.0)),
                        280.0,
                        ky,
                        "#ccc",
                        "12px sans-serif",
                    );
                    ky += 16.0;
                }
            }
            if let Some(t) = self.winner_team {
                let names: Vec<_> = self.characters.iter()
                    .filter(|c| c.base.team == t && !c.base.removed)
                    .map(|c| c.base.name.as_str())
                    .collect();
                ren.fill_text(
                    &format!("Team {} wins — {}", t, names.join(", ")),
                    260.0,
                    225.0,
                    "#fff",
                    "16px sans-serif",
                );
            }
            ren.fill_text("Esc — menu", 340.0, 248.0, "#ccc", "14px sans-serif");
        }
    }

    fn draw_panels(&self, ren: &mut CanvasRenderer) {
        // LF2-style top panels
        let pane_w = 198.0;
        let pane_h = 53.0;
        for (i, ch) in self.characters.iter().enumerate().take(4) {
            let x = 5.0 + (i as f64) * pane_w;
            let y = 6.0;
            if let Some(panel) = &self.ui_panel {
                let pic = panel["pic"].as_str().unwrap_or("UI/panel.png");
                // can't mut borrow ren easily for ensure — use fill fallback
                let _ = pic;
            }
            // team tint border
            let border = match ch.base.team {
                1 => "rgba(40,40,120,0.95)",
                2 => "rgba(120,40,40,0.95)",
                3 => "rgba(40,100,40,0.95)",
                4 => "rgba(100,100,40,0.95)",
                _ => "rgba(20,20,40,0.85)",
            };
            ren.fill_rect_color(x, y, pane_w - 4.0, pane_h, border);
            ren.fill_rect_color(x + 2.0, y + 2.0, pane_w - 8.0, pane_h - 4.0, "#1a1a2e");
            // HP
            let hpx = x + 50.0;
            let hpy = y + 12.0;
            let hpw = 125.0;
            let hph = 10.0;
            ren.fill_rect_color(hpx, hpy, hpw, hph, "#6f081f");
            let pct = (ch.base.hp / ch.base.hp_full).clamp(0.0, 1.0);
            ren.fill_rect_color(hpx, hpy, hpw * pct, hph, if pct > 0.3 { "#ff0000" } else { "#ff8888" });
            // MP
            let mpy = y + 28.0;
            ren.fill_rect_color(hpx, mpy, hpw, hph, "#1f086f");
            let mpct = (ch.base.mp / ch.base.mp_full).clamp(0.0, 1.0);
            ren.fill_rect_color(hpx, mpy, hpw * mpct, hph, "#0000ff");
            let head = &ch.base.data.bmp.head;
            if !head.is_empty() {
                // need mut ren - method is &self draw_panels - change to &mut self
            }
            if !ch.base.data.bmp.head.is_empty() {
                ren.draw_image_scaled(&ch.base.data.bmp.head, x + 4.0, y + 8.0, 40.0, 40.0);
            }
            ren.fill_text(&ch.base.name, x + 48.0, y + 48.0, "#afdcff", "11px sans-serif");
            if ch.base.kills > 0 {
                ren.fill_text(&format!("K:{}", ch.base.kills), x + pane_w - 36.0, y + 48.0, "#ffaa00", "10px sans-serif");
            }
        }
    }
}

struct SpriteDraw {
    sp: crate::core_engine::sprite::SpriteInstance,
    cx: f64,
    cy: f64,
}
