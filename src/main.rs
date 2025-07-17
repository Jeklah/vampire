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
        high_dpi: true,
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

    // Track fullscreen state (starts as true based on window_conf)
    let mut is_fullscreen = true;

    // Performance settings for fullscreen
    let base_width = 1280.0;
    let base_height = 720.0;
    let mut render_scale = if is_fullscreen {
        // Scale down rendering for better performance in fullscreen
        let screen_w = screen_width();
        let screen_h = screen_height();
        let scale_x = base_width / screen_w;
        let scale_y = base_height / screen_h;
        scale_x.max(scale_y).min(1.0) // Never scale up, only down for performance
    } else {
        1.0
    };

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
    game_state.add_debug_message("Game started in fullscreen mode".to_string());

    let mut last_time = get_time();
    let mut frame_count = 0;
    let mut fps_timer = 0.0;

    // Main game loop
    loop {
        let frame_start = get_time();

        // Calculate delta time
        let current_time = get_time();
        let delta_time = (current_time - last_time) as f32;
        last_time = current_time;

        // Cap delta time to prevent large jumps (allow for frame drops/pauses)
        let delta_time = delta_time.min(0.1); // Max 100ms to handle pauses gracefully

        // Update FPS counter and delta time monitoring
        frame_count += 1;
        fps_timer += delta_time;
        if fps_timer >= 1.0 {
            let fps = frame_count as f32 / fps_timer;
            let perf_mode = if renderer.performance_mode() {
                "PERF"
            } else {
                "NORM"
            };

            // Get player speed for monitoring
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

            game_state.add_debug_message(format!(
                "FPS: {:.1} | DT: {:.4}s | {} | Speed: {:.0}",
                fps, delta_time, perf_mode, player_speed
            ));
            frame_count = 0;
            fps_timer = 0.0;
        }

        // Handle input
        input_handler.update();

        // Handle fullscreen toggle with F11
        if is_key_pressed(KeyCode::F11) {
            is_fullscreen = !is_fullscreen;
            set_fullscreen(is_fullscreen);

            // Update render scale based on mode
            render_scale = if is_fullscreen {
                let screen_w = screen_width();
                let screen_h = screen_height();
                let scale_x = base_width / screen_w;
                let scale_y = base_height / screen_h;
                scale_x.max(scale_y).min(1.0)
            } else {
                1.0
            };

            if is_fullscreen {
                game_state.add_debug_message(format!(
                    "Switched to fullscreen mode (render scale: {:.2})",
                    render_scale
                ));
            } else {
                game_state.add_debug_message("Switched to windowed mode".to_string());
            }
        }

        // Handle performance mode toggle with P key
        if is_key_pressed(KeyCode::P) {
            let current_mode = renderer.performance_mode();
            renderer.set_performance_mode(!current_mode);
            if !current_mode {
                game_state.add_debug_message("Performance mode enabled".to_string());
            } else {
                game_state.add_debug_message("Performance mode disabled".to_string());
            }
        }

        // Handle window close
        if is_key_pressed(KeyCode::Q) && is_key_down(KeyCode::LeftControl) {
            break;
        }

        // Update game state
        game_state.update(&input_handler, delta_time);

        // Render the game with performance scaling
        if render_scale < 1.0 {
            // Render to a smaller internal resolution for better performance
            let render_width = (screen_width() * render_scale) as i32;
            let render_height = (screen_height() * render_scale) as i32;

            // Set render target size
            set_camera(&Camera2D {
                target: vec2(0.0, 0.0),
                zoom: vec2(2.0 / render_width as f32, -2.0 / render_height as f32),
                ..Default::default()
            });
        }

        renderer.render(&game_state);

        if render_scale < 1.0 {
            // Reset camera
            set_default_camera();
        }

        // Let macroquad handle frame rate limiting via VSync with next_frame()
        // Remove manual frame limiting to allow 60+ FPS

        // Present frame
        next_frame().await;
    }
}
