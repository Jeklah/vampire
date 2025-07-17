//! Rendering Module
//!
//! This module handles all rendering and drawing operations for the Vampire RPG.

use crate::components::*;
use crate::game_state::GameState;
use crate::systems::ShelterSystem;
use macroquad::prelude::*;

pub struct Renderer {
    zoom_level: f32,
    font: Option<Font>,
    performance_mode: bool,
    last_entity_count: usize,
    last_tile_count: usize,
    // Ground rendering cache
    last_camera_x: f32,
    last_camera_y: f32,
    camera_moved_significantly: bool,
    frame_skip_counter: u32,
}

impl Renderer {
    pub fn new(font: Option<Font>) -> Self {
        Self {
            zoom_level: 1.5,
            font,
            performance_mode: false,
            last_entity_count: 0,
            last_tile_count: 0,
            last_camera_x: 0.0,
            last_camera_y: 0.0,
            camera_moved_significantly: true,
            frame_skip_counter: 0,
        }
    }

    pub fn set_performance_mode(&mut self, enabled: bool) {
        self.performance_mode = enabled;
    }

    pub fn update_performance_scaling(&mut self, player_velocity: Option<&Velocity>) {
        // Automatically adjust performance based on movement speed (less aggressive)
        if let Some(velocity) = player_velocity {
            let speed = (velocity.x.powi(2) + velocity.y.powi(2)).sqrt();
            if speed > 300.0 {
                // Moving very fast - enable performance optimizations
                self.performance_mode = true;
            } else if speed < 25.0 {
                // Moving very slowly or stationary - can afford higher quality
                self.performance_mode = false;
            }
            // Keep current mode if in between to avoid flickering
        }
    }

    pub fn performance_mode(&self) -> bool {
        self.performance_mode
    }

    fn draw_text_with_font(&self, text: &str, x: f32, y: f32, font_size: f32, color: Color) {
        match &self.font {
            Some(font) => {
                let params = TextParams {
                    font: Some(font),
                    font_size: font_size as u16,
                    color,
                    ..Default::default()
                };
                draw_text_ex(text, x, y, params);
            }
            None => {
                draw_text(text, x, y, font_size, color);
            }
        }
    }

    pub fn render(&mut self, game_state: &GameState) {
        // Auto-adjust performance based on player movement speed
        if let Some(player) = game_state
            .entities
            .iter()
            .find(|e| matches!(e.entity_type, EntityType::Player))
        {
            self.update_performance_scaling(player.velocity.as_ref());
        }

        clear_background(Color::new(0.05, 0.05, 0.15, 1.0)); // Dark blue night sky

        // Calculate camera offset with zoom
        let camera_offset_x = screen_width() / 2.0 - game_state.camera_x * self.zoom_level;
        let camera_offset_y = screen_height() / 2.0 - game_state.camera_y * self.zoom_level;

        // Update camera tracking for performance decisions
        let camera_delta_x = (game_state.camera_x - self.last_camera_x).abs();
        let camera_delta_y = (game_state.camera_y - self.last_camera_y).abs();
        let movement_threshold = 10.0; // Smaller threshold for smoother updates

        self.camera_moved_significantly =
            camera_delta_x > movement_threshold || camera_delta_y > movement_threshold;

        if self.camera_moved_significantly {
            self.last_camera_x = game_state.camera_x;
            self.last_camera_y = game_state.camera_y;
        }

        // Draw ground with smart caching
        self.draw_ground_cached(game_state, camera_offset_x, camera_offset_y);

        // Draw stars and moon (always draw but less detail in performance mode)
        self.draw_stars(game_state, camera_offset_x, camera_offset_y);
        self.draw_moon(game_state, camera_offset_x, camera_offset_y);

        // Draw blood particles (reduce count only in extreme performance mode)
        for (i, particle) in game_state.blood_particles.iter().enumerate() {
            if !self.performance_mode || i % 3 != 0 {
                particle.draw(camera_offset_x, camera_offset_y);
            }
        }

        // Draw shelters first (behind entities)
        ShelterSystem::render_shelters(
            &game_state.entities,
            camera_offset_x,
            camera_offset_y,
            self.zoom_level,
            false, // Show debug info - could be made configurable
        );

        // Draw all entities
        self.draw_entities(game_state, camera_offset_x, camera_offset_y);

        // Draw UI
        self.draw_ui(game_state);

        // Draw debug messages
        self.draw_debug_messages(game_state);

        // Draw menus
        if game_state.paused {
            self.draw_pause_menu();
        }

        if game_state.show_clan_menu {
            self.draw_clan_menu(game_state);
        }

        if game_state.show_legend {
            self.draw_legend(game_state);
        }

        if game_state.show_quick_start {
            self.draw_quick_start_guide();
        }
    }

