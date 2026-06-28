//! Sound playback via Web Audio (optional)
pub struct Soundpack {
    pub enabled: bool,
}

impl Soundpack {
    pub fn new() -> Self { Self { enabled: false } }
    pub fn play(&self, _path: &str) {
        // TODO: WebAudio buffer playback
    }
}
