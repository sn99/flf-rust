//! LF2 street / stage campaign runtime (stage.dat semantics).
use serde_json::Value;

#[derive(Clone, Debug)]
pub struct StageSpawn {
    pub id: i32,
    pub hp: f64,
    pub x: Option<f64>,
    pub times: i32,
    pub ratio: f64,
    pub boss: bool,
    pub soldier: bool,
    pub join: Option<f64>,
    pub kind: SpawnKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpawnKind {
    Enemy,
    Object,
}

#[derive(Clone, Debug)]
pub struct StagePhase {
    pub bound: f64,
    pub spawns: Vec<StageSpawn>,
    pub when_clear_goto_phase: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct StageDef {
    pub id: i32,
    pub name: String,
    pub background: i32,
    pub phases: Vec<StagePhase>,
}

#[derive(Clone, Debug, Default)]
pub struct StageFile {
    pub stages: Vec<StageDef>,
    pub hp_scale: [f64; 4],
    pub times_scale: [f64; 4],
}

impl StageFile {
    pub fn from_json(v: &Value) -> Self {
        let diff = &v["difficulty"];
        let hp_scale = [
            diff["easy"]["hp"].as_f64().unwrap_or(0.75),
            diff["normal"]["hp"].as_f64().unwrap_or(1.0),
            diff["difficult"]["hp"].as_f64().unwrap_or(1.0),
            diff["crazy"]["hp"].as_f64().unwrap_or(1.5),
        ];
        let times_scale = [
            diff["easy"]["times"].as_f64().unwrap_or(1.0),
            diff["normal"]["times"].as_f64().unwrap_or(1.0),
            diff["difficult"]["times"].as_f64().unwrap_or(1.0),
            diff["crazy"]["times"].as_f64().unwrap_or(2.0),
        ];
        let mut stages = vec![];
        if let Some(arr) = v["stages"].as_array() {
            for s in arr {
                let mut phases = vec![];
                if let Some(parr) = s["phases"].as_array() {
                    for p in parr {
                        let mut spawns = vec![];
                        if let Some(sarr) = p["spawns"].as_array() {
                            for sp in sarr {
                                let kind = match sp["kind"].as_str() {
                                    Some("object") => SpawnKind::Object,
                                    _ => SpawnKind::Enemy,
                                };
                                spawns.push(StageSpawn {
                                    id: sp["id"].as_i64().unwrap_or(30) as i32,
                                    hp: sp["hp"].as_f64().unwrap_or(500.0),
                                    x: sp.get("x").and_then(|x| x.as_f64()),
                                    times: sp["times"].as_i64().unwrap_or(1) as i32,
                                    ratio: sp["ratio"].as_f64().unwrap_or(1.0),
                                    boss: sp["boss"].as_bool().unwrap_or(false),
                                    soldier: sp["soldier"].as_bool().unwrap_or(false),
                                    join: sp.get("join").and_then(|j| j.as_f64()),
                                    kind,
                                });
                            }
                        }
                        phases.push(StagePhase {
                            bound: p["bound"].as_f64().unwrap_or(1600.0),
                            spawns,
                            when_clear_goto_phase: p
                                .get("when_clear_goto_phase")
                                .and_then(|x| x.as_u64())
                                .map(|u| u as usize),
                        });
                    }
                }
                stages.push(StageDef {
                    id: s["id"].as_i64().unwrap_or(0) as i32,
                    name: s["name"].as_str().unwrap_or("Stage").to_string(),
                    background: s["background"].as_i64().unwrap_or(4) as i32,
                    phases,
                });
            }
        }
        Self {
            stages,
            hp_scale,
            times_scale,
        }
    }

    pub fn chapter(&self, index: usize) -> Option<&StageDef> {
        self.stages.get(index)
    }

    pub fn chapter_count(&self) -> usize {
        self.stages.iter().filter(|s| s.id < 50).count()
    }
}

/// Map manager difficulty (-1..2) to scale index 0..3 (easy..crazy)
pub fn difficulty_index(d: i8) -> usize {
    match d {
        d if d >= 2 => 0, // easy
        1 => 1,           // normal
        0 => 2,           // difficult
        _ => 3,           // crazy (-1)
    }
}

#[derive(Clone, Debug)]
pub struct PendingSpawn {
    pub id: i32,
    pub hp: f64,
    pub x: f64,
    pub z: f64,
    pub team: i32,
    pub is_ai: bool,
    pub boss: bool,
    pub soldier: bool,
    pub join_hp: Option<f64>,
    pub kind: SpawnKind,
    pub name: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StagePhaseState {
    Fighting,
    Go,
    Clearing,
    Complete,
}

pub struct StageRuntime {
    pub file: StageFile,
    pub chapter_index: usize,
    pub phase_index: usize,
    pub state: StagePhaseState,
    pub bound: f64,
    pub enemy_uids: Vec<u32>,
    pub boss_uids: Vec<u32>,
    pub soldier_template: Option<StageSpawn>,
    pub soldiers_spawned: i32,
    pub soldier_max: i32,
    pub go_timer: i32,
    pub message: String,
    pub message_ttl: i32,
    pub human_count: usize,
    pub difficulty: i8,
    pub pending: Vec<PendingSpawn>,
    pub joins_pending: Vec<(u32, f64)>,
    pub survival: bool,
    pub survival_wave: u32,
}

impl StageRuntime {
    pub fn start(file: StageFile, chapter_index: usize, human_count: usize, difficulty: i8) -> Option<Self> {
        let survival = file
            .chapter(chapter_index)
            .map(|c| c.id >= 50)
            .unwrap_or(false);
        let mut rt = Self {
            file,
            chapter_index,
            phase_index: 0,
            state: StagePhaseState::Fighting,
            bound: 1600.0,
            enemy_uids: vec![],
            boss_uids: vec![],
            soldier_template: None,
            soldiers_spawned: 0,
            soldier_max: 0,
            go_timer: 0,
            message: String::new(),
            message_ttl: 0,
            human_count: human_count.max(1),
            difficulty,
            pending: vec![],
            joins_pending: vec![],
            survival,
            survival_wave: 0,
        };
        rt.load_phase(0)?;
        Some(rt)
    }