    fn draw_entities(&self, game_state: &GameState, camera_offset_x: f32, camera_offset_y: f32) {
        // Pre-calculate screen bounds for better culling
        let screen_w = screen_width();
        let screen_h = screen_height();
        let cull_margin = if self.performance_mode { 30.0 } else { 50.0 };

        // Calculate camera movement for LOD decisions
        let camera_speed = ((game_state.camera_x - self.last_camera_x).powi(2)
            + (game_state.camera_y - self.last_camera_y).powi(2))
        .sqrt();
        let skip_details = self.performance_mode || camera_speed > 100.0;

        // Batch entities by type for potential future optimizations
        let mut visible_entities = Vec::with_capacity(game_state.entities.len());

        // First pass: cull and collect visible entities
        for entity in &game_state.entities {
            let screen_x = entity.position.x * self.zoom_level + camera_offset_x;
            let screen_y = entity.position.y * self.zoom_level + camera_offset_y;

            // Improved culling with tighter bounds
            if screen_x > -cull_margin
                && screen_x < screen_w + cull_margin
                && screen_y > -cull_margin
                && screen_y < screen_h + cull_margin
            {
                // Skip dead entities early
                if let Some(health) = &entity.health {
                    if health.current <= 0.0 || matches!(entity.ai_state, AIState::Dead) {
                        continue;
                    }
                }

                // Skip shelter entities (rendered separately)
                if matches!(entity.entity_type, EntityType::Shelter) {
                    continue;
                }

                visible_entities.push((entity, screen_x, screen_y));
            }
        }

        // Second pass: render visible entities
        for (entity, screen_x, screen_y) in visible_entities {
            let size = match entity.entity_type {
                EntityType::Player => 30.0,
                EntityType::ClanLeader(_) => 28.0,
                EntityType::ClanMember(_) => 24.0,
                EntityType::HostileInfected => 20.0,
                EntityType::Animal => 16.0,
                EntityType::Shelter => continue, // Already filtered out
            };

            // Draw entity sprite
            match entity.entity_type {
                EntityType::Player => {
                    let facing_direction = entity
                        .velocity
                        .as_ref()
                        .map(|v| v.x.atan2(v.y))
                        .unwrap_or(0.0);
                    self.draw_vampire_sprite(screen_x, screen_y, size, facing_direction);
                }
                EntityType::ClanLeader(_) => {
                    self.draw_clan_leader_sprite(screen_x, screen_y, size, entity.color);
                }
                EntityType::HostileInfected => {
                    let facing_direction = entity
                        .velocity
                        .as_ref()
                        .map(|v| v.x.atan2(v.y))
                        .unwrap_or(0.0);
                    self.draw_infected_sprite(screen_x, screen_y, size, facing_direction);
                }
                EntityType::Animal => {
                    self.draw_animal_sprite(screen_x, screen_y, size);
                }
                EntityType::ClanMember(_) => {
                    self.draw_clan_member_sprite(screen_x, screen_y, size, entity.color);
                }
                EntityType::Shelter => unreachable!(),
            }

            // Draw health bar only if not skipping details and entity is close enough
            if let Some(health) = &entity.health {
                if !skip_details {
                    let distance_to_camera = ((entity.position.x - game_state.camera_x).powi(2)
                        + (entity.position.y - game_state.camera_y).powi(2))
                    .sqrt();

                    // Only draw health bars for entities within reasonable distance
                    let health_bar_distance = if self.performance_mode { 150.0 } else { 300.0 };
                    if distance_to_camera < health_bar_distance {
                        self.draw_health_bar(screen_x, screen_y, size, health);
                    }
                }
            }
        }
    }

    fn draw_health_bar(&self, screen_x: f32, screen_y: f32, entity_size: f32, health: &Health) {
        let bar_width = entity_size;
        let bar_height = 6.0;
        let bar_y = screen_y - entity_size / 2.0 - 12.0;

        // Background bar
        draw_rectangle(
            screen_x - bar_width / 2.0,
            bar_y,
            bar_width,
            bar_height,
            Color::new(0.3, 0.0, 0.0, 0.8),
        );

        // Health bar
        let health_percentage = health.current / health.max;
        let health_width = bar_width * health_percentage;
        let health_color = if health_percentage > 0.6 {
            GREEN
        } else if health_percentage > 0.3 {
            YELLOW
        } else {
            RED
        };

        draw_rectangle(
            screen_x - bar_width / 2.0,
            bar_y,
            health_width,
            bar_height,
            health_color,
        );
    }

