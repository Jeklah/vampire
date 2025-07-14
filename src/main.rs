//! Vampire RPG - Main entry point
//!
//! A 2D vampire survival RPG built with Rust and Macroquad.

use macroquad::prelude::*;

use vampire_rpg::{GameState, InputHandler, Renderer};

/// Window configuration for the game
fn window_conf() -> Conf {
    Conf {
        window_title: "Vampire RPG: The First Immortal".to_owned(),
        window_width: 1280,
        window_height: 720,
        window_resizable: false,
        fullscreen: false,
        sample_count: 4,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // Initialize random seed
    rand::srand(macroquad::miniquad::date::now() as u64);

    // Create game state and systems
    let mut game_state = GameState::new();
    let mut input_handler = InputHandler::new();
    let renderer = Renderer::new();

    let mut last_time = get_time();

    // Main game loop
    loop {
        // Calculate delta time
        let current_time = get_time();
        let delta_time = (current_time - last_time) as f32;
        last_time = current_time;

        // Cap delta time to prevent large jumps
        let delta_time = delta_time.min(1.0 / 30.0);

        // Handle input
        input_handler.update();

        // Handle window close
        if is_key_pressed(KeyCode::Q) && is_key_down(KeyCode::LeftControl) {
            break;
        }

        // Update game state
        game_state.update(&input_handler, delta_time);

        // Render the game
        renderer.render(&game_state);

        // Present frame
        next_frame().await;
    }
}
