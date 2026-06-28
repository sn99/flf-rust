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
    pub ui_panel: Option<Value>,
    pub winner_team: Option<i32>,
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

        let ai_brains = characters.iter().map(|c| {
                if c.base.ai { crate::lf::ai::AiBrain::default() } else { crate::lf::ai::AiBrain::default() }
            }).collect();
        Ok(Self {
            characters,
            weapons,
            specials: vec![],
            effects: vec![],
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
            ui_panel: package.ui.get("panel").cloned(),
            winner_team: None,
        })
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    pub fn tu(&mut self) {
        if self.paused || self.game_over {
            return;
        }
        self.time += 1;
        let bg_z = self.background.zboundary;
        let bg_w = self.background.width;

        let snapshot: Vec<(i32, f64, f64, bool)> = self
            .characters
            .iter()
            .map(|c| (c.base.team, c.base.ps.x, c.base.ps.z, c.base.removed || c.base.hp <= 0.0))
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

        let ctrls = self.controllers.borrow();
        let time = self.time;
        while self.ai_brains.len() < self.characters.len() {
            self.ai_brains.push(crate::lf::ai::AiBrain::default());
        }
        for (i, ch) in self.characters.iter_mut().enumerate() {
            if ch.base.ai {
                let mut cfg = std::collections::HashMap::new();
                for k in ["up", "down", "left", "right", "def", "jump", "att"] {
                    cfg.insert(k.to_string(), k.to_string());
                }
                let mut ac = Controller::new_keyboard(cfg);
                let my_team = ch.base.team;
                let enemies: Vec<(u32, f64, f64, f64, i32)> = enemy_snap
                    .iter()
                    .enumerate()
                    .filter(|(j, e)| {
                        *j < snapshot.len()
                            && snapshot[*j].0 != my_team
                            && !snapshot[*j].3
                            && e.0 != ch.base.uid
                    })
                    .map(|(_, e)| *e)
                    .collect();
                crate::lf::ai::ai_fill(&mut self.ai_brains[i], &ch.base, &enemies, &mut ac, time);
                ch.tu(Some(&ac), bg_z, bg_w);
            } else if let Some(ci) = ch.base.controller_index {
                ch.tu(ctrls.get(ci), bg_z, bg_w);
            } else {
                ch.tu(None, bg_z, bg_w);
            }
        }
        drop(ctrls);

        // frame sounds
        for ch in &mut self.characters {
            if let Some(path) = ch.base.take_sound() {
                self.sound.play(&path);
            }
        }
        self.sound.tu();

        for w in &mut self.weapons {
            w.tu(bg_z, bg_w);
        }
        for s in &mut self.specials {
            // projectile velocity from frame
            if let Some(fd) = s.base.frame_data().cloned() {
                if fd.dvx != 0.0 && fd.dvx as i32 != global::UNSPECIFIED {
                    s.base.ps.vx = fd.dvx * s.base.facing as f64;
                }
            }
            s.base.physics_tu(bg_z, bg_w);
        }
        self.specials.retain(|s| !s.base.removed);
        for e in &mut self.effects {
            e.base.physics_tu(bg_z, bg_w);
        }
        self.effects.retain(|e| !e.base.removed && !e.base.dead);

        self.process_hits();
        self.process_catches();
        self.process_throws();
        self.special_hits();
        self.spawn_opoints();
        self.pick_weapons();
        self.update_camera();
        self.check_game_over();
    }

    fn process_hits(&mut self) {
        let n = self.characters.len();
        let mut events: Vec<(usize, usize, f64, f64, f64, f64, i32, i32)> = vec![];

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
                if self.characters[i]
                    .base
                    .vrest
                    .get(&self.characters[j].base.uid)
                    .copied()
                    .unwrap_or(0)
                    > 0
                {
                    continue;
                }
                if self.characters[j].base.effect.super_armor {
                    continue;
                }
                let Some(vframe) = self.characters[j].base.frame_data().cloned() else { continue };
                let bdys = Mech::body_volumes(
                    &self.characters[j].base.ps,
                    self.characters[j].base.facing,
                    &vframe,
                );
                for (vol, itr) in &itrs {
                    // kind 0 normal; 3 fire-like; 4 ice-like still apply injury in LF2
                    if itr.kind != 0 && itr.kind != 3 && itr.kind != 4 && itr.kind != 5 {
                        continue;
                    }
                    for b in &bdys {
                        // xy intersect + z range
                        if vol.intersects(b) {
                            let injury = if itr.injury != 0.0 { itr.injury } else { 20.0 };
                            let fall = if itr.fall != 0.0 { itr.fall } else { global::DEFAULT_FALL };
                            events.push((i, j, injury, fall, vol.vx, itr.dvy, itr.arest, itr.effect));
                        }
                    }
                }
            }
        }

        for (i, j, injury, fall, dvx, dvy, arest, eff) in events {
            let facing = self.characters[i].base.facing;
            self.characters[i].base.arest = if arest > 0 { arest } else { global::DEFAULT_AREST };
            let vid = self.characters[j].base.uid;
            self.characters[i]
                .base
                .vrest
                .insert(vid, global::DEFAULT_VREST);

            let defending = self.characters[j].base.state() == 7;
            let mut inj = injury;
            if defending {
                // facing attacker?
                let same_dir = self.characters[j].base.facing == facing;
                if !same_dir {
                    inj *= global::DEFEND_INJURY_FACTOR;
                    self.characters[j].base.bdefend += injury;
                    if self.characters[j].base.bdefend < global::DEFEND_BREAK_LIMIT {
                        self.characters[j].base.trans_frame(111, 8);
                        continue;
                    }
                    // break defend
                    self.characters[j].base.trans_frame(112, 12);
                }
            }
            let dvy_use = if dvy != 0.0 { dvy } else { global::DEFAULT_FALL_DVY };
            self.characters[j]
                .base
                .injure(inj, fall, dvx, dvy_use, facing);
            // blood effect id 301
            let (bx, by, bz) = (
                self.characters[j].base.ps.x,
                self.characters[j].base.ps.y - 40.0,
                self.characters[j].base.ps.z,
            );
            let eid = crate::lf::effect::effect_id_from_num(0); // blood default; itr effect applied below
            // prefer 301 blood, 300 blast
            let mut try_ids = vec![301i32, 300];
            if eff > 0 {
                try_ids.insert(0, crate::lf::effect::effect_id_from_num(eff));
            }
            for try_id in try_ids {
                if let Some(data) = self.package_objects.get(&try_id).cloned() {
                    let eo = crate::lf::effect::EffectObj::new(self.next_uid, data, bx, by, bz);
                    self.next_uid += 1;
                    self.effects.push(eo);
                    break;
                }
            }
            let _ = eid;
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
                // only catch if victim in fall-ish or stand vulnerable
                if !matches!(self.characters[j].base.state(), 0 | 1 | 8 | 11 | 12 | 16) { continue; }
                let bdys = Mech::body_volumes(&self.characters[j].base.ps, self.characters[j].base.facing, &vframe);
                for (vol, itr) in &itrs {
                    if itr.kind != 1 { continue; }
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
        }
        // sync caught position to catcher cpoint roughly
        for i in 0..n {
            if let Some(vid) = self.characters[i].base.holding_uid {
                if let Some(fd) = self.characters[i].base.frame_data() {
                    if let Some(cp) = &fd.cpoint {
                        let x = self.characters[i].base.ps.x + (cp.x - fd.centerx) * self.characters[i].base.facing as f64;
                        let y = self.characters[i].base.ps.y + (cp.y - fd.centery);
                        let z = self.characters[i].base.ps.z;
                        if let Some(ch) = self.characters.iter_mut().find(|c| c.base.uid == vid) {
                            ch.base.ps.x = x;
                            ch.base.ps.y = y;
                            ch.base.ps.z = z;
                            ch.base.ps.vx = 0.0;
                            ch.base.ps.vy = 0.0;
                            ch.base.ps.vz = 0.0;
                            // throw on att while catching (state 9)
                            // handled via frame taction in future
                        }
                    }
                }
                // release if catcher not in catch frames
                let st = self.characters[i].base.state();
                if st != 9 && self.characters[i].base.frame.n < 120 || self.characters[i].base.frame.n > 140 && st != 9 {
                    // keep while in 120-140 range
                }
                let fn_ = self.characters[i].base.frame.n;
                if !(120..=150).contains(&fn_) && self.characters[i].base.state() != 9 {
                    let vid = self.characters[i].base.holding_uid.take();
                    if let Some(vid) = vid {
                        if let Some(ch) = self.characters.iter_mut().find(|c| c.base.uid == vid) {
                            ch.base.held_by = None;
                        }
                    }
                }
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
            if let Some(ch) = self.characters.iter_mut().find(|c| c.base.uid == cuid) {
                ch.base.holding_uid = None;
            }
            if let Some(vic) = self.characters.iter_mut().find(|c| c.base.uid == vid) {
                vic.base.held_by = None;
                vic.base.ps.vx = vx;
                vic.base.ps.vy = vy;
                vic.base.ps.vz = vz;
                vic.base.trans_frame(180, 15); // fall
                vic.base.hp -= 10.0;
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
                if ch.base.removed || ch.base.team == sp.base.team { continue; }
                let Some(vf) = ch.base.frame_data().cloned() else { continue };
                let bdys = Mech::body_volumes(&ch.base.ps, ch.base.facing, &vf);
                for (vol, itr) in &itrs {
                    if itr.kind != 0 { continue; }
                    for b in &bdys {
                        if vol.intersects(b) {
                            events.push((si, ci, itr.injury.max(15.0), itr.fall, vol.vx, itr.dvy));
                        }
                    }
                }
            }
        }
        for (si, ci, inj, fall, dvx, dvy) in events {
            let facing = self.specials[si].base.facing;
            let next_frame = self.specials[si]
                .base
                .frame_data()
                .map(|f| if f.next > 0 { f.next } else { 1000 })
                .unwrap_or(1000);
            self.characters[ci].base.injure(inj, fall, dvx, if dvy != 0.0 { dvy } else { -3.0 }, facing);
            self.specials[si].base.trans_frame(next_frame, 5);
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
                        spawns.push((op.oid, ch.base.team, x, y, ch.base.ps.z, ch.base.facing, op.action));
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
        for (oid, team, x, y, z, facing, action) in spawns {
            if let Some(data) = self.package_objects.get(&oid).cloned() {
                let mut s = SpecialAttack::new(self.next_uid, data, team, x, y, z, facing);
                self.next_uid += 1;
                if action != 0 {
                    s.base.trans_frame(action, 0);
                }
                self.specials.push(s);
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
            let wtype = self.weapons[wi].base.obj_type.clone();
            self.weapons[wi].held = true;
            self.weapons[wi].holder_uid = Some(self.characters[ci].base.uid);
            self.weapons[wi].base.team = self.characters[ci].base.team;
            self.characters[ci].hold_weapon = Some(uid);
            self.characters[ci].base.holding_uid = Some(uid);
            self.characters[ci].base.hold_type = wtype;
        }

        // sync held weapons to wpoint
        for ch in &self.characters {
            if let Some(wid) = ch.hold_weapon {
                if let Some((x, y, z, facing, wact)) = ch.wpoint_world() {
                    if let Some(w) = self.weapons.iter_mut().find(|w| w.base.uid == wid) {
                        w.attach_to(ch.base.uid, x, y, z, facing);
                        if wact > 0 {
                            w.base.trans_frame(wact, 2);
                        }
                    }
                }
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

        if self.paused {
            ren.fill_rect_color(0.0, 180.0, ren.width, 40.0, "rgba(0,0,0,0.5)");
            ren.fill_text("PAUSED", 360.0, 208.0, "#fff", "bold 28px sans-serif");
        }
        if self.game_over {
            ren.fill_rect_color(200.0, 150.0, 400.0, 100.0, "rgba(0,0,0,0.75)");
            ren.fill_text("GAME OVER", 300.0, 195.0, "#ff0", "bold 32px sans-serif");
            if let Some(t) = self.winner_team {
                ren.fill_text(
                    &format!("Team {} wins", t),
                    320.0,
                    225.0,
                    "#fff",
                    "18px sans-serif",
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
            ren.fill_rect_color(x, y, pane_w - 4.0, pane_h, "rgba(20,20,40,0.85)");
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
            ren.fill_text(&ch.base.name, x + 8.0, y + 48.0, "#afdcff", "11px sans-serif");
            // small face would go at x+5 — use first sheet head if any
        }
    }
}

struct SpriteDraw {
    sp: crate::core_engine::sprite::SpriteInstance,
    cx: f64,
    cy: f64,
}
