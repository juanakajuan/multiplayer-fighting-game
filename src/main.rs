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

fn main() {
    const SCREEN_WIDTH: i32 = 1280;
    const SCREEN_HEIGHT: i32 = 720;
    const MAX_CATCH_UP_TICKS: f64 = 5.0;

    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
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
        draw.clear_background(raylib::color::Color::BLACK);
    }
}
