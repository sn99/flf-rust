//! Frame transistor — LF/livingobject.js frame_transistor (authority locks)
#[derive(Debug, Clone)]
pub struct FrameTransistor {
    pub wait: i32,
    pub next: i32,
    pub lock: i32,
    pub lockout: i32,
    pub switch_dir_after: bool,
}

impl Default for FrameTransistor {
    fn default() -> Self {
        Self {
            wait: 1,
            next: 999,
            lock: 0,
            lockout: 1,
            switch_dir_after: false,
        }
    }
}

impl FrameTransistor {
    /// Force frame F with authority au (calls set_next + set_wait(0))
    pub fn frame(&mut self, f: i32, au: i32) {
        self.set_next(f, au);
        self.set_wait(0, au, 1);
    }

    pub fn set_wait(&mut self, value: i32, mut au: i32, out: i32) {
        if au == 99 {
            au = self.lock;
        }
        let out = if out == 0 { 1 } else { out };
        if au >= self.lock {
            self.lock = au;
            self.lockout = if out == 99 { self.wait } else { out };
            self.wait = value.max(0);
        }
    }

    pub fn inc_wait(&mut self, dv: i32, au: i32, out: i32) {
        let w = self.wait + dv;
        self.set_wait(w, au, out);
    }

    pub fn set_next(&mut self, mut f: i32, mut au: i32) {
        if au == 99 {
            au = self.lock;
        }
        if au >= self.lock {
            self.lock = au;
            // negative next = switch dir after transition
            self.switch_dir_after = f < 0;
            if f < 0 {
                f = -f;
            }
            self.next = f;
        }
    }

    /// Natural frame step at end of wait; returns Some(frame) to transit to
    pub fn tick_wait(&mut self) -> Option<i32> {
        if self.lockout > 0 {
            self.lockout -= 1;
            if self.lockout == 0 {
                self.lock = 0;
            }
        }
        if self.wait > 0 {
            self.wait -= 1;
            None
        } else {
            let n = self.next;
            Some(n)
        }
    }

    pub fn reset_lock(&mut self) {
        self.lock = 0;
        self.lockout = 0;
    }
}