    pub fn stage_def(&self) -> Option<&StageDef> {
        self.file.chapter(self.chapter_index)
    }

    pub fn label(&self) -> String {
        let name = self
            .stage_def()
            .map(|s| s.name.as_str())
            .unwrap_or("Stage");
        if self.survival {
            format!("Survival Stage: {}", self.survival_wave + 1)
        } else {
            format!("{} — phase {}", name, self.phase_index + 1)
        }
    }

    fn scales(&self) -> (f64, f64) {
        let i = difficulty_index(self.difficulty);
        (self.file.hp_scale[i], self.file.times_scale[i])
    }

    pub fn load_phase(&mut self, index: usize) -> Option<()> {
        let stage = self.file.chapter(self.chapter_index)?;
        let phase = stage.phases.get(index)?;
        self.phase_index = index;
        self.bound = phase.bound;
        self.state = StagePhaseState::Fighting;
        self.enemy_uids.clear();
        self.boss_uids.clear();
        self.soldier_template = None;
        self.soldiers_spawned = 0;
        self.soldier_max = 0;
        self.go_timer = 0;
        self.pending.clear();
        let (hp_s, times_s) = self.scales();
        let humans = self.human_count as f64;
        let (z0, z1) = (0.0, 1.0); // caller adjusts z
        let _ = (z0, z1);
        for (si, sp) in phase.spawns.iter().enumerate() {
            if sp.soldier {
                self.soldier_template = Some(sp.clone());
                self.soldier_max = ((sp.times as f64) * times_s).round().max(1.0) as i32;
                continue;
            }
            let count = spawn_count(sp, humans, times_s) as usize;
            for n in 0..count {
                let x = spawn_x(sp, phase.bound, n, si);
                let hp = if sp.kind == SpawnKind::Object {
                    0.0
                } else {
                    (sp.hp * hp_s).max(1.0)
                };
                self.pending.push(PendingSpawn {
                    id: resolve_random_id(sp.id),
                    hp,
                    x,
                    z: 0.5,
                    team: if sp.kind == SpawnKind::Object { 0 } else { 2 },
                    is_ai: true,
                    boss: sp.boss,
                    soldier: false,
                    join_hp: sp.join,
                    kind: sp.kind,
                    name: if sp.boss {
                        "Boss".into()
                    } else if sp.kind == SpawnKind::Object {
                        "Item".into()
                    } else {
                        "Enemy".into()
                    },
                });
            }
        }
        self.message = self.label();
        self.message_ttl = 90;
        Some(())
    }

    pub fn register_enemy(&mut self, uid: u32, boss: bool) {
        self.enemy_uids.push(uid);
        if boss {
            self.boss_uids.push(uid);
        }
    }

    pub fn on_enemy_removed(&mut self, uid: u32, join_hp: Option<f64>) {
        self.enemy_uids.retain(|&u| u != uid);
        self.boss_uids.retain(|&u| u != uid);
        if let Some(hp) = join_hp {
            self.joins_pending.push((uid, hp));
        }
    }

    pub fn tick_messages(&mut self) {
        if self.message_ttl > 0 {
            self.message_ttl -= 1;
        }
    }

    /// Returns soldier spawns to inject while bosses live.
    pub fn maybe_spawn_soldiers(&mut self, bosses_alive: bool, z_ratio: f64) -> Vec<PendingSpawn> {
        let mut out = vec![];
        if !bosses_alive || self.soldier_template.is_none() {
            return out;
        }
        if self.soldiers_spawned >= self.soldier_max {
            return out;
        }
        if self.enemy_uids.len() > self.boss_uids.len() + 2 {
            return out;
        }
        let sp = self.soldier_template.clone().unwrap();
        let (hp_s, _) = self.scales();
        let x = spawn_x(&sp, self.bound, self.soldiers_spawned as usize, 9);
        self.soldiers_spawned += 1;
        out.push(PendingSpawn {
            id: resolve_random_id(sp.id),
            hp: (sp.hp * hp_s).max(1.0),
            x,
            z: z_ratio,
            team: 2,
            is_ai: true,
            boss: false,
            soldier: true,
            join_hp: None,
            kind: SpawnKind::Enemy,
            name: "Soldier".into(),
        });
        out
    }

