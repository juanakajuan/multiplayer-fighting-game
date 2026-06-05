//! Headless, deterministic gameplay simulation.
//!
//! Keep rendering and device input outside this module so the same fixed-tick
//! state transition can later be driven by local controls or networked inputs.

/// Number of simulation ticks advanced per second.
pub const TICKS_PER_SECOND: u32 = 60;

/// Wall-clock duration of one fixed simulation tick.
pub const SECONDS_PER_TICK: f64 = 1.0 / (TICKS_PER_SECOND as f64);

/// Arena width in pixels.
pub const ARENA_WIDTH: i32 = 1280;

/// Arena height in pixels.
pub const ARENA_HEIGHT: i32 = 720;

/// Screen-space y coordinate of the top of the floor.
pub const ARENA_GROUND_Y: i32 = 620;

const FIGHTER_WIDTH: i32 = 72;
const FIGHTER_HEIGHT: i32 = 144;
const FIGHTER_HORIZONTAL_SPEED_PER_TICK: i32 = 6;
const FIGHTER_JUMP_SPEED_PER_TICK: i32 = -28;
const FIGHTER_GRAVITY_PER_TICK: i32 = 2;
const PLAYER_ONE_START_X: i32 = 420;
const PLAYER_TWO_START_X: i32 = 860;

/// Startup duration for the current standing melee attack.
pub const STANDING_ATTACK_STARTUP_TICKS: u32 = 6;

/// Active duration for the current standing melee attack.
pub const STANDING_ATTACK_ACTIVE_TICKS: u32 = 4;

/// Recovery duration for the current standing melee attack.
pub const STANDING_ATTACK_RECOVERY_TICKS: u32 = 12;

/// Default one-screen arena used by the current local match.
pub const DEFAULT_ARENA: Arena = Arena {
    width: ARENA_WIDTH,
    height: ARENA_HEIGHT,
    ground_y: ARENA_GROUND_Y,
};

/// Static arena bounds for a flat, one-screen stage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Arena {
    /// Width in pixels.
    pub width: i32,
    /// Height in pixels.
    pub height: i32,
    /// Screen-space y coordinate where fighters stand.
    pub ground_y: i32,
}

/// Integer axis-aligned rectangle in screen-space pixels.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Rect {
    /// Left edge in pixels.
    pub x: i32,
    /// Top edge in pixels.
    pub y: i32,
    /// Width in pixels.
    pub width: i32,
    /// Height in pixels.
    pub height: i32,
}

/// Vulnerable fighter area used for combat collision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Hurtbox {
    rect: Rect,
}

/// Per-fighter gameplay state owned by [`GameState`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FighterState {
    body: Rect,
    facing_direction: FacingDirection,
    vertical_velocity_per_tick: i32,
    standing_attack: Option<StandingAttackState>,
}

impl Hurtbox {
    /// Rectangle occupied by this hurtbox in screen-space pixels.
    #[must_use]
    pub const fn rect(&self) -> Rect {
        self.rect
    }
}

/// Current phase of a standing melee attack.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttackPhase {
    /// Pre-hit frames before the strike is active.
    Startup,
    /// Frames where the strike can later own an attack hitbox.
    Active,
    /// Post-hit frames before the fighter returns to neutral.
    Recovery,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct StandingAttackState {
    phase: AttackPhase,
    ticks_in_phase: u32,
}

/// Horizontal direction a fighter is currently facing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FacingDirection {
    /// Facing screen-left.
    Left,
    /// Facing screen-right.
    Right,
}

impl FighterState {
    /// Creates a grounded fighter whose body is centered on `center_x`.
    #[must_use]
    pub const fn grounded_at_center(
        center_x: i32,
        ground_y: i32,
        facing_direction: FacingDirection,
    ) -> Self {
        Self {
            body: Rect {
                x: center_x - (FIGHTER_WIDTH / 2),
                y: ground_y - FIGHTER_HEIGHT,
                width: FIGHTER_WIDTH,
                height: FIGHTER_HEIGHT,
            },
            facing_direction,
            vertical_velocity_per_tick: 0,
            standing_attack: None,
        }
    }

    /// Current visible body rectangle.
    #[must_use]
    pub const fn body(&self) -> Rect {
        self.body
    }

    /// Current vulnerable area for combat collision.
    #[must_use]
    pub const fn hurtbox(&self) -> Hurtbox {
        Hurtbox { rect: self.body }
    }

    /// Current horizontal facing direction.
    #[must_use]
    pub const fn facing_direction(&self) -> FacingDirection {
        self.facing_direction
    }

