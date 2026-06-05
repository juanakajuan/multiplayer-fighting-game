//! Raylib shell for input, rendering, and fixed-timestep driving of the game.

mod game;

use raylib::prelude::*;

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
    draw.draw_rectangle_lines(body.x, body.y, body.width, body.height, outline);

    let marker_x = match fighter.facing_direction() {
        game::FacingDirection::Left => body.x + 12,
        game::FacingDirection::Right => body.x + body.width - 22,
    };
    draw.draw_rectangle(marker_x, body.y + 28, 10, 10, outline);

    if fighter.is_performing_standing_attack() {
        draw_standing_attack(draw, body, fighter.facing_direction(), outline);
    }
}

fn draw_standing_attack(
    draw: &mut RaylibDrawHandle<'_>,
    body: game::Rect,
    facing_direction: game::FacingDirection,
    color: Color,
) {
    const ATTACK_WIDTH: i32 = 54;
    const ATTACK_HEIGHT: i32 = 26;
    const ATTACK_VERTICAL_OFFSET: i32 = 54;

    let attack_x = match facing_direction {
        game::FacingDirection::Left => body.x - ATTACK_WIDTH,
        game::FacingDirection::Right => body.x + body.width,
    };
    let attack_y = body.y + ATTACK_VERTICAL_OFFSET;

    draw.draw_rectangle(attack_x, attack_y, ATTACK_WIDTH, ATTACK_HEIGHT, color);
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
