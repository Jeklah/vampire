//! Rendering Module
//!
//! This module handles all rendering and drawing operations for the Vampire RPG.

use crate::components::*;
use crate::game_state::GameState;
use macroquad::prelude::*;

pub struct Renderer {
    zoom_level: f32,
}

impl Renderer {
    pub fn new() -> Self {
        Self { zoom_level: 1.5 }
    }

    pub fn render(&self, game_state: &GameState) {
        clear_background(Color::new(0.05, 0.05, 0.15, 1.0)); // Dark blue night sky

        // Calculate camera offset with zoom
        let camera_offset_x = screen_width() / 2.0 - game_state.camera_x * self.zoom_level;
        let camera_offset_y = screen_height() / 2.0 - game_state.camera_y * self.zoom_level;

        // Draw ground first (background layer)
        self.draw_ground(game_state, camera_offset_x, camera_offset_y);

        // Draw stars and moon
        self.draw_stars(game_state, camera_offset_x, camera_offset_y);
        self.draw_moon(game_state, camera_offset_x, camera_offset_y);

        // Draw blood particles
        for particle in &game_state.blood_particles {
            particle.draw(camera_offset_x, camera_offset_y);
        }

        // Draw all entities
        self.draw_entities(game_state, camera_offset_x, camera_offset_y);

        // Draw UI
        self.draw_ui(game_state);

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
        for entity in &game_state.entities {
            let screen_x = entity.position.x * self.zoom_level + camera_offset_x;
            let screen_y = entity.position.y * self.zoom_level + camera_offset_y;

            // Only draw if on screen
            if screen_x > -40.0
                && screen_x < screen_width() + 40.0
                && screen_y > -40.0
                && screen_y < screen_height() + 40.0
            {
                let size = match entity.entity_type {
                    EntityType::Player => 30.0, // Larger for better detail
                    EntityType::ClanLeader(_) => 28.0,
                    EntityType::ClanMember(_) => 24.0,
                    EntityType::HostileInfected => 20.0,
                    EntityType::Animal => 16.0,
                };

                // Don't draw dead entities
                if let Some(health) = &entity.health {
                    if health.current <= 0.0 || matches!(entity.ai_state, AIState::Dead) {
                        continue;
                    }
                }

                // Draw entity with pixel art
                match entity.entity_type {
                    EntityType::Player => {
                        self.draw_vampire_sprite(screen_x, screen_y, size, entity.facing_direction);
                    }
                    EntityType::ClanLeader(_) => {
                        self.draw_clan_leader_sprite(screen_x, screen_y, size, entity.color);
                    }
                    EntityType::HostileInfected => {
                        self.draw_infected_sprite(
                            screen_x,
                            screen_y,
                            size,
                            entity.facing_direction,
                        );
                    }
                    EntityType::Animal => {
                        self.draw_animal_sprite(screen_x, screen_y, size);
                    }
                    EntityType::ClanMember(_) => {
                        self.draw_clan_member_sprite(screen_x, screen_y, size, entity.color);
                    }
                }

                // Draw health bar if entity has health
                if let Some(health) = &entity.health {
                    let bar_width = size;
                    let bar_height = 6.0; // Slightly thicker for zoom
                    let bar_y = screen_y - size / 2.0 - 12.0; // More space for zoom

                    // Background bar
                    draw_rectangle(
                        screen_x - bar_width / 2.0,
                        bar_y,
                        bar_width,
                        bar_height,
                        Color::new(0.3, 0.0, 0.0, 0.8),
                    );

                    // Health bar
                    let health_percentage = health.current / health.maximum;
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
            }
        }
    }

    fn draw_ui(&self, game_state: &GameState) {
        // Time display
        let time_text = format!(
            "Time: {} - Day {}",
            game_state.time.get_time_string(),
            game_state.time.day_count
        );
        draw_text(&time_text, 20.0, 30.0, 24.0, WHITE);

        // Day/night indicator
        let day_text = if game_state.time.is_day {
            "DAY"
        } else {
            "NIGHT"
        };
        let day_color = if game_state.time.is_day { YELLOW } else { BLUE };
        draw_text(day_text, 20.0, 60.0, 24.0, day_color);

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
                let health_width = 200.0 * (health.current / health.maximum);
                draw_rectangle(20.0, y_offset, health_width, 20.0, RED);
                draw_text("Health", 20.0, y_offset - 5.0, 16.0, WHITE);
                y_offset += 30.0;
            }

            // Blood bar
            if let Some(blood) = &player.blood_meter {
                draw_rectangle(20.0, y_offset, 200.0, 20.0, Color::new(0.0, 0.0, 0.3, 1.0));
                let blood_width = 200.0 * (blood.current / blood.maximum);
                draw_rectangle(20.0, y_offset, blood_width, 20.0, BLUE);
                draw_text("Blood", 20.0, y_offset - 5.0, 16.0, WHITE);
                y_offset += 30.0;
            }

            // Phase info
            draw_text(
                &format!("Phase: {:?}", game_state.phase),
                20.0,
                y_offset,
                16.0,
                WHITE,
            );
            y_offset += 20.0;

            // Stats
            draw_text(
                &format!(
                    "Kills: {} | Feedings: {}",
                    game_state.kills, game_state.feeding_count
                ),
                20.0,
                y_offset,
                14.0,
                GRAY,
            );
            y_offset += 20.0;

            // Objectives
            draw_text("Objectives:", 20.0, y_offset, 18.0, YELLOW);
            y_offset += 25.0;

            for objective in &game_state.phase_objectives {
                draw_text(&format!("• {}", objective), 30.0, y_offset, 14.0, WHITE);
                y_offset += 18.0;
            }
        }