    pub fn enemies_cleared(&self) -> bool {
        self.enemy_uids.is_empty() && self.pending.is_empty() && self.state == StagePhaseState::Fighting
    }

    pub fn enter_go(&mut self) {
        self.state = StagePhaseState::Go;
        self.go_timer = 0;
        self.message = "GO!".into();
        self.message_ttl = 9999;
    }

    /// Player reached bound; advance phase/stage.
    pub fn advance(&mut self) -> StageAdvance {
        let stage = match self.file.chapter(self.chapter_index) {
            Some(s) => s.clone(),
            None => return StageAdvance::CampaignComplete,
        };
        let cur = match stage.phases.get(self.phase_index) {
            Some(p) => p.clone(),
            None => return StageAdvance::CampaignComplete,
        };
        if let Some(next) = cur.when_clear_goto_phase {
            self.survival_wave += 1;
            self.load_phase(next.min(stage.phases.len().saturating_sub(1)));
            return StageAdvance::NextPhase {
                background: stage.background,
                reload_background: false,
            };
        }
        let next_phase = self.phase_index + 1;
        if next_phase < stage.phases.len() {
            self.load_phase(next_phase);
            return StageAdvance::NextPhase {
                background: stage.background,
                reload_background: false,
            };
        }
        // next chapter
        let next_chapter = self.chapter_index + 1;
        if next_chapter < self.file.chapter_count() {
            self.chapter_index = next_chapter;
            self.survival_wave = 0;
            let bg = self.file.chapter(next_chapter).map(|c| c.background).unwrap_or(4);
            self.load_phase(0);
            self.message = format!("Stage clear! {}", self.label());
            self.message_ttl = 120;
            return StageAdvance::NextPhase {
                background: bg,
                reload_background: true,
            };
        }
        self.state = StagePhaseState::Complete;
        self.message = "Campaign complete!".into();
        self.message_ttl = 9999;
        StageAdvance::CampaignComplete
    }
}

#[derive(Clone, Debug)]
pub enum StageAdvance {
    NextPhase {
        background: i32,
        reload_background: bool,
    },
    CampaignComplete,
}

fn spawn_count(sp: &StageSpawn, humans: f64, times_s: f64) -> i32 {
    let base = (sp.times as f64) * times_s;
    let scaled = (base * sp.ratio * humans).round();
    scaled.max(1.0) as i32
}

fn spawn_x(sp: &StageSpawn, bound: f64, n: usize, salt: usize) -> f64 {
    if let Some(x) = sp.x {
        // LF2: x plus random up to 300
        x + ((n * 97 + salt * 13) % 300) as f64
    } else {
        // from the right, slightly past bound
        bound + 40.0 + ((n * 37 + salt * 11) % 120) as f64
    }
}

/// id 3000 → bandit; id 1000 → rotate heroes; else passthrough
fn resolve_random_id(id: i32) -> i32 {
    match id {
        3000 => 30,
        1000 => {
            const H: [i32; 10] = [1, 2, 4, 5, 6, 7, 8, 9, 10, 11];
            #[cfg(target_arch = "wasm32")]
            let i = (js_sys::Date::now() as usize) % H.len();
            #[cfg(not(target_arch = "wasm32"))]
            let i = 0usize;
            H[i]
        }
        _ => id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_minimal_stage() {
        let v = json!({
            "difficulty": {"easy":{"hp":0.75,"times":1.0},"normal":{"hp":1.0,"times":1.0},
                "difficult":{"hp":1.0,"times":1.0},"crazy":{"hp":1.5,"times":2.0}},
            "stages": [{"id":0,"name":"T","background":4,"phases":[
                {"bound":1600,"spawns":[{"id":30,"hp":50,"times":2,"ratio":1}]}
            ]}]
        });
        let f = StageFile::from_json(&v);
        assert_eq!(f.stages.len(), 1);
        assert_eq!(f.stages[0].phases[0].bound, 1600.0);
        let mut rt = StageRuntime::start(f, 0, 1, 1).unwrap();
        assert_eq!(rt.pending.len(), 2);
        assert_eq!(rt.bound, 1600.0);
        rt.enemy_uids.push(1);
        assert!(!rt.enemies_cleared());
        rt.pending.clear();
        rt.enemy_uids.clear();
        assert!(rt.enemies_cleared());
        rt.enter_go();
        assert_eq!(rt.state, StagePhaseState::Go);
        let adv = rt.advance();
        assert!(matches!(adv, StageAdvance::CampaignComplete) || matches!(adv, StageAdvance::NextPhase { .. }) || rt.phase_index == 0);
    }
}
