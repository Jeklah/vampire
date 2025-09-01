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
        fullscreen: true,
        sample_count: 4,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // Initialize random seed
    rand::srand(macroquad::miniquad::date::now() as u64);

    // Removed "Initializing..." screen for faster startup

    // Create game state and systems
    let mut game_state = GameState::new();
    let mut input_handler = InputHandler::new();

    // Track fullscreen state (starts as true, using macroquad's native fullscreen)
    let mut is_fullscreen = true;

    // Embed font data directly in binary for reliable loading
    let font_data: &[u8] = include_bytes!("../assets/fonts/default.ttf");
    let font = match load_ttf_font_from_bytes(font_data) {
        Ok(font) => {
            game_state.add_debug_message("Font loaded successfully from embedded data".to_string());
            Some(font)
        }
        Err(e) => {
            game_state.add_debug_message(format!("Warning: Could not load embedded font: {}", e));
            game_state.add_debug_message("Using default system font".to_string());
            None
        }
    };

    let mut renderer = Renderer::new(font);

    // Add debug message about fullscreen mode
    game_state
        .add_debug_message("Game started in fullscreen mode (F11 to toggle windowed)".to_string());

    // Add debug message about spawning system controls
    game_state.add_debug_message(
        "Debug keys: 1=Spawn Hostiles, 2=Spawn Animals, 3=Spawn Clan, C=Cleanup, P=Perf Mode"
            .to_string(),
    );

    let mut last_time = get_time();
    let mut frame_count = 0;
    let mut fps_timer = 0.0;
    let mut cleanup_frame_counter = 0;
    let mut adaptive_cleanup_interval = 180; // Default: 3 seconds at 60fps

    // Main game loop
    loop {
        let frame_start = get_time();
        cleanup_frame_counter += 1;

        // Calculate delta time
        let current_time = get_time();
        let delta_time = (current_time - last_time) as f32;
        last_time = current_time;

        // Cap delta time to prevent large jumps (allow for frame drops/pauses)
        let delta_time = delta_time.min(0.1); // Max 100ms to handle pauses gracefully

        // Adaptive cleanup based on macroquad's FPS for optimal performance
        let current_fps = get_fps();

        // Adjust cleanup frequency based on FPS
        adaptive_cleanup_interval = if current_fps < 30 {
            90 // Cleanup every 1.5 seconds if FPS is low
        } else if current_fps < 45 {
            120 // Cleanup every 2 seconds if FPS is moderate
        } else {
            180 // Cleanup every 3 seconds if FPS is good
        };

        // Use macroquad's frame timing for efficient cleanup scheduling
        if cleanup_frame_counter % adaptive_cleanup_interval == 0 {
            game_state.force_cleanup();
            game_state.add_debug_message(format!(
                "Adaptive cleanup (interval: {}f @ {:.0}fps)",
                adaptive_cleanup_interval, current_fps
            ));
        }

        // Update FPS counter using macroquad's built-in FPS tracking
        frame_count += 1;
        fps_timer += delta_time;
        if fps_timer >= 1.0 {
            let fps = get_fps() as f32; // Use macroquad's FPS counter
            let perf_mode = if renderer.performance_mode() {
                "PERF"
            } else {
                "NORM"
            };

            // Get player speed for monitoring and entity count
            let player_speed = game_state
                .entities
                .iter()
                .find(|e| {
                    matches!(
                        e.entity_type,
                        vampire_rpg::components::game_data::EntityType::Player
                    )
                })
                .and_then(|p| p.velocity.as_ref())
                .map(|v| (v.x.powi(2) + v.y.powi(2)).sqrt())
                .unwrap_or(0.0);

            let entity_count = game_state.entities.len();
            let max_entities = game_state.max_entities;

            game_state.add_debug_message(format!(
                "FPS: {:.0} | DT: {:.4}s | {} | Speed: {:.0} | Entities: {}/{} | Cleanup: {}f",
                fps,
                delta_time,
                perf_mode,
                player_speed,
                entity_count,
                max_entities,
                adaptive_cleanup_interval
            ));

            // Auto-enable performance mode if FPS drops below 30
            if fps < 30.0 && !renderer.performance_mode() {
                renderer.set_performance_mode(true);
                game_state.set_performance_mode(true);
                game_state
                    .add_debug_message("Auto-enabled performance mode due to low FPS".to_string());
            }

            // Force cleanup if FPS is critically low using macroquad timing
            if fps < 20.0 {
                game_state.force_cleanup();
                cleanup_frame_counter = 0; // Reset counter after emergency cleanup
                game_state.add_debug_message("Emergency cleanup triggered".to_string());
            }

            frame_count = 0;
            fps_timer = 0.0;
        }

        // Handle input
        input_handler.update();

        // Handle fullscreen toggle with F11
        if is_key_pressed(KeyCode::F11) {
            is_fullscreen = !is_fullscreen;
            set_fullscreen(is_fullscreen);

            if is_fullscreen {
                game_state.add_debug_message("Switched to fullscreen mode".to_string());
            } else {
                game_state.add_debug_message("Switched to windowed mode".to_string());
            }
        }

        // Handle performance mode toggle with P key
        if is_key_pressed(KeyCode::P) {
            let current_mode = renderer.performance_mode();
            renderer.set_performance_mode(!current_mode);
            game_state.set_performance_mode(!current_mode);
            if !current_mode {
                game_state.add_debug_message("Performance mode enabled".to_string());
            } else {
                game_state.add_debug_message("Performance mode disabled".to_string());
            }
        }

        // Handle manual cleanup with C key
        if is_key_pressed(KeyCode::C) {
            game_state.force_cleanup();
            game_state.add_debug_message("Manual cleanup executed".to_string());
        }

        // Debug spawning keys (for testing the spawning system)
        if is_key_pressed(KeyCode::Key1) {
            game_state.debug_spawn_entities("hostile", 3);
        }
        if is_key_pressed(KeyCode::Key2) {
            game_state.debug_spawn_entities("animal", 3);
        }
        if is_key_pressed(KeyCode::Key3) {
            game_state.debug_spawn_entities("clan_member", 1);
        }

        // Handle window close
        if is_key_pressed(KeyCode::Q) && is_key_down(KeyCode::LeftControl) {
            break;
        }

        // Update game state
        game_state.update(&input_handler, delta_time);

        // Render the game (removed problematic resolution scaling for cross-platform compatibility)
        renderer.render(&mut game_state);

        // Let macroquad handle frame rate limiting via VSync with next_frame()
        // Remove manual frame limiting to allow 60+ FPS

        // Present frame
        next_frame().await;
    }
}
