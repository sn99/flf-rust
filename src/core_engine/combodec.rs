use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct ComboDef {
    pub name: String,
    pub seq: Vec<String>,
    pub clear_on_combo: bool,
}

pub struct ComboDecoder {
    pub combos: Vec<ComboDef>,
    history: VecDeque<(String, u32)>,
    timeout: u32,
    age: u32,
}

impl ComboDecoder {
    pub fn new(combos: Vec<ComboDef>, timeout: u32) -> Self {
        Self { combos, history: VecDeque::new(), timeout, age: 0 }
    }

    pub fn tick(&mut self) {
        self.age = self.age.saturating_add(1);
        while let Some((_, t)) = self.history.front() {
            if self.age.saturating_sub(*t) > self.timeout {
                self.history.pop_front();
            } else {
                break;
            }
        }
    }

    pub fn feed(&mut self, action: &str) {
        self.history.push_back((action.to_string(), self.age));
        if self.history.len() > 16 {
            self.history.pop_front();
        }
    }

    /// Returns highest priority matching combo name
    pub fn match_combo(&self) -> Option<String> {
        let hist: Vec<&str> = self.history.iter().map(|(a, _)| a.as_str()).collect();
        let mut best: Option<(usize, String)> = None;
        for c in &self.combos {
            let seq = &c.seq;
            if seq.is_empty() { continue; }
            if hist.len() < seq.len() { continue; }
            let slice = &hist[hist.len() - seq.len()..];
            if slice.iter().zip(seq.iter()).all(|(a, b)| *a == b.as_str()) {
                let score = seq.len();
                if best.as_ref().map(|(s, _)| score > *s).unwrap_or(true) {
                    best = Some((score, c.name.clone()));
                }
            }
        }
        best.map(|(_, n)| n)
    }

    pub fn clear(&mut self) {
        self.history.clear();
    }
}