    fn draw_ui(&self, game_state: &GameState) {
        // Time display
        let time_text = format!(
            "Time: {} - Day {}",
            game_state.time.get_time_string(),
            game_state.time.day_count()
        );
        self.draw_text_with_font(&time_text, 20.0, 30.0, 24.0, WHITE);

        // Day/night indicator
        let day_text = if game_state.time.is_day() {
            "DAY"
        } else {
            "NIGHT"
        };
        let day_color = if game_state.time.is_day() {
            YELLOW
        } else {
            BLUE
        };
        self.draw_text_with_font(day_text, 20.0, 60.0, 24.0, day_color);

        // Player stats
        if let Some(player) = game_state
            .entities
            .iter()
            .find(|e| e.id == game_state.player_id)
        {
            let mut y_offset = 100.0;

            // Health bar
            if let Some(health) = &player.health {
                draw_rectangle(20.0, y_offset, 200.0, 20.0, Color::new(0.3, 0.0, 0.0, 1.0));
                let health_width = 200.0 * (health.current / health.max);
                draw_rectangle(20.0, y_offset, health_width, 20.0, RED);
                self.draw_text_with_font("Health", 20.0, y_offset - 5.0, 16.0, WHITE);
                y_offset += 30.0;
            }

            // Blood bar
            if let Some(blood) = &player.blood_meter {
                draw_rectangle(20.0, y_offset, 200.0, 20.0, Color::new(0.0, 0.0, 0.3, 1.0));
                let blood_width = 200.0 * (blood.current / blood.maximum);
                draw_rectangle(20.0, y_offset, blood_width, 20.0, BLUE);
                self.draw_text_with_font("Blood", 20.0, y_offset - 5.0, 16.0, WHITE);
                y_offset += 30.0;
            }

            // Phase info
            self.draw_text_with_font(
                &format!("Phase: {:?}", game_state.phase),
                20.0,
                y_offset,
                18.0,
                WHITE,
            );
            y_offset += 25.0;

            // Stats
            self.draw_text_with_font(
                &format!(
                    "Kills: {} | Feedings: {}",
                    game_state.kills, game_state.feeding_count
                ),
                20.0,
                y_offset,
                18.0,
                WHITE,
            );
            y_offset += 25.0;

            // Shelter status
            if game_state.is_player_in_shelter() {
                let protection = game_state.get_player_shelter_protection();
                let protection_text =
                    format!("In Shelter - {}% Protection", (protection * 100.0) as u32);
                self.draw_text_with_font(&protection_text, 20.0, y_offset, 18.0, GREEN);
                y_offset += 25.0;
            } else if game_state.time.is_day() && game_state.time.get_sunlight_intensity() > 0.0 {
                let danger_text = "EXPOSED TO SUNLIGHT!";
                self.draw_text_with_font(danger_text, 20.0, y_offset, 18.0, RED);
                y_offset += 25.0;
            }

            // Nearby shelters
            let nearby_shelters = game_state.get_nearby_shelters();
            if !nearby_shelters.is_empty() {
                self.draw_text_with_font("Nearby Shelters:", 20.0, y_offset, 16.0, LIGHTGRAY);
                y_offset += 20.0;

                for (_i, shelter) in nearby_shelters.iter().take(3).enumerate() {
                    let shelter_text = if shelter.distance <= shelter.shelter_type.discovery_range()
                    {
                        format!("F: {}", shelter.get_description())
                    } else {
                        format!("? - {:.0}m away", shelter.distance)
                    };

                    let text_color = if shelter.discovered { WHITE } else { GRAY };

                    self.draw_text_with_font(&shelter_text, 25.0, y_offset, 14.0, text_color);
                    y_offset += 18.0;
                }
            }

            // Objectives
            self.draw_text_with_font("Objectives:", 20.0, y_offset, 18.0, YELLOW);
            y_offset += 25.0;

            for objective in &game_state.phase_objectives {
                self.draw_text_with_font(&format!("• {}", objective), 30.0, y_offset, 14.0, WHITE);
                y_offset += 18.0;
            }
        }

        // Controls
        let controls_y = screen_height() - 100.0;
        self.draw_text_with_font(
            "Controls: WASD=Move, R=Feed, E=Interact, Space=Attack, Tab=Clans, L=Legend, H=Help, Esc=Pause",
            20.0,
            controls_y,
            16.0,
            GRAY,
        );
    }

    fn draw_pause_menu(&self) {
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            Color::new(0.0, 0.0, 0.0, 0.7),
        );

        let center_x = screen_width() / 2.0;
        let center_y = screen_height() / 2.0;

