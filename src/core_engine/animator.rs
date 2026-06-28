/// Frame animator driving wait/next transitions (LF2 TU timing)
#[derive(Clone, Debug)]
pub struct Animator {
    pub frame: i32,
    pub wait_left: i32,
    pub paused: bool,
}

impl Default for Animator {
    fn default() -> Self {
        Self { frame: 0, wait_left: 0, paused: false }
    }
}

impl Animator {
    pub fn set_frame(&mut self, frame: i32, wait: i32) {
        self.frame = frame;
        self.wait_left = wait;
    }

    /// Advance one TU; returns Some(next_frame_hint) when wait expires (caller applies `next`)
    pub fn tu(&mut self) -> bool {
        if self.paused { return false; }
        if self.wait_left > 0 {
            self.wait_left -= 1;
            false
        } else {
            true
        }
    }
}
