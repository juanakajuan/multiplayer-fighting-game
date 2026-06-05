pub const TICKS_PER_SECOND: u32 = 60;
pub const SECONDS_PER_TICK: f64 = 1.0 / (TICKS_PER_SECOND as f64);

pub struct GameState {
    tick: u64,
}

impl GameState {
    pub fn new() -> Self {
        Self { tick: 0 }
    }

    pub fn step_one_frame(&mut self) {
        self.tick += 1;
    }
}