    /// Whether the fighter is performing the simple standing melee attack this tick.
    #[must_use]
    pub const fn is_performing_standing_attack(&self) -> bool {
        self.standing_attack.is_some()
    }

    /// Current standing attack phase, if the fighter is attacking.
    #[must_use]
    pub const fn standing_attack_phase(&self) -> Option<AttackPhase> {
        match self.standing_attack {
            Some(standing_attack) => Some(standing_attack.phase),
            None => None,
        }
    }

    /// Whether the standing attack is in its active frames this tick.
    #[must_use]
    pub const fn is_standing_attack_active(&self) -> bool {
        matches!(self.standing_attack_phase(), Some(AttackPhase::Active))
    }

    fn move_horizontally(&mut self, direction: i32, arena: Arena) {
        let max_x = (arena.width - self.body.width).max(0);
        self.body.x =
            (self.body.x + (direction * FIGHTER_HORIZONTAL_SPEED_PER_TICK)).clamp(0, max_x);
    }

    fn move_vertically(&mut self, jump: bool, arena: Arena) {
        // Apply jump impulse before gravity so jumps visibly begin on this tick.
        if jump && self.is_grounded(arena) {
            self.vertical_velocity_per_tick = FIGHTER_JUMP_SPEED_PER_TICK;
        }

        self.body.y += self.vertical_velocity_per_tick;
        self.vertical_velocity_per_tick += FIGHTER_GRAVITY_PER_TICK;

        let ground_top = arena.ground_y - self.body.height;
        if self.body.y >= ground_top {
            self.body.y = ground_top;
            self.vertical_velocity_per_tick = 0;
        }
    }

    fn is_grounded(&self, arena: Arena) -> bool {
        self.body.y + self.body.height >= arena.ground_y
    }

    fn update_standing_attack(&mut self, attack: bool, arena: Arena) {
        if let Some(standing_attack) = self.standing_attack {
            self.standing_attack = standing_attack.advance();
        } else if attack && self.is_grounded(arena) {
            self.standing_attack = Some(StandingAttackState::new());
        }
    }
}

impl StandingAttackState {
    const fn new() -> Self {
        Self {
            phase: AttackPhase::Startup,
            ticks_in_phase: 1,
        }
    }

    /// Advances the standing attack by one fixed tick.
    ///
    /// Returns the next attack state, or `None` once recovery has finished.
    fn advance(self) -> Option<Self> {
        if self.ticks_in_phase < self.phase.duration_ticks() {
            return Some(Self {
                ticks_in_phase: self.ticks_in_phase + 1,
                ..self
            });
        }

        self.phase.next().map(|phase| Self {
            phase,
            ticks_in_phase: 1,
        })
    }
}

impl AttackPhase {
    const fn duration_ticks(self) -> u32 {
        match self {
            Self::Startup => STANDING_ATTACK_STARTUP_TICKS,
            Self::Active => STANDING_ATTACK_ACTIVE_TICKS,
            Self::Recovery => STANDING_ATTACK_RECOVERY_TICKS,
        }
    }

    const fn next(self) -> Option<Self> {
        match self {
            Self::Startup => Some(Self::Active),
            Self::Active => Some(Self::Recovery),
            Self::Recovery => None,
        }
    }
}

/// Raw controls sampled for one player during a single fixed tick.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PlayerInput {
    /// Request movement toward screen-left.
    pub move_left: bool,
    /// Request movement toward screen-right.
    pub move_right: bool,
    /// Request a jump if the fighter is grounded.
    pub jump: bool,
    /// Request a grounded standing attack if the fighter is neutral.
    pub attack: bool,
}

/// Complete input snapshot consumed by one [`GameState::step`] call.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FrameInput {
    /// Player one's input for this tick.
    pub player_one: PlayerInput,
    /// Player two's input for this tick.
    pub player_two: PlayerInput,
}

/// Complete deterministic match state for the current local game.
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
            player_one: FighterState::grounded_at_center(
                PLAYER_ONE_START_X,
                ARENA_GROUND_Y,
                FacingDirection::Right,
            ),
            player_two: FighterState::grounded_at_center(
                PLAYER_TWO_START_X,
                ARENA_GROUND_Y,
                FacingDirection::Left,
            ),
        }
    }
}

impl GameState {
    /// Creates a new local match at the default starting positions.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Arena used for movement constraints and rendering.
    #[must_use]
    pub const fn arena(&self) -> Arena {
        self.arena
    }

