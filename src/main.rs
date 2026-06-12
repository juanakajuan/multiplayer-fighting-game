//! Raylib shell for input, rendering, and fixed-timestep driving of the game.

use multiplayer_fighting_game::game;
use raylib::prelude::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct HealthBarBounds {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

fn read_player_one_input(rl: &RaylibHandle) -> game::PlayerInput {
    game::PlayerInput {
        move_left: rl.is_key_down(KeyboardKey::KEY_A),
        move_right: rl.is_key_down(KeyboardKey::KEY_D),
        jump: rl.is_key_down(KeyboardKey::KEY_W),
        attack: rl.is_key_down(KeyboardKey::KEY_SPACE),
    }
}

fn read_player_two_input(rl: &RaylibHandle) -> game::PlayerInput {
    game::PlayerInput {
        move_left: rl.is_key_down(KeyboardKey::KEY_LEFT),
        move_right: rl.is_key_down(KeyboardKey::KEY_RIGHT),
        jump: rl.is_key_down(KeyboardKey::KEY_UP),
        attack: rl.is_key_down(KeyboardKey::KEY_SLASH),
    }
}

fn read_frame_input(rl: &RaylibHandle) -> game::FrameInput {
    game::FrameInput {
        player_one: read_player_one_input(rl),
        player_two: read_player_two_input(rl),
    }
}

fn draw_game(draw: &mut RaylibDrawHandle<'_>, state: &game::GameState) {
    draw.clear_background(Color::new(11, 11, 11, 255));
    draw_arena(draw, state.arena());
    draw_health_hud(draw, state);
    draw_fighter(
        draw,
        state.player_one(),
        Color::new(70, 145, 255, 255),
        Color::new(210, 231, 255, 255),
    );
    draw_fighter(
        draw,
        state.player_two(),
        Color::new(238, 92, 92, 255),
        Color::new(255, 224, 224, 255),
    );
}

fn draw_health_hud(draw: &mut RaylibDrawHandle<'_>, state: &game::GameState) {
    const BAR_WIDTH: i32 = 320;
    const BAR_HEIGHT: i32 = 20;
    const BAR_Y: i32 = 28;
    const BAR_MARGIN_X: i32 = 40;

    draw_health_bar(
        draw,
        "P1",
        state.player_one().health(),
        HealthBarBounds {
            x: BAR_MARGIN_X,
            y: BAR_Y,
            width: BAR_WIDTH,
            height: BAR_HEIGHT,
        },
        Color::new(70, 145, 255, 255),
    );
    draw_health_bar(
        draw,
        "P2",
        state.player_two().health(),
        HealthBarBounds {
            x: game::ARENA_WIDTH - BAR_MARGIN_X - BAR_WIDTH,
            y: BAR_Y,
            width: BAR_WIDTH,
            height: BAR_HEIGHT,
        },
        Color::new(238, 92, 92, 255),
    );
}

fn draw_health_bar(
    draw: &mut RaylibDrawHandle<'_>,
    label: &str,
    health: u32,
    bounds: HealthBarBounds,
    fill: Color,
) {
    let clamped_health = health.min(game::FIGHTER_MAX_HEALTH);
    let filled_width = health_bar_fill_width(clamped_health, bounds.width);
    let text = format!("{label} {clamped_health}/{}", game::FIGHTER_MAX_HEALTH);

    draw.draw_rectangle(
        bounds.x,
        bounds.y,
        bounds.width,
        bounds.height,
        Color::new(25, 26, 30, 255),
    );
    draw.draw_rectangle(bounds.x, bounds.y, filled_width, bounds.height, fill);
    draw.draw_rectangle_lines(
        bounds.x,
        bounds.y,
        bounds.width,
        bounds.height,
        Color::new(228, 231, 236, 255),
    );
    draw.draw_text(
        &text,
        bounds.x,
        bounds.y + bounds.height + 6,
        20,
        Color::new(228, 231, 236, 255),
    );
}

fn health_bar_fill_width(health: u32, width: i32) -> i32 {
    let filled_width = (i64::from(width) * i64::from(health)) / i64::from(game::FIGHTER_MAX_HEALTH);

    i32::try_from(filled_width).expect("health bar width fits i32")
}

fn draw_arena(draw: &mut RaylibDrawHandle<'_>, arena: game::Arena) {
    draw.draw_rectangle(
        0,
        arena.ground_y,
        arena.width,
        arena.height - arena.ground_y,
        Color::new(36, 38, 44, 255),
    );
    draw.draw_line(
        0,
        arena.ground_y,
        arena.width,
        arena.ground_y,
        Color::new(228, 231, 236, 255),
    );
    draw.draw_rectangle_lines(
        0,
        0,
        arena.width,
        arena.ground_y,
        Color::new(76, 82, 92, 255),
    );
}

fn draw_fighter(
    draw: &mut RaylibDrawHandle<'_>,
    fighter: game::FighterState,
    fill: Color,
    outline: Color,
) {
    let body = fighter.body();
    draw.draw_rectangle(body.x, body.y, body.width, body.height, fill);

    let hurtbox = fighter.hurtbox().rect();
    draw.draw_rectangle_lines(hurtbox.x, hurtbox.y, hurtbox.width, hurtbox.height, outline);

    let marker_x = match fighter.facing_direction() {
        game::FacingDirection::Left => body.x + 12,
        game::FacingDirection::Right => body.x + body.width - 22,
    };
    draw.draw_rectangle(marker_x, body.y + 28, 10, 10, outline);

    if let Some(hitbox) = fighter.attack_hitbox() {
        draw_attack_hitbox(draw, hitbox, outline);
    }
}

fn draw_attack_hitbox(draw: &mut RaylibDrawHandle<'_>, hitbox: game::AttackHitbox, color: Color) {
    let rect = hitbox.rect();
    draw.draw_rectangle(rect.x, rect.y, rect.width, rect.height, color);
}

fn main() {
    // Prevent long stalls from causing an unbounded simulation catch-up spiral.
    const MAX_CATCH_UP_TICKS: f64 = 5.0;

    let (mut rl, thread) = raylib::init()
        .size(game::ARENA_WIDTH, game::ARENA_HEIGHT)
        .title("Multiplayer Fighting Game")
        .build();

    rl.set_target_fps(game::TICKS_PER_SECOND);

    let mut game_state = game::GameState::new();
    let mut previous_time_seconds = rl.get_time();
    let mut accumulated_time_seconds = 0.0;

    while !rl.window_should_close() {
        let current_time_seconds = rl.get_time();
        let elapsed_time_seconds = (current_time_seconds - previous_time_seconds)
            .min(game::SECONDS_PER_TICK * MAX_CATCH_UP_TICKS);

        previous_time_seconds = current_time_seconds;
        accumulated_time_seconds += elapsed_time_seconds;

        while accumulated_time_seconds >= game::SECONDS_PER_TICK {
            game_state.step(read_frame_input(&rl));
            accumulated_time_seconds -= game::SECONDS_PER_TICK;
        }

        let mut draw = rl.begin_drawing(&thread);
        draw_game(&mut draw, &game_state);
    }
}