        // Controls
        let controls_y = screen_height() - 100.0;
        draw_text(
            "Controls: WASD=Move, R=Feed, E=Interact, Space=Attack, Tab=Clans, L=Legend, H=Help, Esc=Pause",
            20.0,
            controls_y,
            16.0,
            LIGHTGRAY,
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

        draw_text("PAUSED", center_x - 50.0, center_y - 50.0, 36.0, WHITE);
        draw_text(
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

        draw_text("CLAN RELATIONS", 70.0, 80.0, 24.0, WHITE);

        let mut y = 120.0;
        for clan in game_state.clans.values() {
            let status_color = if clan.is_allied { GREEN } else { RED };

            draw_text(&clan.name, 70.0, y, 20.0, WHITE);
            draw_text(
                &format!("Leader: {}", clan.leader_name),
                200.0,
                y,
                16.0,
                GRAY,
            );
            draw_text(
                &format!("Members: {}", clan.member_count),
                350.0,
                y,
                16.0,
                GRAY,
            );
            draw_text(
                &format!("Trust: {:.1}", clan.trust_towards_player),
                450.0,
                y,
                16.0,
                GRAY,
            );

            let status = if clan.is_allied { "Allied" } else { "Neutral" };
            draw_text(status, 550.0, y, 16.0, status_color);

            y += 25.0;
        }

        draw_text(
            "Press TAB to close",
            70.0,
            screen_height() - 80.0,
            16.0,
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
        draw_text("LEGEND", screen_width() - 310.0, 80.0, 24.0, WHITE);

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
        draw_text(
            "Player (You) - Vampire with red cape",
            legend_x + text_offset,
            y + 12.0,
            14.0,
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

    fn draw_ground(&self, game_state: &GameState, camera_offset_x: f32, camera_offset_y: f32) {
        for tile in &game_state.ground_tiles {
            let screen_x = tile.x * self.zoom_level + camera_offset_x;
            let screen_y = tile.y * self.zoom_level + camera_offset_y;

            // Only draw tiles that are visible on screen
            if screen_x > -100.0
                && screen_x < screen_width() + 100.0
                && screen_y > -100.0
                && screen_y < screen_height() + 100.0
            {
                self.draw_ground_tile(screen_x, screen_y, 64.0 * self.zoom_level, &tile.tile_type);
            }
        }
    }

    fn draw_ground_tile(&self, x: f32, y: f32, size: f32, tile_type: &TileType) {
        let pixel_size = size / 16.0; // 16x16 pixel tiles

        match tile_type {
            TileType::Grass => {
                // Base grass color
                draw_rectangle(x, y, size, size, Color::new(0.2, 0.4, 0.1, 1.0));

                // Grass texture - small green pixels
                for i in 0..8 {
                    for j in 0..4 {
                        let px = x
                            + (i as f32 * pixel_size * 2.0)
                            + rand::gen_range(-pixel_size, pixel_size);
                        let py = y
                            + (j as f32 * pixel_size * 4.0)
                            + rand::gen_range(-pixel_size, pixel_size);
                        draw_rectangle(
                            px,
                            py,
                            pixel_size,
                            pixel_size * 2.0,
                            Color::new(0.3, 0.6, 0.2, 1.0),
                        );
                    }
                }
            }
            TileType::DeadGrass => {
                // Dead grass base
                draw_rectangle(x, y, size, size, Color::new(0.4, 0.3, 0.1, 1.0));

                // Dead grass texture
                for i in 0..6 {
                    for j in 0..3 {
                        let px = x
                            + (i as f32 * pixel_size * 2.5)
                            + rand::gen_range(-pixel_size, pixel_size);
                        let py = y
                            + (j as f32 * pixel_size * 5.0)
                            + rand::gen_range(-pixel_size, pixel_size);
                        draw_rectangle(
                            px,
                            py,
                            pixel_size,
                            pixel_size * 2.0,
                            Color::new(0.5, 0.4, 0.2, 1.0),
                        );
                    }
                }
            }
            TileType::Dirt => {
                // Base dirt color
                draw_rectangle(x, y, size, size, Color::new(0.4, 0.2, 0.1, 1.0));

                // Dirt texture - small darker spots
                for _i in 0..12 {
                    let px = x + rand::gen_range(0.0, size);
                    let py = y + rand::gen_range(0.0, size);
                    draw_circle(px, py, pixel_size * 0.8, Color::new(0.3, 0.15, 0.05, 1.0));
                }
            }
            TileType::Stone => {
                // Base stone color
                draw_rectangle(x, y, size, size, Color::new(0.5, 0.5, 0.5, 1.0));

                // Stone texture - rectangular patterns
                for i in 0..4 {
                    for j in 0..4 {
                        let px = x + (i as f32 * pixel_size * 4.0);
                        let py = y + (j as f32 * pixel_size * 4.0);
                        draw_rectangle(
                            px,
                            py,
                            pixel_size * 3.0,
                            pixel_size * 3.0,
                            Color::new(0.6, 0.6, 0.6, 1.0),
                        );
                        draw_rectangle_lines(
                            px,
                            py,
                            pixel_size * 3.0,
                            pixel_size * 3.0,
                            1.0,
                            Color::new(0.4, 0.4, 0.4, 1.0),
                        );
                    }
                }
            }
        }
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
            let moon_size = if game_state.time.is_day { 22.0 } else { 38.0 }; // Larger for zoom
            let moon_alpha = if game_state.time.is_day {
                0.2
            } else {
                game_state.moon.glow_intensity
            };

            // Moon glow
            if !game_state.time.is_day {
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
            if !game_state.time.is_day {
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
                let alpha = star.brightness * if game_state.time.is_day { 0.1 } else { 1.0 };
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
        draw_text(
            "VAMPIRE RPG - QUICK START GUIDE",
            center_x - 200.0,
            y,
            32.0,
            RED,
        );
        y += 60.0;

        // Story intro
        draw_text(
            "You are the sole survivor of a viral outbreak that created vampires.",
            center_x - 250.0,
            y,
            18.0,
            WHITE,
        );
        y += 25.0;

        draw_text(
            "You must survive, adapt, and eventually rule the savage clans.",
            center_x - 220.0,
            y,
            18.0,
            WHITE,
        );
        y += 25.0;

        draw_text(
            "The game features pixel art graphics, ground terrain, and a starry night sky.",
            center_x - 240.0,
            y,
            16.0,
            LIGHTGRAY,
        );
        y += 35.0;

        // Essential controls
        draw_text("ESSENTIAL CONTROLS:", center_x - 100.0, y, 20.0, YELLOW);
        y += 30.0;

        draw_text("WASD - Move around", center_x - 150.0, y, 16.0, LIGHTGRAY);
        y += 20.0;

        draw_text(
            "R - Feed on animals and enemies (restores blood & health)",
            center_x - 200.0,
            y,
            16.0,
            LIGHTGRAY,
        );
        y += 20.0;

        draw_text(
            "Space - Attack hostile infected (red-eyed creatures with claws)",
            center_x - 200.0,
            y,
            16.0,
            LIGHTGRAY,
        );
        y += 20.0;

        draw_text(
            "E - Interact with clan leaders (pixel warriors with gold crowns)",
            center_x - 210.0,
            y,
            16.0,
            LIGHTGRAY,
        );
        y += 30.0;

        // Survival tips
        draw_text("SURVIVAL TIPS:", center_x - 70.0, y, 20.0, YELLOW);
        y += 30.0;

        draw_text(
            "• Keep your BLOOD meter above 20% or you'll take damage",
            center_x - 200.0,
            y,
            16.0,
            LIGHTGRAY,
        );
        y += 20.0;

        draw_text(
            "• Avoid sunlight during DAY - it damages you significantly",
            center_x - 200.0,
            y,
            16.0,
            LIGHTGRAY,
        );
        y += 20.0;

        draw_text(
            "• Feed on small animals (creatures with ears and tails) on the ground",
            center_x - 200.0,
            y,
            16.0,
            LIGHTGRAY,
        );
        y += 20.0;

        draw_text(
            "• Build trust with clan leaders by repeatedly pressing E near them",
            center_x - 220.0,
            y,
            16.0,
            LIGHTGRAY,
        );
        y += 20.0;

        draw_text(
            "• Your abilities improve each time you feed",
            center_x - 160.0,
            y,
            16.0,
            LIGHTGRAY,
        );
        y += 20.0;

        draw_text(
            "• Walk on varied ground terrain (grass, dirt, stone)",
            center_x - 170.0,
            y,
            16.0,
            LIGHTGRAY,
        );
        y += 40.0;

        // Legend reference
        draw_text(
            "Press L for detailed LEGEND • Press Tab for CLAN RELATIONS",
            center_x - 200.0,
            y,
            16.0,
            YELLOW,
        );
        y += 40.0;

        // Close instructions
        draw_text(
            "Press H to toggle this guide • Start moving (WASD) to begin!",
            center_x - 200.0,
            y,
            18.0,
            GREEN,
        );
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}
