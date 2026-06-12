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
const STANDING_ATTACK_HITBOX_WIDTH: i32 = 54;
const STANDING_ATTACK_HITBOX_HEIGHT: i32 = 26;
const STANDING_ATTACK_HITBOX_VERTICAL_OFFSET: i32 = 54;

/// Starting health for each fighter.
pub const FIGHTER_MAX_HEALTH: u32 = 100;

/// Damage dealt by the current standing melee attack.
pub const STANDING_ATTACK_DAMAGE: u32 = 10;

/// Hitstun applied by the current standing melee attack.
pub const STANDING_ATTACK_HITSTUN_TICKS: u32 = 18;

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

/// Offensive area used for attack collision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AttackHitbox {
    rect: Rect,
}

/// Current hitbox versus hurtbox overlaps for both fighters.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HitboxCollisions {
    /// Player one's active attack hitbox overlaps player two's hurtbox.
    pub player_one_hits_player_two: bool,
    /// Player two's active attack hitbox overlaps player one's hurtbox.
    pub player_two_hits_player_one: bool,
}

/// Per-fighter gameplay state owned by [`GameState`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FighterState {
    body: Rect,
    facing_direction: FacingDirection,
    vertical_velocity_per_tick: i32,
    health: u32,
    hitstun_ticks_remaining: u32,
    standing_attack: Option<StandingAttackState>,
}

impl Hurtbox {
    /// Rectangle occupied by this hurtbox in screen-space pixels.
    #[must_use]
    pub const fn rect(&self) -> Rect {
        self.rect
    }
}

