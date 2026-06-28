/// Pool of short-lived visual effects (blood, blast)
#[derive(Clone, Debug)]
pub struct PooledEffect {
    pub active: bool,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub pic: i32,
    pub life: i32,
    pub effect_id: i32,
}

pub struct EffectsPool {
    pub effects: Vec<PooledEffect>,
}

impl EffectsPool {
    pub fn new(capacity: usize) -> Self {
        Self {
            effects: (0..capacity)
                .map(|_| PooledEffect {
                    active: false, x: 0.0, y: 0.0, z: 0.0, pic: 0, life: 0, effect_id: 0,
                })
                .collect(),
        }
    }

    pub fn spawn(&mut self, effect_id: i32, x: f64, y: f64, z: f64, life: i32) {
        if let Some(e) = self.effects.iter_mut().find(|e| !e.active) {
            e.active = true;
            e.effect_id = effect_id;
            e.x = x;
            e.y = y;
            e.z = z;
            e.life = life;
            e.pic = 0;
        }
    }

    pub fn tu(&mut self) {
        for e in &mut self.effects {
            if e.active {
                e.life -= 1;
                e.pic += 1;
                if e.life <= 0 {
                    e.active = false;
                }
            }
        }
    }
}