        self.draw_text_with_font("PAUSED", center_x - 50.0, center_y - 50.0, 36.0, WHITE);
        self.draw_text_with_font(
            "Press ESC to Resume",
            center_x - 80.0,
            center_y,
            20.0,
            WHITE,
        );
    }

    fn draw_clan_menu(&self, game_state: &GameState) {
        draw_rectangle(
            50.0,
            50.0,
            screen_width() - 100.0,
            screen_height() - 100.0,
            Color::new(0.1, 0.1, 0.2, 0.9),
        );

        self.draw_text_with_font("CLAN RELATIONS", 70.0, 80.0, 24.0, WHITE);

        let mut y = 120.0;
        for clan in game_state.clans.values() {
            let status_color = if clan.is_allied { GREEN } else { RED };

            self.draw_text_with_font(&clan.name, 70.0, y, 20.0, WHITE);
            self.draw_text_with_font(
                &format!("Leader: {}", clan.leader_name),
                200.0,
                y,
                16.0,
                GRAY,
            );
            self.draw_text_with_font(
                &format!("Members: {}", clan.member_count),
                350.0,
                y,
                16.0,
                GRAY,
            );
            self.draw_text_with_font(
                &format!("Trust: {:.1}", clan.trust_towards_player),
                450.0,
                y,
                16.0,
                GRAY,
            );

            let status = if clan.is_allied { "Allied" } else { "Neutral" };
            self.draw_text_with_font(status, 550.0, y, 16.0, status_color);

            y += 25.0;
        }

        self.draw_text_with_font(
            "Press TAB to close",
            70.0,
            screen_height() - 40.0,
            18.0,
            LIGHTGRAY,
        );
    }

    fn draw_legend(&self, _game_state: &GameState) {
        // Semi-transparent background
        draw_rectangle(
            screen_width() - 320.0,
            50.0,
            270.0,
            400.0,
            Color::new(0.0, 0.0, 0.0, 0.8),
        );

        // Legend title
        self.draw_text_with_font("LEGEND", screen_width() - 310.0, 80.0, 24.0, WHITE);

        let mut y = 110.0;
        let legend_x = screen_width() - 310.0;
        let color_size = 15.0;
        let text_offset = 25.0;

        // Player - vampire with pixel art
        self.draw_vampire_sprite(
            legend_x + color_size / 2.0,
            y + color_size / 2.0,
            color_size * 1.5, // Larger for better visibility
            0.0,
        );
        self.draw_text_with_font(
            "Player (You) - Vampire with red cape",
            legend_x + text_offset,
            y,
            16.0,
            WHITE,
        );
        y += 25.0;

        // Clan Leaders with pixel art
        self.draw_clan_leader_sprite(
            legend_x + color_size / 2.0,
            y + color_size / 2.0,
            color_size * 1.5, // Larger for better visibility
            BEIGE,
        );
        draw_text(
            "Bone-Eaters Leader (Gold crown)",
            legend_x + text_offset,
            y + 12.0,
            14.0,
            WHITE,
        );
        y += 20.0;

        self.draw_clan_leader_sprite(
            legend_x + color_size / 2.0,
            y + color_size / 2.0,
            color_size * 1.5, // Larger for better visibility
            PURPLE,
        );
        draw_text(
            "Flame-Haters Leader (Gold crown)",
            legend_x + text_offset,
            y + 12.0,
            14.0,
            WHITE,
        );
        y += 20.0;

        self.draw_clan_leader_sprite(
            legend_x + color_size / 2.0,
            y + color_size / 2.0,
            color_size * 1.5, // Larger for better visibility
            DARKBLUE,
        );
        draw_text(
            "Night-Bloods Leader (Gold crown)",
            legend_x + text_offset,
            y + 12.0,
            14.0,
            WHITE,
        );
        y += 25.0;

        // Enemies with pixel art
        self.draw_infected_sprite(
            legend_x + color_size / 2.0,
            y + color_size / 2.0,
            color_size * 1.5, // Larger for better visibility
            0.0,
        );
        draw_text(
            "Hostile Infected (Red eyes, claws)",
            legend_x + text_offset,
            y + 12.0,
            14.0,
            WHITE,
        );
        y += 20.0;

        // Animals with pixel art
        self.draw_animal_sprite(
            legend_x + color_size / 2.0,
            y + color_size / 2.0,
            color_size * 1.5, // Larger for better visibility
        );
        draw_text(
            "Animals (Blood sources)",
            legend_x + text_offset,
            y + 12.0,
            14.0,
            WHITE,
        );
        y += 30.0;

        // Game mechanics legend
        draw_text("GAME TIPS:", legend_x, y, 18.0, YELLOW);
        y += 25.0;

        draw_text("• Red entities are hostile", legend_x, y, 14.0, LIGHTGRAY);
        y += 18.0;

        draw_text("• Feed on animals for blood", legend_x, y, 14.0, LIGHTGRAY);
        y += 18.0;

        draw_text("• Interact with clan leaders", legend_x, y, 14.0, LIGHTGRAY);
        y += 18.0;

        draw_text("• Avoid sunlight during day", legend_x, y, 14.0, LIGHTGRAY);
        y += 18.0;

        draw_text("• Watch your blood meter", legend_x, y, 14.0, LIGHTGRAY);
        y += 18.0;

        draw_text(
            "• Health bars show above entities",
            legend_x,
            y,
            14.0,
            LIGHTGRAY,
        );
        y += 18.0;

        draw_text(
            "• Ground terrain shows grass, dirt, stone",
            legend_x,
            y,
            14.0,
            LIGHTGRAY,
        );
        y += 18.0;

        draw_text(
            "• Stars twinkle in the night sky above",
            legend_x,
            y,
            14.0,
            LIGHTGRAY,
        );
        y += 25.0;

        draw_text("Press L to close", legend_x, y, 16.0, YELLOW);
    }

    fn draw_ground_cached(
        &mut self,
        game_state: &GameState,
        camera_offset_x: f32,
        camera_offset_y: f32,
    ) {
        // Increment frame skip counter
        self.frame_skip_counter += 1;

        let mut tiles_drawn = 0;
        let tile_cull_margin = if self.performance_mode { 30.0 } else { 80.0 };

        // Calculate camera movement speed for LOD
        let camera_speed = ((game_state.camera_x - self.last_camera_x).powi(2)
            + (game_state.camera_y - self.last_camera_y).powi(2))
        .sqrt();
        let is_moving_fast = camera_speed > 150.0;

        // Always draw ground, but vary detail level based on performance conditions
        for tile in &game_state.ground_tiles {
            let screen_x = tile.x * self.zoom_level + camera_offset_x;
            let screen_y = tile.y * self.zoom_level + camera_offset_y;

            // Only draw tiles that are visible on screen
            if screen_x > -tile_cull_margin
                && screen_x < screen_width() + tile_cull_margin
                && screen_y > -tile_cull_margin
                && screen_y < screen_height() + tile_cull_margin
            {
                // Determine detail level based on performance conditions
                let distance_from_center = ((screen_x - screen_width() / 2.0).powi(2)
                    + (screen_y - screen_height() / 2.0).powi(2))
                .sqrt();

                // Use simple rendering for performance optimization, but always render something
                let use_simple_rendering =
                    self.performance_mode || is_moving_fast || distance_from_center > 400.0;

                if use_simple_rendering {
                    self.draw_simple_ground_tile(
                        screen_x,
                        screen_y,
                        64.0 * self.zoom_level,
                        &tile.tile_type,
                    );
                } else {
                    self.draw_ground_tile_optimized(
                        screen_x,
                        screen_y,
                        64.0 * self.zoom_level,
                        tile,
                    );
                }
                tiles_drawn += 1;
            }
        }
        self.last_tile_count = tiles_drawn;
    }

    fn draw_ground_tile_optimized(&self, x: f32, y: f32, size: f32, tile: &GroundTile) {
        let scale = size / 64.0;

        match tile.tile_type {
            TileType::Grass => {
                // Base grass color
                draw_rectangle(x, y, size, size, Color::new(0.2, 0.4, 0.1, 1.0));

                // Optimized detail: draw fewer patches for performance
                for (i, (px_offset, py_offset, width, height)) in
                    tile.texture_data.grass_patches.iter().enumerate()
                {
                    if i % 3 == 0 {
                        // Draw every 3rd patch for good balance
                        let px = x + px_offset * scale;
                        let py = y + py_offset * scale;
                        draw_rectangle(
                            px,
                            py,
                            width * scale,
                            height * scale,
                            Color::new(0.3, 0.6, 0.2, 1.0),
                        );
                    }
                }
            }
            TileType::DeadGrass => {
                // Dead grass base
                draw_rectangle(x, y, size, size, Color::new(0.4, 0.3, 0.1, 1.0));

                // Optimized detail for dead grass
                for (i, (px_offset, py_offset, width, height)) in
                    tile.texture_data.grass_patches.iter().enumerate()
                {
                    if i % 3 == 0 {
                        let px = x + px_offset * scale;
                        let py = y + py_offset * scale;
                        draw_rectangle(
                            px,
                            py,
                            width * scale,
                            height * scale,
                            Color::new(0.5, 0.4, 0.2, 1.0),
                        );
                    }
                }
            }
            TileType::Dirt => {
                // Base dirt color
                draw_rectangle(x, y, size, size, Color::new(0.4, 0.2, 0.1, 1.0));

                // Optimized dirt spots
                for (i, (px_offset, py_offset, radius)) in
                    tile.texture_data.dirt_spots.iter().enumerate()
                {
                    if i % 2 == 0 {
                        // Draw every other spot
                        let px = x + px_offset * scale;
                        let py = y + py_offset * scale;
                        draw_circle(px, py, radius * scale, Color::new(0.3, 0.15, 0.05, 1.0));
                    }
                }
            }
            TileType::Stone => {
                // Simplified stone rendering
                draw_rectangle(x, y, size, size, Color::new(0.5, 0.5, 0.5, 1.0));

                // Optimized stone blocks
                for (i, (px_offset, py_offset, width, height)) in
                    tile.texture_data.stone_blocks.iter().enumerate()
                {
                    if i % 2 == 0 {
                        // Draw every other block
                        let px = x + px_offset * scale;
                        let py = y + py_offset * scale;
                        draw_rectangle(
                            px,
                            py,
                            width * scale,
                            height * scale,
                            Color::new(0.6, 0.6, 0.6, 1.0),
                        );
                    }
                }
            }
        }
    }

    fn draw_simple_ground_tile(&self, x: f32, y: f32, size: f32, tile_type: &TileType) {
        // Simplified tile rendering for performance mode
        let color = match tile_type {
            TileType::Grass => Color::new(0.2, 0.4, 0.1, 1.0),
            TileType::DeadGrass => Color::new(0.4, 0.3, 0.1, 1.0),
            TileType::Dirt => Color::new(0.4, 0.2, 0.1, 1.0),
            TileType::Stone => Color::new(0.5, 0.5, 0.5, 1.0),
        };
        draw_rectangle(x, y, size, size, color);
    }

    fn draw_moon(&self, game_state: &GameState, camera_offset_x: f32, camera_offset_y: f32) {
        let screen_x = game_state.moon.x * self.zoom_level + camera_offset_x;
        let screen_y = game_state.moon.y * self.zoom_level + camera_offset_y;

        // Only draw moon if on screen
        if screen_x > -50.0
            && screen_x < screen_width() + 50.0
            && screen_y > -50.0
            && screen_y < screen_height() + 50.0
        {
            let moon_size = if game_state.time.is_day() { 22.0 } else { 38.0 }; // Larger for zoom
            let moon_alpha = if game_state.time.is_day() {
                0.2
            } else {
                game_state.moon.glow_intensity
            };

            // Moon glow
            if !game_state.time.is_day() {
                draw_circle(
                    screen_x,
                    screen_y,
                    moon_size + 8.0,
                    Color::new(0.9, 0.9, 0.7, moon_alpha * 0.3),
                );
            }

            // Main moon body
            draw_circle(
                screen_x,
                screen_y,
                moon_size,
                Color::new(0.95, 0.95, 0.85, moon_alpha),
            );

            // Moon craters for detail
            if !game_state.time.is_day() {
                draw_circle(
                    screen_x - 6.0,
                    screen_y - 4.0,
                    3.0,
                    Color::new(0.8, 0.8, 0.7, moon_alpha * 0.6),
                );
                draw_circle(
                    screen_x + 4.0,
                    screen_y + 2.0,
                    2.0,
                    Color::new(0.8, 0.8, 0.7, moon_alpha * 0.6),
                );
                draw_circle(
                    screen_x - 2.0,
                    screen_y + 6.0,
                    1.5,
                    Color::new(0.8, 0.8, 0.7, moon_alpha * 0.6),
                );
            }
        }
    }

    fn draw_stars(&self, game_state: &GameState, camera_offset_x: f32, camera_offset_y: f32) {
        for star in &game_state.stars {
            let screen_x = star.x * self.zoom_level + camera_offset_x;
            let screen_y = star.y * self.zoom_level + camera_offset_y;

            // Only draw stars on screen
            if screen_x > -10.0
                && screen_x < screen_width() + 10.0
                && screen_y > -10.0
                && screen_y < screen_height() + 10.0
            {
                let alpha = star.brightness * if game_state.time.is_day() { 0.1 } else { 1.0 };
                draw_circle(screen_x, screen_y, 1.5, Color::new(1.0, 1.0, 0.9, alpha));
                // Slightly larger stars
            }
        }
    }

    fn draw_vampire_sprite(&self, x: f32, y: f32, size: f32, facing: f32) {
        let pixel_size = size / 8.0;

        // Main body (red)
        draw_rectangle(
            x - 2.0 * pixel_size,
            y - 3.0 * pixel_size,
            4.0 * pixel_size,
            6.0 * pixel_size,
            RED,
        );

        // Head (pale)
        draw_rectangle(
            x - 1.5 * pixel_size,
            y - 4.0 * pixel_size,
            3.0 * pixel_size,
            2.0 * pixel_size,
            Color::new(0.9, 0.8, 0.7, 1.0),
        );

        // Eyes (glowing red)
        draw_rectangle(
            x - 1.0 * pixel_size,
            y - 3.5 * pixel_size,
            pixel_size * 0.5,
            pixel_size * 0.5,
            Color::new(1.0, 0.2, 0.2, 1.0),
        );
        draw_rectangle(
            x + 0.5 * pixel_size,
            y - 3.5 * pixel_size,
            pixel_size * 0.5,
            pixel_size * 0.5,
            Color::new(1.0, 0.2, 0.2, 1.0),
        );

        // Cape (dark red)
        if facing.cos() > 0.0 {
            // Facing right
            draw_rectangle(
                x - 3.0 * pixel_size,
                y - 2.0 * pixel_size,
                2.0 * pixel_size,
                4.0 * pixel_size,
                Color::new(0.3, 0.0, 0.0, 1.0),
            );
        } else {
            // Facing left
            draw_rectangle(
                x + 1.0 * pixel_size,
                y - 2.0 * pixel_size,
                2.0 * pixel_size,
                4.0 * pixel_size,
                Color::new(0.3, 0.0, 0.0, 1.0),
            );
        }

        // Fangs
        draw_rectangle(
            x - 0.5 * pixel_size,
            y - 2.5 * pixel_size,
            pixel_size * 0.3,
            pixel_size * 0.5,
            WHITE,
        );
        draw_rectangle(
            x + 0.2 * pixel_size,
            y - 2.5 * pixel_size,
            pixel_size * 0.3,
            pixel_size * 0.5,
            WHITE,
        );

        // Border for visibility
        draw_rectangle_lines(
            x - 2.0 * pixel_size,
            y - 4.0 * pixel_size,
            4.0 * pixel_size,
            7.0 * pixel_size,
            1.0,
            WHITE,
        );
    }

    fn draw_clan_leader_sprite(&self, x: f32, y: f32, size: f32, color: Color) {
        let pixel_size = size / 10.0;

        // Body
        draw_rectangle(
            x - 2.5 * pixel_size,
            y - 2.0 * pixel_size,
            5.0 * pixel_size,
            4.0 * pixel_size,
            color,
        );

        // Head
        draw_rectangle(
            x - 2.0 * pixel_size,
            y - 4.0 * pixel_size,
            4.0 * pixel_size,
            2.0 * pixel_size,
            Color::new(0.8, 0.7, 0.6, 1.0),
        );

        // Crown
        draw_rectangle(
            x - 2.5 * pixel_size,
            y - 5.0 * pixel_size,
            5.0 * pixel_size,
            pixel_size,
            GOLD,
        );
        draw_triangle(
            Vec2::new(x, y - 5.5 * pixel_size),
            Vec2::new(x - pixel_size, y - 4.5 * pixel_size),
            Vec2::new(x + pixel_size, y - 4.5 * pixel_size),
            GOLD,
        );

        // Eyes
        draw_rectangle(
            x - 1.5 * pixel_size,
            y - 3.5 * pixel_size,
            pixel_size * 0.5,
            pixel_size * 0.5,
            BLACK,
        );
        draw_rectangle(
            x + pixel_size,
            y - 3.5 * pixel_size,
            pixel_size * 0.5,
            pixel_size * 0.5,
            BLACK,
        );

        // Weapon/Staff
        draw_rectangle(
            x + 3.0 * pixel_size,
            y - 4.0 * pixel_size,
            pixel_size * 0.5,
            6.0 * pixel_size,
            BROWN,
        );
        draw_circle(
            x + 3.25 * pixel_size,
            y - 4.5 * pixel_size,
            pixel_size * 0.8,
            color,
        );
    }

    fn draw_infected_sprite(&self, x: f32, y: f32, size: f32, facing: f32) {
        let pixel_size = size / 8.0;

        // Twisted body (dark red)
        draw_rectangle(
            x - 2.0 * pixel_size,
            y - 2.0 * pixel_size,
            4.0 * pixel_size,
            4.0 * pixel_size,
            Color::new(0.4, 0.1, 0.1, 1.0),
        );

        // Deformed head
        draw_rectangle(
            x - 1.5 * pixel_size,
            y - 3.5 * pixel_size,
            3.0 * pixel_size,
            1.5 * pixel_size,
            Color::new(0.5, 0.3, 0.2, 1.0),
        );

        // Glowing hostile eyes
        draw_rectangle(
            x - pixel_size,
            y - 3.0 * pixel_size,
            pixel_size * 0.7,
            pixel_size * 0.7,
            Color::new(1.0, 0.0, 0.0, 1.0),
        );
        draw_rectangle(
            x + 0.3 * pixel_size,
            y - 3.0 * pixel_size,
            pixel_size * 0.7,
            pixel_size * 0.7,
            Color::new(1.0, 0.0, 0.0, 1.0),
        );

        // Claws
        if facing.cos() > 0.0 {
            // Facing right
            for i in 0..3 {
                draw_rectangle(
                    x + 2.0 * pixel_size + i as f32 * pixel_size * 0.3,
                    y - pixel_size + i as f32 * pixel_size * 0.2,
                    pixel_size * 0.2,
                    pixel_size,
                    GRAY,
                );
            }
        } else {
            // Facing left
            for i in 0..3 {
                draw_rectangle(
                    x - 2.5 * pixel_size - i as f32 * pixel_size * 0.3,
                    y - pixel_size + i as f32 * pixel_size * 0.2,
                    pixel_size * 0.2,
                    pixel_size,
                    GRAY,
                );
            }
        }

        // Danger X mark
        draw_line(
            x - pixel_size,
            y - pixel_size,
            x + pixel_size,
            y + pixel_size,
            2.0,
            RED,
        );
        draw_line(
            x + pixel_size,
            y - pixel_size,
            x - pixel_size,
            y + pixel_size,
            2.0,
            RED,
        );
    }

    fn draw_animal_sprite(&self, x: f32, y: f32, size: f32) {
        let pixel_size = size / 6.0;

        // Body (brown circle with texture)
        draw_circle(x, y, size / 2.0, BROWN);
        draw_circle(x, y, size / 2.5, Color::new(0.4, 0.2, 0.1, 1.0));

        // Ears
        draw_triangle(
            Vec2::new(x - pixel_size, y - pixel_size * 1.5),
            Vec2::new(x - pixel_size * 1.5, y - pixel_size * 2.5),
            Vec2::new(x - pixel_size * 0.5, y - pixel_size * 2.0),
            BROWN,
        );
        draw_triangle(
            Vec2::new(x + pixel_size, y - pixel_size * 1.5),
            Vec2::new(x + pixel_size * 1.5, y - pixel_size * 2.5),
            Vec2::new(x + pixel_size * 0.5, y - pixel_size * 2.0),
            BROWN,
        );

        // Eyes
        draw_circle(
            x - pixel_size * 0.5,
            y - pixel_size * 0.3,
            pixel_size * 0.3,
            BLACK,
        );
        draw_circle(
            x + pixel_size * 0.5,
            y - pixel_size * 0.3,
            pixel_size * 0.3,
            BLACK,
        );

        // Nose
        draw_circle(x, y + pixel_size * 0.2, pixel_size * 0.2, BLACK);

        // Tail
        draw_circle(
            x + pixel_size * 1.8,
            y + pixel_size * 0.5,
            pixel_size * 0.4,
            BROWN,
        );
    }

    fn draw_clan_member_sprite(&self, x: f32, y: f32, size: f32, color: Color) {
        let pixel_size = size / 8.0;

        // Body
        draw_rectangle(
            x - 2.0 * pixel_size,
            y - 2.0 * pixel_size,
            4.0 * pixel_size,
            4.0 * pixel_size,
            color,
        );

        // Head
        draw_rectangle(
            x - 1.5 * pixel_size,
            y - 3.5 * pixel_size,
            3.0 * pixel_size,
            1.5 * pixel_size,
            Color::new(0.8, 0.7, 0.6, 1.0),
        );

        // Eyes
        draw_rectangle(
            x - pixel_size,
            y - 3.0 * pixel_size,
            pixel_size * 0.4,
            pixel_size * 0.4,
            BLACK,
        );
        draw_rectangle(
            x + 0.6 * pixel_size,
            y - 3.0 * pixel_size,
            pixel_size * 0.4,
            pixel_size * 0.4,
            BLACK,
        );

        // Simple weapon
        draw_rectangle(
            x + 2.5 * pixel_size,
            y - 3.0 * pixel_size,
            pixel_size * 0.3,
            4.0 * pixel_size,
            GRAY,
        );
    }

    fn draw_quick_start_guide(&self) {
        // Full screen overlay
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            Color::new(0.0, 0.0, 0.0, 0.85),
        );

        let center_x = screen_width() / 2.0;
        let mut y = 80.0;

        // Title
        self.draw_text_with_font(
            "VAMPIRE RPG - QUICK START GUIDE",
            center_x - 200.0,
            y,
            32.0,
            RED,
        );
        y += 60.0;

        // Story intro
        self.draw_text_with_font(
            "You are the sole survivor of a viral outbreak that created vampires.",
            center_x - 250.0,
            y,
            18.0,
            WHITE,
        );
        y += 25.0;

        self.draw_text_with_font(
            "You must survive, adapt, and eventually rule the savage clans.",
            center_x - 220.0,
            y,
            18.0,
            WHITE,
        );
        y += 25.0;

        self.draw_text_with_font(
            "The game features pixel art graphics, ground terrain, and a starry night sky.",
            center_x - 240.0,
            y,
            16.0,
            LIGHTGRAY,
        );
        y += 35.0;

        // Essential controls
        self.draw_text_with_font("ESSENTIAL CONTROLS:", center_x - 100.0, y, 20.0, YELLOW);
        y += 30.0;

        self.draw_text_with_font("WASD - Move around", center_x - 150.0, y, 16.0, LIGHTGRAY);
        y += 20.0;

        self.draw_text_with_font(
            "R - Feed on animals and enemies (restores blood & health)",
            center_x - 200.0,
            y,
            16.0,
            LIGHTGRAY,
        );
        y += 20.0;

        self.draw_text_with_font(
            "Space - Attack hostile infected (red-eyed creatures with claws)",
            center_x - 200.0,
            y,
            16.0,
            LIGHTGRAY,
        );
        y += 20.0;

        self.draw_text_with_font(
            "E - Interact with clan leaders (pixel warriors with gold crowns)",
            center_x - 210.0,
            y,
            16.0,
            LIGHTGRAY,
        );
        y += 30.0;

        // Survival tips
        self.draw_text_with_font("SURVIVAL TIPS:", center_x - 70.0, y, 20.0, YELLOW);
        y += 30.0;

        self.draw_text_with_font(
            "• Keep your BLOOD meter above 20% or you'll take damage",
            center_x - 200.0,
            y,
            16.0,
            LIGHTGRAY,
        );
        y += 20.0;

        self.draw_text_with_font(
            "• Avoid sunlight during DAY - it damages you significantly",
            center_x - 200.0,
            y,
            16.0,
            LIGHTGRAY,
        );
        y += 20.0;

        self.draw_text_with_font(
            "• Feed on small animals (creatures with ears and tails) on the ground",
            center_x - 200.0,
            y,
            16.0,
            LIGHTGRAY,
        );
        y += 20.0;

        self.draw_text_with_font(
            "• Build trust with clan leaders by repeatedly pressing E near them",
            center_x - 220.0,
            y,
            16.0,
            LIGHTGRAY,
        );
        y += 20.0;

        self.draw_text_with_font(
            "• Your abilities improve each time you feed",
            center_x - 160.0,
            y,
            16.0,
            LIGHTGRAY,
        );
        y += 20.0;

        self.draw_text_with_font(
            "• Walk on varied ground terrain (grass, dirt, stone)",
            center_x - 170.0,
            y,
            16.0,
            LIGHTGRAY,
        );
        y += 40.0;

        // Legend reference
        self.draw_text_with_font(
            "Press L for detailed LEGEND • Press Tab for CLAN RELATIONS",
            center_x - 200.0,
            y,
            16.0,
            YELLOW,
        );
        y += 40.0;

        // Close instructions
        self.draw_text_with_font(
            "Press H to toggle this guide • Start moving (WASD) to begin!",
            center_x - 200.0,
            y,
            18.0,
            WHITE,
        );
    }

    fn draw_debug_messages(&self, game_state: &GameState) {
        let right_margin = 20.0;
        let debug_x = screen_width() - 400.0 - right_margin;
        let mut debug_y = 50.0;

        // Draw background for debug messages
        draw_rectangle(
            debug_x - 10.0,
            debug_y - 30.0,
            410.0,
            (game_state.debug_messages.len() as f32 * 18.0) + 40.0,
            Color::new(0.0, 0.0, 0.0, 0.7),
        );

        // Draw title
        self.draw_text_with_font("DEBUG LOG", debug_x, debug_y, 16.0, YELLOW);
        debug_y += 25.0;

        // Draw messages
        for message in &game_state.debug_messages {
            self.draw_text_with_font(message, debug_x, debug_y, 12.0, WHITE);
            debug_y += 18.0;
        }
    }
}