impl AttackHitbox {
    /// Rectangle occupied by this hitbox in screen-space pixels.
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
    /// Frames where the strike owns an attack hitbox.
    Active,
    /// Post-hit frames before the fighter returns to neutral.
    Recovery,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct StandingAttackState {
    phase: AttackPhase,
    ticks_in_phase: u32,
    has_hit: bool,
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
            health: FIGHTER_MAX_HEALTH,
            hitstun_ticks_remaining: 0,
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

    /// Current remaining health.
    #[must_use]
    pub const fn health(&self) -> u32 {
        self.health
    }

    /// Whether the fighter is currently unable to act because they were hit.
    #[must_use]
    pub const fn is_in_hitstun(&self) -> bool {
        self.hitstun_ticks_remaining > 0
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

    /// Current offensive hitbox, if the standing attack is active this tick.
    #[must_use]
    pub const fn attack_hitbox(&self) -> Option<AttackHitbox> {
        if !self.is_standing_attack_active() {
            return None;
        }

        let x = match self.facing_direction {
            FacingDirection::Left => self.body.x - STANDING_ATTACK_HITBOX_WIDTH,
            FacingDirection::Right => self.body.x + self.body.width,
        };

        Some(AttackHitbox {
            rect: Rect {
                x,
                y: self.body.y + STANDING_ATTACK_HITBOX_VERTICAL_OFFSET,
                width: STANDING_ATTACK_HITBOX_WIDTH,
                height: STANDING_ATTACK_HITBOX_HEIGHT,
            },
        })
    }

    fn step(&mut self, input: PlayerInput, arena: Arena) {
        if self.is_in_hitstun() {
            self.advance_hitstun();
            self.move_vertically(false, arena);
            return;
        }

        self.move_horizontally(horizontal_direction(input), arena);
        self.move_vertically(input.jump, arena);
        self.update_standing_attack(input.attack, arena);
    }

    fn advance_hitstun(&mut self) {
        self.hitstun_ticks_remaining = self.hitstun_ticks_remaining.saturating_sub(1);
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

    fn can_apply_standing_attack_damage(&self) -> bool {
        matches!(
            self.standing_attack,
            Some(StandingAttackState {
                phase: AttackPhase::Active,
                has_hit: false,
                ..
            })
        )
    }

    fn mark_standing_attack_hit(&mut self) {
        if let Some(standing_attack) = self.standing_attack.as_mut() {
            standing_attack.has_hit = true;
        }
    }

    fn apply_damage(&mut self, damage: u32) {
        self.health = self.health.saturating_sub(damage);
    }

    fn apply_standing_attack_hit(&mut self) {
        self.apply_damage(STANDING_ATTACK_DAMAGE);
        self.hitstun_ticks_remaining = STANDING_ATTACK_HITSTUN_TICKS;
        self.standing_attack = None;
    }
}

impl StandingAttackState {
    const fn new() -> Self {
        Self {
            phase: AttackPhase::Startup,
            ticks_in_phase: 1,
            has_hit: false,
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
            has_hit: self.has_hit,
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

    /// Current hitbox versus hurtbox overlaps.
    #[must_use]
    pub fn hitbox_collisions(&self) -> HitboxCollisions {
        HitboxCollisions {
            player_one_hits_player_two: attack_overlaps_hurtbox(
                self.player_one.attack_hitbox(),
                self.player_two.hurtbox(),
            ),
            player_two_hits_player_one: attack_overlaps_hurtbox(
                self.player_two.attack_hitbox(),
                self.player_one.hurtbox(),
            ),
        }
    }

    /// Advances gameplay by exactly one fixed tick.
    pub fn step(&mut self, input: FrameInput) {
        self.player_one.step(input.player_one, self.arena);
        self.player_two.step(input.player_two, self.arena);
        self.update_facing_directions();
        self.apply_hit_damage();

        if self.has_defeated_fighter() {
            self.reset_match();
            return;
        }

        self.tick = self.tick.checked_add(1).expect("tick counter overflow");
    }

    fn apply_hit_damage(&mut self) {
        let collisions = self.hitbox_collisions();
        let player_one_hits_player_two = collisions.player_one_hits_player_two
            && self.player_one.can_apply_standing_attack_damage();
        let player_two_hits_player_one = collisions.player_two_hits_player_one
            && self.player_two.can_apply_standing_attack_damage();

        if player_one_hits_player_two {
            self.player_two.apply_standing_attack_hit();
            self.player_one.mark_standing_attack_hit();
        }

        if player_two_hits_player_one {
            self.player_one.apply_standing_attack_hit();
            self.player_two.mark_standing_attack_hit();
        }
    }

    fn has_defeated_fighter(&self) -> bool {
        self.player_one.health == 0 || self.player_two.health == 0
    }

    fn reset_match(&mut self) {
        *self = Self::default();
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

fn attack_overlaps_hurtbox(attack_hitbox: Option<AttackHitbox>, hurtbox: Hurtbox) -> bool {
    match attack_hitbox {
        Some(attack_hitbox) => rects_overlap(attack_hitbox.rect(), hurtbox.rect()),
        None => false,
    }
}

fn rects_overlap(first: Rect, second: Rect) -> bool {
    first.x < second.x + second.width
        && first.x + first.width > second.x
        && first.y < second.y + second.height
        && first.y + first.height > second.y
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
        FIGHTER_MAX_HEALTH, FacingDirection, FrameInput, GameState, HitboxCollisions, PlayerInput,
        STANDING_ATTACK_ACTIVE_TICKS, STANDING_ATTACK_DAMAGE, STANDING_ATTACK_HITBOX_HEIGHT,
        STANDING_ATTACK_HITBOX_VERTICAL_OFFSET, STANDING_ATTACK_HITBOX_WIDTH,
        STANDING_ATTACK_HITSTUN_TICKS, STANDING_ATTACK_RECOVERY_TICKS,
        STANDING_ATTACK_STARTUP_TICKS,
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

        assert_eq!(
            state.player_one().standing_attack_phase(),
            Some(AttackPhase::Startup)
        );
        assert!(!state.player_one().is_standing_attack_active());
        assert_eq!(state.player_two().standing_attack_phase(), None);
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

        assert_eq!(state.player_one().standing_attack_phase(), None);
    }

    #[test]
    fn standing_attack_has_hitbox_only_during_active_frames() {
        let mut state = GameState::new();

        state.step(FrameInput {
            player_one: PlayerInput {
                attack: true,
                ..PlayerInput::default()
            },
            player_two: PlayerInput {
                attack: true,
                ..PlayerInput::default()
            },
        });

        assert_eq!(
            state.player_one().standing_attack_phase(),
            Some(AttackPhase::Startup)
        );
        assert_eq!(state.player_one().attack_hitbox(), None);

        for _ in 1..STANDING_ATTACK_STARTUP_TICKS {
            state.step(FrameInput::default());
        }

        state.step(FrameInput::default());

        let player_one_body = state.player_one().body();
        let player_two_body = state.player_two().body();
        let player_one_hitbox = state
            .player_one()
            .attack_hitbox()
            .expect("player one active attack should have a hitbox")
            .rect();
        let player_two_hitbox = state
            .player_two()
            .attack_hitbox()
            .expect("player two active attack should have a hitbox")
            .rect();

        assert_eq!(
            player_one_hitbox.x,
            player_one_body.x + player_one_body.width
        );
        assert_eq!(
            player_one_hitbox.y,
            player_one_body.y + STANDING_ATTACK_HITBOX_VERTICAL_OFFSET
        );
        assert_eq!(player_one_hitbox.width, STANDING_ATTACK_HITBOX_WIDTH);
        assert_eq!(player_one_hitbox.height, STANDING_ATTACK_HITBOX_HEIGHT);

        assert_eq!(
            player_two_hitbox.x,
            player_two_body.x - STANDING_ATTACK_HITBOX_WIDTH
        );
        assert_eq!(
            player_two_hitbox.y,
            player_two_body.y + STANDING_ATTACK_HITBOX_VERTICAL_OFFSET
        );
        assert_eq!(player_two_hitbox.width, STANDING_ATTACK_HITBOX_WIDTH);
        assert_eq!(player_two_hitbox.height, STANDING_ATTACK_HITBOX_HEIGHT);

        for _ in 1..STANDING_ATTACK_ACTIVE_TICKS {
            state.step(FrameInput::default());
        }

        state.step(FrameInput::default());

        assert_eq!(
            state.player_one().standing_attack_phase(),
            Some(AttackPhase::Recovery)
        );
        assert_eq!(state.player_one().attack_hitbox(), None);
    }

    #[test]
    fn active_attack_detects_hitbox_hurtbox_collision() {
        let mut state = GameState::new();
        let player_one_body = state.player_one().body();
        state.player_two.body.x =
            player_one_body.x + player_one_body.width + STANDING_ATTACK_HITBOX_WIDTH - 1;
        state.player_two.body.y = player_one_body.y;

        state.step(FrameInput {
            player_one: PlayerInput {
                attack: true,
                ..PlayerInput::default()
            },
            player_two: PlayerInput::default(),
        });

        assert_eq!(state.hitbox_collisions(), HitboxCollisions::default());

        for _ in 0..STANDING_ATTACK_STARTUP_TICKS {
            state.step(FrameInput::default());
        }

        assert_eq!(
            state.player_one().standing_attack_phase(),
            Some(AttackPhase::Active)
        );
        assert_eq!(
            state.hitbox_collisions(),
            HitboxCollisions {
                player_one_hits_player_two: true,
                player_two_hits_player_one: false,
            }
        );
    }

    #[test]
    fn active_attack_applies_damage_once_on_hit() {
        let mut state = GameState::new();
        let player_one_body = state.player_one().body();
        state.player_two.body.x =
            player_one_body.x + player_one_body.width + STANDING_ATTACK_HITBOX_WIDTH - 1;
        state.player_two.body.y = player_one_body.y;

        state.step(FrameInput {
            player_one: PlayerInput {
                attack: true,
                ..PlayerInput::default()
            },
            player_two: PlayerInput::default(),
        });

        assert_eq!(state.player_two().health(), FIGHTER_MAX_HEALTH);

        for _ in 0..STANDING_ATTACK_STARTUP_TICKS {
            state.step(FrameInput::default());
        }

        assert_eq!(
            state.player_two().health(),
            FIGHTER_MAX_HEALTH - STANDING_ATTACK_DAMAGE
        );

        for _ in 0..STANDING_ATTACK_ACTIVE_TICKS {
            state.step(FrameInput::default());
        }

        assert_eq!(
            state.player_two().health(),
            FIGHTER_MAX_HEALTH - STANDING_ATTACK_DAMAGE
        );
        assert_eq!(state.player_one().health(), FIGHTER_MAX_HEALTH);
    }

    #[test]
    fn active_attack_applies_hitstun_on_hit() {
        let mut state = GameState::new();
        let player_one_body = state.player_one().body();
        state.player_two.body.x =
            player_one_body.x + player_one_body.width + STANDING_ATTACK_HITBOX_WIDTH - 1;
        state.player_two.body.y = player_one_body.y;

        state.step(FrameInput {
            player_one: PlayerInput {
                attack: true,
                ..PlayerInput::default()
            },
            player_two: PlayerInput::default(),
        });

        for _ in 0..STANDING_ATTACK_STARTUP_TICKS {
            state.step(FrameInput::default());
        }

        assert!(state.player_two().is_in_hitstun());
        assert_eq!(
            state.player_two.hitstun_ticks_remaining,
            STANDING_ATTACK_HITSTUN_TICKS
        );

        let player_two_hit_x = state.player_two().body().x;
        state.step(FrameInput {
            player_one: PlayerInput::default(),
            player_two: PlayerInput {
                move_left: true,
                jump: true,
                attack: true,
                ..PlayerInput::default()
            },
        });

        assert_eq!(state.player_two().body().x, player_two_hit_x);
        assert_eq!(state.player_two().standing_attack_phase(), None);
        assert_eq!(
            state.player_two.hitstun_ticks_remaining,
            STANDING_ATTACK_HITSTUN_TICKS - 1
        );

        for _ in 1..STANDING_ATTACK_HITSTUN_TICKS {
            state.step(FrameInput::default());
        }

        assert!(!state.player_two().is_in_hitstun());

        state.step(FrameInput {
            player_one: PlayerInput::default(),
            player_two: PlayerInput {
                move_left: true,
                ..PlayerInput::default()
            },
        });

        assert_eq!(
            state.player_two().body().x,
            player_two_hit_x - FIGHTER_HORIZONTAL_SPEED_PER_TICK
        );
    }

    #[test]
    fn lethal_attack_resets_match() {
        let mut state = GameState::new();
        let player_one_body = state.player_one().body();
        state.player_two.body.x =
            player_one_body.x + player_one_body.width + STANDING_ATTACK_HITBOX_WIDTH - 1;
        state.player_two.body.y = player_one_body.y;
        state.player_two.health = STANDING_ATTACK_DAMAGE;

        state.step(FrameInput {
            player_one: PlayerInput {
                attack: true,
                ..PlayerInput::default()
            },
            player_two: PlayerInput::default(),
        });

        for _ in 0..STANDING_ATTACK_STARTUP_TICKS {
            state.step(FrameInput::default());
        }

        assert_eq!(state, GameState::new());
    }

    #[test]
    fn active_attack_misses_when_hurtbox_is_out_of_range() {
        let mut state = GameState::new();

        state.step(FrameInput {
            player_one: PlayerInput {
                attack: true,
                ..PlayerInput::default()
            },
            player_two: PlayerInput::default(),
        });

        for _ in 0..STANDING_ATTACK_STARTUP_TICKS {
            state.step(FrameInput::default());
        }

        assert_eq!(
            state.player_one().standing_attack_phase(),
            Some(AttackPhase::Active)
        );
        assert_eq!(state.hitbox_collisions(), HitboxCollisions::default());
        assert_eq!(state.player_two().health(), FIGHTER_MAX_HEALTH);
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

        assert_eq!(state.player_one().standing_attack_phase(), None);
    }
}
