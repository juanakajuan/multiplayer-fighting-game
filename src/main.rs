mod game;

use raylib::prelude::*;

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
            game_state.step(game::FrameInput::default());
            accumulated_time_seconds -= game::SECONDS_PER_TICK;
        }

        let mut draw = rl.begin_drawing(&thread);
        draw.clear_background(raylib::color::Color::BLACK);
    }
}
