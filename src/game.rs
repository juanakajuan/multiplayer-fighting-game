pub const TICKS_PER_SECOND: u32 = 60;
pub const SECONDS_PER_TICK: f64 = 1.0 / (TICKS_PER_SECOND as f64);

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PlayerInput {
    pub move_left: bool,
    pub move_right: bool,
    pub jump: bool,
    pub attack: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FrameInput {
    pub player_one: PlayerInput,
    pub player_two: PlayerInput,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GameState {
    tick: u64,
}

impl GameState {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn tick(&self) -> u64 {
        self.tick
    }

    pub fn step(&mut self, _input: FrameInput) {
        self.tick = self.tick.checked_add(1).expect("tick counter overflow");
    }
}

#[cfg(test)]
mod tests {
    use super::{FrameInput, GameState};

    #[test]
    fn neutral_step_advances_one_tick() {
        let mut state = GameState::new();

        state.step(FrameInput::default());

        assert_eq!(state.tick(), 1);
    }
}