    /// Current state for player one.
    #[must_use]
    pub const fn player_one(&self) -> FighterState {
        self.player_one
    }

    /// Current state for player two.
    #[must_use]
    pub const fn player_two(&self) -> FighterState {
        self.player_two
    }

    /// Advances gameplay by exactly one fixed tick.
    pub fn step(&mut self, input: FrameInput) {
        self.player_one
            .move_horizontally(horizontal_direction(input.player_one), self.arena);
        self.player_one
            .move_vertically(input.player_one.jump, self.arena);
        self.player_one
            .update_standing_attack(input.player_one.attack, self.arena);
        self.player_two
            .move_horizontally(horizontal_direction(input.player_two), self.arena);
        self.player_two
            .move_vertically(input.player_two.jump, self.arena);
        self.player_two
            .update_standing_attack(input.player_two.attack, self.arena);
        self.update_facing_directions();
        self.tick = self.tick.checked_add(1).expect("tick counter overflow");
    }

    fn update_facing_directions(&mut self) {
        let player_one_center_x = center_x(self.player_one.body);
        let player_two_center_x = center_x(self.player_two.body);

        if player_one_center_x < player_two_center_x {
            self.player_one.facing_direction = FacingDirection::Right;
            self.player_two.facing_direction = FacingDirection::Left;
        } else if player_one_center_x > player_two_center_x {
            self.player_one.facing_direction = FacingDirection::Left;
            self.player_two.facing_direction = FacingDirection::Right;
        }
    }
}

fn center_x(rect: Rect) -> i32 {
    rect.x + (rect.width / 2)
}

