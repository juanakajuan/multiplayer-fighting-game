pub const TICKS_PER_SECOND: u32 = 60;
pub const SECONDS_PER_TICK: f64 = 1.0 / (TICKS_PER_SECOND as f64);
pub const ARENA_WIDTH: i32 = 1280;
pub const ARENA_HEIGHT: i32 = 720;
pub const ARENA_GROUND_Y: i32 = 620;

const FIGHTER_WIDTH: i32 = 72;
const FIGHTER_HEIGHT: i32 = 144;
const PLAYER_ONE_START_X: i32 = 420;
const PLAYER_TWO_START_X: i32 = 860;

pub const DEFAULT_ARENA: Arena = Arena {
    width: ARENA_WIDTH,
    height: ARENA_HEIGHT,
    ground_y: ARENA_GROUND_Y,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Arena {
    pub width: i32,
    pub height: i32,
    pub ground_y: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FighterState {
    body: Rect,
}

impl FighterState {
    #[must_use]
    pub const fn grounded_at_center(center_x: i32, ground_y: i32) -> Self {
        Self {
            body: Rect {
                x: center_x - (FIGHTER_WIDTH / 2),
                y: ground_y - FIGHTER_HEIGHT,
                width: FIGHTER_WIDTH,
                height: FIGHTER_HEIGHT,
            },
        }
    }

    #[must_use]
    pub const fn body(&self) -> Rect {
        self.body
    }
}

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameState {
    tick: u64,
    arena: Arena,
    player_one: FighterState,
    player_two: FighterState,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            tick: 0,
            arena: DEFAULT_ARENA,
            player_one: FighterState::grounded_at_center(PLAYER_ONE_START_X, ARENA_GROUND_Y),
            player_two: FighterState::grounded_at_center(PLAYER_TWO_START_X, ARENA_GROUND_Y),
        }
    }
}

impl GameState {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub const fn arena(&self) -> Arena {
        self.arena
    }

    #[must_use]
    pub const fn player_one(&self) -> FighterState {
        self.player_one
    }

    #[must_use]
    pub const fn player_two(&self) -> FighterState {
        self.player_two
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

        assert_eq!(state.tick, 1);
    }
}
