//! Match host — runs a VS game (LF/match.js)
use crate::core_engine::controller::Controller;
use crate::core_engine::sprite::CanvasRenderer;
use crate::lf::background::Background;
use crate::lf::character::Character;
use crate::lf::data::ObjectData;
use crate::lf::global;
use crate::lf::livingobject::LivingObject;
use crate::lf::mechanics::Mech;
use crate::lf::package::Package;
use crate::lf::specialattack::SpecialAttack;
use crate::lf::weapon::Weapon;
use std::cell::RefCell;
use std::rc::Rc;

pub struct PlayerSetup {
    pub id: i32,
    pub team: i32,
    pub controller_index: Option<usize>,
    pub is_ai: bool,
}

pub struct Match {
    pub characters: Vec<Character>,
    pub weapons: Vec<Weapon>,
    pub specials: Vec<SpecialAttack>,
    pub background: Background,
    pub next_uid: u32,
    pub time: u32,
    pub game_over: bool,
    pub paused: bool,
    pub camera_x: f64,
    controllers: Rc<RefCell<Vec<Controller>>>,
    ai_controllers: Vec<Controller>,
    package_objects: std::collections::HashMap<i32, ObjectData>,
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
            .unwrap_or(serde_json::json!({"width": 1200, "zboundary": [320, 490], "name": "default"}));
        let background = Background::from_json(background_id, &bg_val);
        let (z0, z1) = background.zboundary;
        let mid_z = (z0 + z1) / 2.0;

        let mut characters = vec![];
        let mut next_uid = 1u32;
        let mut ai_controllers = vec![];

        let n = players.len().max(1) as f64;
        for (i, p) in players.iter().enumerate() {
            let data = package
                .objects
                .get(&p.id)
                .cloned()
                .ok_or_else(|| format!("character id {} not loaded", p.id))?;
            let x = 200.0 + (i as f64) * (400.0 / n);
            let mut ch = Character::new(next_uid, data, p.team, x, mid_z);
            next_uid += 1;
            ch.base.controller_index = p.controller_index;
            ch.base.ai = p.is_ai;
            if p.is_ai {
                let mut cfg = std::collections::HashMap::new();
                for k in ["up", "down", "left", "right", "def", "jump", "att"] {
                    cfg.insert(k.to_string(), k.to_string());
                }
                ai_controllers.push(Controller::new_keyboard(cfg));
            }
            if i % 2 == 1 {
                ch.base.facing = -1;
            }
            characters.push(ch);
        }

        let mut weapons = vec![];
        if drop_weapons {
            for wid in [100i32, 101, 150] {
                if let Some(data) = package.objects.get(&wid).cloned() {
                    let w = Weapon::new(next_uid, data, 400.0 + weapons.len() as f64 * 50.0, mid_z);
                    next_uid += 1;
                    weapons.push(w);
                }
            }
        }