fn horizontal_direction(input: PlayerInput) -> i32 {
    match (input.move_left, input.move_right) {
        (true, false) => -1,
        (false, true) => 1,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AttackPhase, FIGHTER_HORIZONTAL_SPEED_PER_TICK, FIGHTER_JUMP_SPEED_PER_TICK,
        FacingDirection, FrameInput, GameState, PlayerInput, STANDING_ATTACK_ACTIVE_TICKS,
        STANDING_ATTACK_RECOVERY_TICKS, STANDING_ATTACK_STARTUP_TICKS,
    };

    #[test]
    fn neutral_step_advances_one_tick() {
        let mut state = GameState::new();

        state.step(FrameInput::default());

        assert_eq!(state.tick, 1);
    }

    #[test]
    fn fighters_start_facing_each_other() {
        let state = GameState::new();

        assert_eq!(
            state.player_one().facing_direction(),
            FacingDirection::Right
        );
        assert_eq!(state.player_two().facing_direction(), FacingDirection::Left);
    }

    #[test]
    fn fighters_turn_to_face_each_other_after_crossing() {
        let mut state = GameState::new();
        let input = FrameInput {
            player_one: PlayerInput {
                move_right: true,
                ..PlayerInput::default()
            },
            player_two: PlayerInput {
                move_left: true,
                ..PlayerInput::default()
            },
        };

        for _ in 0..40 {
            state.step(input);
        }

        assert_eq!(state.player_one().facing_direction(), FacingDirection::Left);
        assert_eq!(
            state.player_two().facing_direction(),
            FacingDirection::Right
        );
    }

    #[test]
    fn horizontal_input_moves_each_player() {
        let mut state = GameState::new();
        let player_one_start_x = state.player_one().body().x;
        let player_two_start_x = state.player_two().body().x;

        state.step(FrameInput {
            player_one: PlayerInput {
                move_right: true,
                ..PlayerInput::default()
            },
            player_two: PlayerInput {
                move_left: true,
                ..PlayerInput::default()
            },
        });

        assert_eq!(
            state.player_one().body().x,
            player_one_start_x + FIGHTER_HORIZONTAL_SPEED_PER_TICK
        );
        assert_eq!(
            state.player_two().body().x,
            player_two_start_x - FIGHTER_HORIZONTAL_SPEED_PER_TICK
        );
    }

    #[test]
    fn hurtbox_tracks_fighter_body() {
        let mut state = GameState::new();
        let start_hurtbox = state.player_one().hurtbox().rect();

        state.step(FrameInput {
            player_one: PlayerInput {
                move_right: true,
                jump: true,
                ..PlayerInput::default()
            },
            player_two: PlayerInput::default(),
        });

        let hurtbox = state.player_one().hurtbox().rect();

        assert_eq!(
            hurtbox.x,
            start_hurtbox.x + FIGHTER_HORIZONTAL_SPEED_PER_TICK
        );
        assert_eq!(hurtbox.y, start_hurtbox.y + FIGHTER_JUMP_SPEED_PER_TICK);
        assert_eq!(hurtbox.width, start_hurtbox.width);
        assert_eq!(hurtbox.height, start_hurtbox.height);
    }

    #[test]
    fn opposing_horizontal_inputs_cancel_movement() {
        let mut state = GameState::new();
        let player_one_start_x = state.player_one().body().x;

        state.step(FrameInput {
            player_one: PlayerInput {
                move_left: true,
                move_right: true,
                ..PlayerInput::default()
            },
            player_two: PlayerInput::default(),
        });

        assert_eq!(state.player_one().body().x, player_one_start_x);
    }

    #[test]
    fn horizontal_movement_stays_inside_arena() {
        let mut state = GameState::new();
        let input = FrameInput {
            player_one: PlayerInput {
                move_left: true,
                ..PlayerInput::default()
            },
            player_two: PlayerInput {
                move_right: true,
                ..PlayerInput::default()
            },
        };

        for _ in 0..200 {
            state.step(input);
        }

        assert_eq!(state.player_one().body().x, 0);
        assert_eq!(
            state.player_two().body().x + state.player_two().body().width,
            state.arena().width
        );
    }

    #[test]
    fn jump_input_lifts_fighter_off_ground() {
        let mut state = GameState::new();
        let player_one_start_y = state.player_one().body().y;

        state.step(FrameInput {
            player_one: PlayerInput {
                jump: true,
                ..PlayerInput::default()
            },
            player_two: PlayerInput::default(),
        });

        assert!(state.player_one().body().y < player_one_start_y);
    }

    #[test]
    fn jumping_fighter_lands_on_ground() {
        let mut state = GameState::new();
        let player_one_start_y = state.player_one().body().y;

        state.step(FrameInput {
            player_one: PlayerInput {
                jump: true,
                ..PlayerInput::default()
            },
            player_two: PlayerInput::default(),
        });

        for _ in 0..60 {
            state.step(FrameInput::default());
        }

        assert_eq!(state.player_one().body().y, player_one_start_y);
    }

    #[test]
    fn grounded_attack_input_starts_standing_attack() {
        let mut state = GameState::new();

        state.step(FrameInput {
            player_one: PlayerInput {
                attack: true,
                ..PlayerInput::default()
            },
            player_two: PlayerInput::default(),
        });

        assert!(state.player_one().is_performing_standing_attack());
        assert_eq!(
            state.player_one().standing_attack_phase(),
            Some(AttackPhase::Startup)
        );
        assert!(!state.player_one().is_standing_attack_active());
        assert!(!state.player_two().is_performing_standing_attack());
    }

    #[test]
    fn standing_attack_advances_through_startup_active_and_recovery() {
        let mut state = GameState::new();

        state.step(FrameInput {
            player_one: PlayerInput {
                attack: true,
                ..PlayerInput::default()
            },
            player_two: PlayerInput::default(),
        });

        for _ in 1..STANDING_ATTACK_STARTUP_TICKS {
            assert_eq!(
                state.player_one().standing_attack_phase(),
                Some(AttackPhase::Startup)
            );
            assert!(!state.player_one().is_standing_attack_active());

            state.step(FrameInput::default());
        }

        state.step(FrameInput::default());

        for _ in 1..STANDING_ATTACK_ACTIVE_TICKS {
            assert_eq!(
                state.player_one().standing_attack_phase(),
                Some(AttackPhase::Active)
            );
            assert!(state.player_one().is_standing_attack_active());

            state.step(FrameInput::default());
        }

        state.step(FrameInput::default());

        for _ in 1..STANDING_ATTACK_RECOVERY_TICKS {
            assert_eq!(
                state.player_one().standing_attack_phase(),
                Some(AttackPhase::Recovery)
            );
            assert!(!state.player_one().is_standing_attack_active());

            state.step(FrameInput::default());
        }

        state.step(FrameInput::default());

        assert!(!state.player_one().is_performing_standing_attack());
        assert_eq!(state.player_one().standing_attack_phase(), None);
    }

    #[test]
    fn airborne_attack_input_does_not_start_standing_attack() {
        let mut state = GameState::new();

        state.step(FrameInput {
            player_one: PlayerInput {
                jump: true,
                attack: true,
                ..PlayerInput::default()
            },
            player_two: PlayerInput::default(),
        });

        assert!(!state.player_one().is_performing_standing_attack());
        assert_eq!(state.player_one().standing_attack_phase(), None);
    }
}