        Ok(Self {
            characters,
            weapons,
            specials: vec![],
            background,
            next_uid,
            time: 0,
            game_over: false,
            paused: false,
            camera_x: 0.0,
            controllers,
            ai_controllers,
            package_objects: package.objects.clone(),
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

        // Snapshot targets for AI (team, x, z) per character index
        let snapshot: Vec<(i32, f64, f64, bool)> = self
            .characters
            .iter()
            .map(|c| (c.base.team, c.base.ps.x, c.base.ps.z, c.base.dead))
            .collect();

        // Character TU with controllers
        let ctrls = self.controllers.borrow();
        let time = self.time;
        for (i, ch) in self.characters.iter_mut().enumerate() {
            if ch.base.ai {
                let mut cfg = std::collections::HashMap::new();
                for k in ["up", "down", "left", "right", "def", "jump", "att"] {
                    cfg.insert(k.to_string(), k.to_string());
                }
                let mut ac = Controller::new_keyboard(cfg);
                let team = snapshot[i].0;
                let sx = snapshot[i].1;
                let sz = snapshot[i].2;
                if let Some((_, tx, tz, _)) = snapshot
                    .iter()
                    .enumerate()
                    .filter(|(j, (t, _, _, dead))| *j != i && *t != team && !*dead)
                    .map(|(_, s)| s)
                    .next()
                {
                    let dx = tx - sx;
                    let dz = tz - sz;
                    if dx.abs() > 40.0 {
                        if dx > 0.0 {
                            ac.keypress("right");
                        } else {
                            ac.keypress("left");
                        }
                    }
                    if dz.abs() > 8.0 {
                        if dz > 0.0 {
                            ac.keypress("down");
                        } else {
                            ac.keypress("up");
                        }
                    }
                    if dx.abs() < 60.0 && dz.abs() < 15.0 {
                        ac.keypress("att");
                    }
                } else if time % 30 < 10 {
                    ac.keypress("right");
                } else if time % 30 < 20 {
                    ac.keypress("left");
                }
                if time % 45 == 0 {
                    ac.keypress("att");
                }
                ch.tu(Some(&ac), bg_z, bg_w);
            } else if let Some(ci) = ch.base.controller_index {
                let c = ctrls.get(ci);
                ch.tu(c, bg_z, bg_w);
            } else {
                ch.tu(None, bg_z, bg_w);
            }
        }
        drop(ctrls);

        for w in &mut self.weapons {
            w.tu(bg_z, bg_w);
        }
        for s in &mut self.specials {
            s.tu(bg_z, bg_w);
        }
        self.specials.retain(|s| !s.base.dead);

        self.process_hits();
        self.spawn_opoints();
        self.update_camera();
        self.check_game_over();
    }

    fn process_hits(&mut self) {
        // Collect itrs and bdys
        let n = self.characters.len();
        let mut events: Vec<(usize, usize, f64, f64, f64, f64)> = vec![]; // attacker, victim, injury, fall, dvx, dvy

        for i in 0..n {
            if self.characters[i].base.dead || self.characters[i].base.arest > 0 {
                continue;
            }
            let Some(frame) = self.characters[i].base.frame().cloned() else {
                continue;
            };
            let itrs = Mech::itr_volumes(
                &self.characters[i].base.ps,
                self.characters[i].base.facing,
                &frame,
            );
            if itrs.is_empty() {
                continue;
            }
            for j in 0..n {
                if i == j || self.characters[j].base.dead {
                    continue;
                }
                if self.characters[i].base.team == self.characters[j].base.team
                    && self.characters[i].base.team != 0
                {
                    continue;
                }
                // vrest
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
                let Some(vframe) = self.characters[j].base.frame().cloned() else {
                    continue;
                };
                let bdys = Mech::body_volumes(
                    &self.characters[j].base.ps,
                    self.characters[j].base.facing,
                    &vframe,
                );
                for (vol, itr) in &itrs {
                    if itr.kind != 0 {
                        continue; // only normal hit kind 0 for now; kind 1 catch etc. later
                    }
                    for b in &bdys {
                        if vol.intersects(b) {
                            let injury = if itr.injury != 0.0 {
                                itr.injury
                            } else {
                                20.0
                            };
                            let fall = if itr.fall != 0.0 {
                                itr.fall
                            } else {
                                global::DEFAULT_FALL
                            };
                            events.push((i, j, injury, fall, vol.vx, itr.dvy));
                        }
                    }
                }
            }
        }

        for (i, j, injury, fall, dvx, dvy) in events {
            let facing = self.characters[i].base.facing;
            self.characters[i].base.arest = global::DEFAULT_AREST;
            let vid = self.characters[j].base.uid;
            self.characters[i]
                .base
                .vrest
                .insert(vid, global::DEFAULT_VREST);
            // defend?
            let defending = self.characters[j].base.state() == 7
                || self.characters[j].base.frame_id >= 110 && self.characters[j].base.frame_id <= 111;
            let mut inj = injury;
            if defending && self.characters[j].base.facing != facing {
                // facing attacker
                inj *= global::DEFEND_INJURY_FACTOR;
                self.characters[j].base.bdefend += injury;
                if self.characters[j].base.bdefend < global::DEFEND_BREAK_LIMIT {
                    continue;
                }
            }
            self.characters[j]
                .base
                .injure(inj, fall, dvx, if dvy != 0.0 { dvy } else { global::DEFAULT_FALL_DVY });
            self.characters[j].base.blink = 8;
        }
    }

    fn spawn_opoints(&mut self) {
        let mut spawns: Vec<(i32, i32, f64, f64, f64, i32)> = vec![]; // oid, team, x,y,z, facing
        for ch in &self.characters {
            if let Some(fd) = ch.base.frame() {
                if let Some(op) = &fd.opoint {
                    // spawn once per frame wait start — approximate: when wait_left == wait
                    if ch.base.animator.wait_left == fd.wait && op.oid != 0 {
                        let x = ch.base.ps.x + op.x * ch.base.facing as f64;
                        let y = ch.base.ps.y + op.y;
                        spawns.push((
                            op.oid,
                            ch.base.team,
                            x,
                            y,
                            ch.base.ps.z,
                            ch.base.facing,
                        ));
                    }
                }
            }
        }
        for (oid, team, x, y, z, facing) in spawns {
            if let Some(data) = self.package_objects.get(&oid).cloned() {
                let mut s = SpecialAttack::new(self.next_uid, data, team, x, y, z, facing);
                self.next_uid += 1;
                if let Some(fd) = s.base.frame().cloned() {
                    s.base.vx = fd.dvx * facing as f64;
                    s.base.vy = fd.dvy;
                }
                self.specials.push(s);
            }
        }
    }

    fn update_camera(&mut self) {
        if self.characters.is_empty() {
            return;
        }
        let mut min_x = f64::MAX;
        let mut max_x = f64::MIN;
        for ch in &self.characters {
            if ch.base.dead {
                continue;
            }
            min_x = min_x.min(ch.base.ps.x);
            max_x = max_x.max(ch.base.ps.x);
        }
        if min_x > max_x {
            return;
        }
        let mid = (min_x + max_x) / 2.0;
        let target = (mid - global::WINDOW_WIDTH / 2.0)
            .max(0.0)
            .min((self.background.width - global::WINDOW_WIDTH).max(0.0));
        self.camera_x += (target - self.camera_x) * global::CAMERA_SPEED_FACTOR * 3.0;
    }

    fn check_game_over(&mut self) {
        let mut teams_alive = std::collections::HashSet::new();
        for ch in &self.characters {
            if !ch.base.dead && ch.base.hp > 0.0 {
                teams_alive.insert(ch.base.team);
            }
        }
        // also mark dead if hp 0 long
        for ch in &mut self.characters {
            if ch.base.hp <= 0.0 && ch.base.state() >= 180 {
                // lying
                if ch.base.frame_id >= 180 && ch.base.animator.wait_left == 0 && ch.base.time_dead() {
                    ch.base.dead = true;
                }
            }
        }
        if teams_alive.len() <= 1 && self.time > 60 {
            self.game_over = true;
        }
    }

    pub fn render(&mut self, ren: &mut CanvasRenderer) {
        ren.cam_x = self.camera_x;
        ren.cam_y = 0.0;
        self.background.draw(ren);

        // shadows
        for ch in &self.characters {
            if ch.base.dead {
                continue;
            }
            let sx = ch.base.ps.x - self.camera_x - 20.0;
            let sy = ch.base.ps.z - 5.0;
            ren.ctx.set_fill_style_str("rgba(0,0,0,0.35)");
            ren.ctx.begin_path();
            let _ = ren.ctx.ellipse(sx + 20.0, sy, 22.0, 8.0, 0.0, 0.0, std::f64::consts::TAU);
            ren.ctx.fill();
        }

        // collect draw list
        let mut items: Vec<(f64, f64, SpriteDraw)> = vec![];
        for ch in &self.characters {
            if let Some(fd) = ch.base.frame() {
                items.push((
                    ch.base.ps.z,
                    ch.base.ps.y,
                    SpriteDraw {
                        sp: ch.base.sp.clone(),
                        cx: fd.centerx,
                        cy: fd.centery,
                    },
                ));
            }
        }
        for w in &self.weapons {
            if let Some(fd) = w.base.frame() {
                items.push((
                    w.base.ps.z,
                    w.base.ps.y,
                    SpriteDraw {
                        sp: w.base.sp.clone(),
                        cx: fd.centerx,
                        cy: fd.centery,
                    },
                ));
            }
        }
        for s in &self.specials {
            if let Some(fd) = s.base.frame() {
                items.push((
                    s.base.ps.z,
                    s.base.ps.y,
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
                .then(a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        });
        // fix Equal
        let _ = 0;

        for (_, _, d) in &items {
            ren.draw_sprite(&d.sp, d.cx, d.cy);
        }

        // HP bars
        for (i, ch) in self.characters.iter().enumerate() {
            let x = 10.0 + i as f64 * 200.0;
            let y = 8.0;
            ren.ctx.set_fill_style_str("#333");
            ren.ctx.fill_rect(x, y, 160.0, 14.0);
            let pct = (ch.base.hp / ch.base.hp_full).clamp(0.0, 1.0);
            ren.ctx.set_fill_style_str(if pct > 0.3 { "#3c3" } else { "#c33" });
            ren.ctx.fill_rect(x, y, 160.0 * pct, 14.0);
            ren.ctx.set_fill_style_str("#339");
            let mpct = (ch.base.mp / ch.base.mp_full).clamp(0.0, 1.0);
            ren.ctx.fill_rect(x, y + 14.0, 160.0 * mpct, 6.0);
            ren.fill_text(
                &ch.base.name,
                x,
                y + 32.0,
                "#fff",
                "12px sans-serif",
            );
        }

        if self.paused {
            ren.fill_text("PAUSED", 350.0, 200.0, "#fff", "bold 32px sans-serif");
        }
        if self.game_over {
            ren.fill_text("GAME OVER", 300.0, 200.0, "#ff0", "bold 36px sans-serif");
            ren.fill_text("Press F4 or Esc for menu", 280.0, 240.0, "#fff", "16px sans-serif");
        }
    }
}

struct SpriteDraw {
    sp: crate::core_engine::sprite::SpriteInstance,
    cx: f64,
    cy: f64,
}

// Extension: avoid unimplemented method
trait DeadCheck {
    fn time_dead(&self) -> bool;
}
impl DeadCheck for LivingObject {
    fn time_dead(&self) -> bool {
        self.hp <= 0.0
    }
}
