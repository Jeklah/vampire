//! Shelter System Module
//!
//! Manages shelter mechanics, sunlight protection, and shelter interactions
//! in the Vampire RPG. This system handles shelter discovery, occupancy,
//! and protection calculations against deadly sunlight.

use crate::components::*;
use macroquad::prelude::*;

/// Shelter system responsible for managing all shelter-related mechanics
pub struct ShelterSystem;

impl ShelterSystem {
    /// Update all shelter-related mechanics
    pub fn update_shelters(
        entities: &mut Vec<GameEntity>,
        current_time: f32,
        sunlight_intensity: f32,
        delta_time: f32,
    ) {
        // Update shelter conditions and occupancy
        Self::update_shelter_conditions(entities, current_time, delta_time);

        // Handle automatic shelter seeking for NPCs during dangerous sunlight
        Self::handle_npc_shelter_seeking(entities, current_time, sunlight_intensity);

        // Apply shelter protection effects
        Self::apply_shelter_protection(entities, sunlight_intensity);

        // Clean up invalid occupancy references
        Self::cleanup_occupancy_references(entities);
    }

    /// Handle player attempting to enter/exit shelter
    pub fn handle_player_shelter_interaction(
        entities: &mut Vec<GameEntity>,
        player_id: u32,
        current_time: f32,
    ) -> Option<String> {
        let player_pos = entities.iter().find(|e| e.id == player_id)?.position;

        // Check if player is already in a shelter
        if let Some(player) = entities.iter_mut().find(|e| e.id == player_id) {
            if let Some(occupancy) = &mut player.shelter_occupancy {
                if occupancy.is_in_shelter() {
                    // Exit shelter
                    let shelter_id = occupancy.shelter_id.unwrap();
                    occupancy.leave_shelter();

                    // Remove from shelter's occupant list
                    if let Some(shelter_entity) = entities.iter_mut().find(|e| e.id == shelter_id) {
                        if let Some(shelter) = &mut shelter_entity.shelter {
                            shelter.remove_occupant(player_id);
                        }
                    }

                    return Some("Exited shelter".to_string());
                }
            }
        }

        // Look for nearby shelters to enter
        let mut nearest_shelter: Option<(u32, f32, String)> = None;
        let mut nearby_shelters_found = 0;

        for entity in entities.iter() {
            if let Some(shelter) = &entity.shelter {
                let distance = ((player_pos.x - entity.position.x).powi(2)
                    + (player_pos.y - entity.position.y).powi(2))
                .sqrt();

                if distance <= shelter.shelter_type.discovery_range() {
                    nearby_shelters_found += 1;
                    let shelter_name = shelter.get_status_text();

                    if nearest_shelter.is_none() || distance < nearest_shelter.as_ref().unwrap().1 {
                        nearest_shelter = Some((entity.id, distance, shelter_name));
                    }
                }
            }
        }

        if let Some((shelter_id, distance, shelter_name)) = nearest_shelter {
            // Try to enter the shelter
            if let Some(shelter_entity) = entities.iter_mut().find(|e| e.id == shelter_id) {
                if let Some(shelter) = &mut shelter_entity.shelter {
                    if shelter.can_accommodate() {
                        shelter.discover();

                        if shelter.add_occupant(player_id) {
                            // Update player's occupancy
                            if let Some(player) = entities.iter_mut().find(|e| e.id == player_id) {
                                if player.shelter_occupancy.is_none() {
                                    player.shelter_occupancy = Some(ShelterOccupancy::new());
                                }
                                if let Some(occupancy) = &mut player.shelter_occupancy {
                                    occupancy.enter_shelter(shelter_id, current_time);
                                }
                            }

                            return Some(format!(
                                "Entered {} (distance: {:.1})",
                                shelter_name, distance
                            ));
                        } else {
                            return Some("Shelter is full".to_string());
                        }
                    } else {
                        return Some(format!("Shelter cannot be entered: {}", shelter_name));
                    }
                }
            }
        } else {
            // No shelters nearby - provide helpful feedback
            let total_shelters = entities.iter().filter(|e| e.shelter.is_some()).count();
            if total_shelters == 0 {
                return Some("No shelters found in the world".to_string());
            } else {
                return Some(format!(
                    "No shelters nearby (found {} shelters in world, {} within discovery range)",
                    total_shelters, nearby_shelters_found
                ));
            }
        }

        None
    }

    /// Get shelter information for nearby shelters (for UI display)
    pub fn get_nearby_shelter_info(
        entities: &[GameEntity],
        player_id: u32,
        max_distance: f32,
    ) -> Vec<ShelterInfo> {
        let player_pos = match entities.iter().find(|e| e.id == player_id) {
            Some(player) => player.position,
            None => return Vec::new(),
        };

        let mut shelter_info: Vec<ShelterInfo> = entities
            .iter()
            .filter_map(|entity| {
                entity.shelter.as_ref().map(|shelter| {
                    let distance = ((player_pos.x - entity.position.x).powi(2)
                        + (player_pos.y - entity.position.y).powi(2))
                    .sqrt();
                    (entity, shelter, distance)
                })
            })
            .filter(|(_, shelter, distance)| {
                *distance <= max_distance
                    && (shelter.discovered || *distance <= shelter.shelter_type.discovery_range())
            })
            .map(|(entity, shelter, distance)| ShelterInfo {
                id: entity.id,
                position: entity.position,
                shelter_type: shelter.shelter_type.clone(),
                condition: shelter.condition.clone(),
                protection_level: shelter.effective_protection(),
                occupancy: format!(
                    "{}/{}",
                    shelter.occupant_count(),
                    shelter.shelter_type.max_capacity()
                ),
                distance,
                discovered: shelter.discovered,
                name: shelter.name.clone(),
            })
            .collect();

        // Sort by distance
        shelter_info.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
        shelter_info
    }

    /// Check if an entity is currently protected by shelter
    pub fn is_protected_by_shelter(
        entities: &[GameEntity],
        entity_id: u32,
        sunlight_intensity: f32,
    ) -> bool {
        let entity = match entities.iter().find(|e| e.id == entity_id) {
            Some(e) => e,
            None => return false,
        };

        if let Some(occupancy) = &entity.shelter_occupancy {
            if let Some(shelter_id) = occupancy.shelter_id {
                if let Some(shelter_entity) = entities.iter().find(|e| e.id == shelter_id) {
                    if let Some(shelter) = &shelter_entity.shelter {
                        return shelter.provides_adequate_protection(sunlight_intensity);
                    }
                }
            }
        }

        false
    }

    /// Calculate the effective sunlight damage after shelter protection
    pub fn calculate_shelter_protection(
        entities: &[GameEntity],
        entity_id: u32,
        base_sunlight_damage: f32,
    ) -> f32 {
        let entity = match entities.iter().find(|e| e.id == entity_id) {
            Some(e) => e,
            None => return base_sunlight_damage,
        };

        if let Some(occupancy) = &entity.shelter_occupancy {
            if let Some(shelter_id) = occupancy.shelter_id {
                if let Some(shelter_entity) = entities.iter().find(|e| e.id == shelter_id) {
                    if let Some(shelter) = &shelter_entity.shelter {
                        let protection = shelter.effective_protection();
                        return base_sunlight_damage * (1.0 - protection).max(0.0);
                    }
                }
            }
        }

        base_sunlight_damage
    }

    /// Check if a position has ground (is within the ground area)
    pub fn has_ground_at_position(x: f32, y: f32) -> bool {
        let world_width = 1600.0;
        let world_height = 1200.0;
        let ground_level = 640.0; // Ground starts at y = 640 (aligned with tile positions)

        // Check if position is within world bounds and at or below ground level
        x >= 0.0 && x <= world_width && y >= ground_level && y <= world_height
    }

    /// Spawn a shelter at the specified location with ground validation
    pub fn spawn_shelter_safe(
        entities: &mut Vec<GameEntity>,
        next_entity_id: &mut u32,
        shelter_type: ShelterType,
        x: f32,
        y: f32,
        condition: Option<ShelterCondition>,
        name: Option<String>,
    ) -> Option<u32> {
        // Check if the position has ground
        if !Self::has_ground_at_position(x, y) {
            eprintln!(
                "Warning: Cannot spawn shelter at ({}, {}) - no ground at this position",
                x, y
            );
            return None;
        }

        Some(Self::spawn_shelter(
            entities,
            next_entity_id,
            shelter_type,
            x,
            y,
            condition,
            name,
        ))
    }

    /// Spawn a shelter at the specified location
    pub fn spawn_shelter(
        entities: &mut Vec<GameEntity>,
        next_entity_id: &mut u32,
        shelter_type: ShelterType,
        x: f32,
        y: f32,
        condition: Option<ShelterCondition>,
        name: Option<String>,
    ) -> u32 {
        // Warn if spawning on invalid ground
        if !Self::has_ground_at_position(x, y) {
            eprintln!(
                "Warning: Spawning shelter at ({}, {}) where there is no ground",
                x, y
            );
        }

        let id = *next_entity_id;
        *next_entity_id += 1;

        let mut shelter = match condition {
            Some(cond) => Shelter::with_condition(shelter_type, cond),
            None => Shelter::new(shelter_type),
        };

        if let Some(shelter_name) = name {
            shelter.name = Some(shelter_name);
        }

        let entity = GameEntity {
            id,
            position: Position { x, y },
            velocity: Some(Velocity { x: 0.0, y: 0.0 }),
            entity_type: EntityType::Shelter,
            health: None,
            combat_stats: None,
            ai_state: AIState::Idle,
            blood_meter: None,
            vampire_abilities: None,
            shelter: Some(shelter),
            shelter_occupancy: None,
            color: WHITE, // Will be overridden by shelter rendering
        };

        entities.push(entity);
        id
    }

    /// Find the nearest available shelter for an entity
    pub fn find_nearest_available_shelter(
        entities: &[GameEntity],
        entity_pos: Position,
        max_distance: f32,
    ) -> Option<u32> {
        let mut nearest: Option<(u32, f32)> = None;

        for entity in entities {
            if let Some(shelter) = &entity.shelter {
                let distance = ((entity_pos.x - entity.position.x).powi(2)
                    + (entity_pos.y - entity.position.y).powi(2))
                .sqrt();

                if distance <= max_distance && shelter.can_accommodate() {
                    if nearest.is_none() || distance < nearest.as_ref().unwrap().1 {
                        nearest = Some((entity.id, distance));
                    }
                }
            }
        }

        nearest.map(|(id, _)| id)
    }

    /// Update shelter conditions over time
    fn update_shelter_conditions(entities: &mut [GameEntity], current_time: f32, delta_time: f32) {
        for entity in entities {
            if let Some(shelter) = &mut entity.shelter {
                shelter.update(delta_time, current_time);
            }
        }
    }

    /// Handle NPCs automatically seeking shelter during dangerous sunlight
    fn handle_npc_shelter_seeking(
        entities: &mut Vec<GameEntity>,
        current_time: f32,
        sunlight_intensity: f32,
    ) {
        let dangerous_sunlight = sunlight_intensity > 0.6;
        let mut shelter_requests: Vec<(u32, Position)> = Vec::new();

        // First pass: identify NPCs that need shelter
        for entity in entities.iter_mut() {
            // Skip player - they manage their own shelter
            if entity.entity_type == EntityType::Player {
                continue;
            }

            // Skip entities that don't have vampire abilities (not affected by sunlight)
            if entity.vampire_abilities.is_none() {
                continue;
            }

            if entity.shelter_occupancy.is_none() {
                entity.shelter_occupancy = Some(ShelterOccupancy::new());
            }

            if let Some(occupancy) = &mut entity.shelter_occupancy {
                if dangerous_sunlight && !occupancy.is_in_shelter() {
                    if !occupancy.seeking_shelter
                        || current_time - occupancy.last_shelter_search > 2.0
                    {
                        occupancy.start_seeking(current_time);
                        shelter_requests.push((entity.id, entity.position));
                    }
                } else if !dangerous_sunlight && occupancy.seeking_shelter {
                    occupancy.stop_seeking();
                }
            }
        }

        // Second pass: fulfill shelter requests
        for (entity_id, entity_pos) in shelter_requests {
            if let Some(shelter_id) =
                Self::find_nearest_available_shelter(entities, entity_pos, 200.0)
            {
                // Try to assign this entity to the shelter
                if let Some(shelter_entity) = entities.iter_mut().find(|e| e.id == shelter_id) {
                    if let Some(shelter) = &mut shelter_entity.shelter {
                        if shelter.add_occupant(entity_id) {
                            // Update entity's occupancy
                            if let Some(entity) = entities.iter_mut().find(|e| e.id == entity_id) {
                                if let Some(occupancy) = &mut entity.shelter_occupancy {
                                    occupancy.enter_shelter(shelter_id, current_time);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Apply protection effects to entities in shelters
    fn apply_shelter_protection(entities: &mut [GameEntity], _sunlight_intensity: f32) {
        for entity in entities {
            if let Some(occupancy) = &entity.shelter_occupancy {
                if occupancy.is_in_shelter() {
                    // Entity is in shelter - they get protection
                    // This will be used by the blood system to reduce sunlight damage
                    continue;
                }
            }

            // Entity is not in shelter - they are exposed to full sunlight
            // The blood system will handle the actual damage calculation
        }
    }

    /// Clean up invalid shelter occupancy references
    fn cleanup_occupancy_references(entities: &mut [GameEntity]) {
        let shelter_ids: std::collections::HashSet<u32> = entities
            .iter()
            .filter_map(|e| e.shelter.as_ref().map(|_| e.id))
            .collect();

        for entity in entities {
            if let Some(occupancy) = &mut entity.shelter_occupancy {
                if let Some(shelter_id) = occupancy.shelter_id {
                    if !shelter_ids.contains(&shelter_id) {
                        // Shelter no longer exists, remove occupancy
                        occupancy.leave_shelter();
                    }
                }
            }
        }
    }

    /// Render all shelters with pixel art style
    pub fn render_shelters(
        entities: &[GameEntity],
        camera_offset_x: f32,
        camera_offset_y: f32,
        zoom_level: f32,
        show_debug_info: bool,
    ) {
        for entity in entities {
            if let Some(shelter) = &entity.shelter {
                Self::render_shelter(
                    entity,
                    shelter,
                    camera_offset_x,
                    camera_offset_y,
                    zoom_level,
                    show_debug_info,
                );
            }
        }
    }

    /// Render a single shelter with pixel art style
    fn render_shelter(
        entity: &GameEntity,
        shelter: &Shelter,
        camera_offset_x: f32,
        camera_offset_y: f32,
        zoom_level: f32,
        show_debug_info: bool,
    ) {
        let screen_x = entity.position.x * zoom_level + camera_offset_x;
        let screen_y = entity.position.y * zoom_level + camera_offset_y;
        let (width, height) = shelter.shelter_type.visual_size();
        let scaled_width = width * zoom_level;
        let scaled_height = height * zoom_level;

        // Draw main shelter structure based on type
        match shelter.shelter_type {
            ShelterType::Cave => {
                Self::draw_cave(screen_x, screen_y, scaled_width, scaled_height, shelter)
            }
            ShelterType::Building => {
                Self::draw_building(screen_x, screen_y, scaled_width, scaled_height, shelter)
            }
            ShelterType::TreeCover => {
                Self::draw_tree_cover(screen_x, screen_y, scaled_width, scaled_height, shelter)
            }
            ShelterType::Underground => {
                Self::draw_underground(screen_x, screen_y, scaled_width, scaled_height, shelter)
            }
            ShelterType::Ruins => {
                Self::draw_ruins(screen_x, screen_y, scaled_width, scaled_height, shelter)
            }
            ShelterType::Shed => {
                Self::draw_shed(screen_x, screen_y, scaled_width, scaled_height, shelter)
            }
            ShelterType::BridgeUnderpass => Self::draw_bridge_underpass(
                screen_x,
                screen_y,
                scaled_width,
                scaled_height,
                shelter,
            ),
        }

        // Draw status indicators
        if shelter.discovered {
            Self::draw_status_indicators(screen_x, screen_y, scaled_width, scaled_height, shelter);
        }

        // Draw debug information if enabled
        if show_debug_info && shelter.discovered {
            Self::draw_debug_info(screen_x, screen_y, scaled_height, shelter);
        }
    }

    /// Draw a cave shelter
    fn draw_cave(screen_x: f32, screen_y: f32, width: f32, height: f32, shelter: &Shelter) {
        let primary = shelter.shelter_type.primary_color();
        let secondary = shelter.shelter_type.secondary_color();

        // Cave opening (dark interior)
        draw_rectangle(
            screen_x - width / 2.0,
            screen_y - height / 2.0,
            width,
            height,
            secondary,
        );

        // Cave entrance arch
        draw_circle(screen_x, screen_y - height / 4.0, width / 3.0, primary);
        draw_circle(screen_x, screen_y - height / 4.0, width / 4.0, secondary);

        // Rocky details
        (0..5).for_each(|i| {
            let offset_x = (i as f32 - 2.0) * width / 8.0;
            let offset_y = rand::gen_range(-height / 3.0, height / 3.0);
            draw_circle(screen_x + offset_x, screen_y + offset_y, 3.0, primary);
        });
    }

    /// Draw a building shelter
    fn draw_building(screen_x: f32, screen_y: f32, width: f32, height: f32, shelter: &Shelter) {
        let primary = shelter.shelter_type.primary_color();
        let secondary = shelter.shelter_type.secondary_color();

        // Main building structure
        draw_rectangle(
            screen_x - width / 2.0,
            screen_y - height / 2.0,
            width,
            height,
            primary,
        );

        // Roof
        draw_triangle(
            Vec2::new(screen_x - width / 2.0, screen_y - height / 2.0),
            Vec2::new(screen_x + width / 2.0, screen_y - height / 2.0),
            Vec2::new(screen_x, screen_y - height),
            secondary,
        );

        // Door
        draw_rectangle(
            screen_x - width / 8.0,
            screen_y + height / 4.0,
            width / 4.0,
            height / 4.0,
            DARKBROWN,
        );

        // Windows
        draw_rectangle(
            screen_x - width / 3.0,
            screen_y - height / 6.0,
            width / 6.0,
            width / 6.0,
            DARKBLUE,
        );
        draw_rectangle(
            screen_x + width / 6.0,
            screen_y - height / 6.0,
            width / 6.0,
            width / 6.0,
            DARKBLUE,
        );
    }

    /// Draw tree cover shelter
    fn draw_tree_cover(screen_x: f32, screen_y: f32, width: f32, height: f32, shelter: &Shelter) {
        let primary = shelter.shelter_type.primary_color();
        let secondary = shelter.shelter_type.secondary_color();

        // Tree trunks
        (0..3).for_each(|i| {
            let x_offset = (i as f32 - 1.0) * width / 3.0;
            draw_rectangle(screen_x + x_offset - 4.0, screen_y, 8.0, height, secondary);
        });

        // Tree canopy (overlapping circles for density)
        (0..5).for_each(|i| {
            let x_offset = (i as f32 - 2.0) * width / 6.0;
            let y_offset = rand::gen_range(-height / 4.0, 0.0);
            draw_circle(
                screen_x + x_offset,
                screen_y - height / 3.0 + y_offset,
                width / 4.0,
                primary,
            );
        });

        // Denser inner foliage
        draw_circle(screen_x, screen_y - height / 3.0, width / 3.0, secondary);
    }

    /// Draw underground shelter
    fn draw_underground(screen_x: f32, screen_y: f32, width: f32, height: f32, shelter: &Shelter) {
        let primary = shelter.shelter_type.primary_color();
        let secondary = shelter.shelter_type.secondary_color();

        // Hatch/entrance
        draw_rectangle(
            screen_x - width / 2.0,
            screen_y - height / 2.0,
            width,
            height,
            primary,
        );

        // Metal grating pattern
        (0..4).for_each(|i| {
            let line_y = screen_y - height / 2.0 + (i as f32 + 1.0) * height / 5.0;
            draw_line(
                screen_x - width / 2.0,
                line_y,
                screen_x + width / 2.0,
                line_y,
                2.0,
                secondary,
            );
        });
        (0..3).for_each(|i| {
            let line_x = screen_x - width / 2.0 + (i as f32 + 1.0) * width / 4.0;
            draw_line(
                line_x,
                screen_y - height / 2.0,
                line_x,
                screen_y + height / 2.0,
                2.0,
                secondary,
            );
        });

        // Ladder indication
        draw_rectangle(screen_x - 3.0, screen_y, 6.0, height + 10.0, DARKGRAY);
    }

    /// Draw ruins shelter
    fn draw_ruins(screen_x: f32, screen_y: f32, width: f32, height: f32, shelter: &Shelter) {
        let primary = shelter.shelter_type.primary_color();
        let secondary = shelter.shelter_type.secondary_color();

        // Broken walls
        draw_rectangle(
            screen_x - width / 2.0,
            screen_y - height / 2.0,
            width / 3.0,
            height,
            primary,
        );
        draw_rectangle(
            screen_x + width / 6.0,
            screen_y - height / 3.0,
            width / 3.0,
            height * 2.0 / 3.0,
            primary,
        );

        // Rubble and debris
        (0..8).for_each(|_| {
            let debris_x = screen_x + rand::gen_range(-width / 2.0, width / 2.0);
            let debris_y = screen_y + rand::gen_range(-height / 2.0, height / 2.0);
            draw_circle(debris_x, debris_y, rand::gen_range(2.0, 6.0), secondary);
        });

        // Archway (partially collapsed)
        draw_circle(screen_x, screen_y - height / 4.0, width / 4.0, primary);
        draw_rectangle(
            screen_x - width / 4.0,
            screen_y - height / 4.0,
            width / 2.0,
            height / 2.0,
            primary,
        );
    }

    /// Draw shed shelter
    fn draw_shed(screen_x: f32, screen_y: f32, width: f32, height: f32, shelter: &Shelter) {
        let primary = shelter.shelter_type.primary_color();
        let secondary = shelter.shelter_type.secondary_color();

        // Main shed structure
        draw_rectangle(
            screen_x - width / 2.0,
            screen_y - height / 2.0,
            width,
            height,
            primary,
        );

        // Slanted roof
        draw_triangle(
            Vec2::new(screen_x - width / 2.0, screen_y - height / 2.0),
            Vec2::new(screen_x + width / 2.0, screen_y - height / 2.0),
            Vec2::new(screen_x + width / 2.0, screen_y - height),
            secondary,
        );

        // Door
        draw_rectangle(
            screen_x - width / 6.0,
            screen_y + height / 6.0,
            width / 3.0,
            height / 3.0,
            DARKBROWN,
        );

        // Small window
        draw_rectangle(
            screen_x + width / 4.0,
            screen_y - height / 6.0,
            width / 8.0,
            width / 8.0,
            SKYBLUE,
        );
    }

    /// Draw bridge underpass shelter
    fn draw_bridge_underpass(
        screen_x: f32,
        screen_y: f32,
        width: f32,
        height: f32,
        shelter: &Shelter,
    ) {
        let primary = shelter.shelter_type.primary_color();
        let secondary = shelter.shelter_type.secondary_color();

        // Bridge structure above
        draw_rectangle(
            screen_x - width / 2.0,
            screen_y - height,
            width,
            height / 3.0,
            secondary,
        );

        // Support pillars
        draw_rectangle(
            screen_x - width / 3.0,
            screen_y - height,
            8.0,
            height * 1.5,
            primary,
        );
        draw_rectangle(
            screen_x + width / 3.0,
            screen_y - height,
            8.0,
            height * 1.5,
            primary,
        );

        // Underpass area (shadowed)
        draw_rectangle(
            screen_x - width / 2.0,
            screen_y - height / 2.0,
            width,
            height / 2.0,
            Color::new(0.2, 0.2, 0.2, 0.8),
        );
    }

    /// Draw status indicators for shelters
    fn draw_status_indicators(
        screen_x: f32,
        screen_y: f32,
        width: f32,
        height: f32,
        shelter: &Shelter,
    ) {
        // Protection level indicator
        let protection = shelter.effective_protection();
        let indicator_color = if protection > 0.8 {
            GREEN
        } else if protection > 0.5 {
            YELLOW
        } else {
            RED
        };

        // Protection level bar
        let bar_width = width * 0.8;
        let bar_height = 4.0;
        let bar_x = screen_x - bar_width / 2.0;
        let bar_y = screen_y + height / 2.0 + 5.0;

        draw_rectangle(bar_x, bar_y, bar_width, bar_height, DARKGRAY);
        draw_rectangle(
            bar_x,
            bar_y,
            bar_width * protection,
            bar_height,
            indicator_color,
        );

        // Occupancy indicator
        if shelter.occupied {
            draw_circle(
                screen_x + width / 2.0 - 8.0,
                screen_y - height / 2.0 + 8.0,
                4.0,
                BLUE,
            );
        }

        // Condition indicator
        let condition_color = shelter.condition.status_color();
        draw_circle(
            screen_x - width / 2.0 + 8.0,
            screen_y - height / 2.0 + 8.0,
            3.0,
            condition_color,
        );
    }

    /// Draw debug information for shelters
    fn draw_debug_info(screen_x: f32, screen_y: f32, height: f32, shelter: &Shelter) {
        let debug_y = screen_y + height / 2.0 + 20.0;

        let protection_text = format!("{}%", (shelter.effective_protection() * 100.0) as u32);
        draw_text(&protection_text, screen_x - 15.0, debug_y, 16.0, WHITE);

        let occupancy_text = format!(
            "{}/{}",
            shelter.occupant_count(),
            shelter.shelter_type.max_capacity()
        );
        draw_text(
            &occupancy_text,
            screen_x - 10.0,
            debug_y + 15.0,
            14.0,
            LIGHTGRAY,
        );
    }
}

/// Information about a shelter for UI display
#[derive(Debug, Clone)]
pub struct ShelterInfo {
    pub id: u32,
    pub position: Position,
    pub shelter_type: ShelterType,
    pub condition: ShelterCondition,
    pub protection_level: f32,
    pub occupancy: String,
    pub distance: f32,
    pub discovered: bool,
    pub name: Option<String>,
}

impl ShelterInfo {
    /// Get a formatted description for UI display
    pub fn get_description(&self) -> String {
        let name = self
            .name
            .as_deref()
            .unwrap_or(self.shelter_type.display_name());
        let protection_pct = (self.protection_level * 100.0) as u32;

        format!(
            "{} - {}% protection, {} occupants, {:.0}m away",
            name, protection_pct, self.occupancy, self.distance
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shelter_spawning() {
        let mut entities = Vec::new();
        let mut next_id = 0;

        let shelter_id = ShelterSystem::spawn_shelter(
            &mut entities,
            &mut next_id,
            ShelterType::Cave,
            100.0,
            200.0,
            None,
            Some("Test Cave".to_string()),
        );

        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].id, shelter_id);
        assert!(entities[0].shelter.is_some());

        let shelter = entities[0].shelter.as_ref().unwrap();
        assert_eq!(shelter.shelter_type, ShelterType::Cave);
        assert_eq!(shelter.name, Some("Test Cave".to_string()));
    }

    #[test]
    fn test_shelter_protection_calculation() {
        let mut entities = Vec::new();
        let mut next_id = 0;

        // Create a shelter and an entity
        let shelter_id = ShelterSystem::spawn_shelter(
            &mut entities,
            &mut next_id,
            ShelterType::Cave,
            0.0,
            0.0,
            None,
            None,
        );

        let entity_id = next_id;
        next_id += 1;

        let mut entity = GameEntity {
            id: entity_id,
            position: Position { x: 0.0, y: 0.0 },
            velocity: Some(Velocity { x: 0.0, y: 0.0 }),
            entity_type: EntityType::Player,
            health: Some(Health {
                current: 100.0,
                max: 100.0,
            }),
            combat_stats: None,
            ai_state: AIState::Idle,
            blood_meter: None,
            vampire_abilities: None,
            shelter: None,
            shelter_occupancy: Some(ShelterOccupancy::new()),
            color: RED,
        };

        entities.push(entity);

        // Test without shelter
        let damage_no_shelter =
            ShelterSystem::calculate_shelter_protection(&entities, entity_id, 100.0);
        assert_eq!(damage_no_shelter, 100.0);

        // Enter shelter
        if let Some(shelter) = &mut entities[0].shelter {
            shelter.add_occupant(entity_id);
        }
        if let Some(occupancy) = &mut entities[1].shelter_occupancy {
            occupancy.enter_shelter(shelter_id, 0.0);
        }

        // Test with shelter
        let damage_with_shelter =
            ShelterSystem::calculate_shelter_protection(&entities, entity_id, 100.0);
        assert!(damage_with_shelter < 100.0);
    }

    #[test]
    fn test_find_nearest_shelter() {
        let mut entities = Vec::new();
        let mut next_id = 0;

        // Create two shelters at different distances
        ShelterSystem::spawn_shelter(
            &mut entities,
            &mut next_id,
            ShelterType::Cave,
            50.0,
            0.0,
            None,
            None,
        );
        ShelterSystem::spawn_shelter(
            &mut entities,
            &mut next_id,
            ShelterType::Shed,
            200.0,
            0.0,
            None,
            None,
        );

        let search_pos = Position { x: 0.0, y: 0.0 };

        // Should find the closer cave
        let nearest = ShelterSystem::find_nearest_available_shelter(&entities, search_pos, 300.0);
        assert!(nearest.is_some());

        // The first shelter (cave) should be closer
        let nearest_entity = entities.iter().find(|e| e.id == nearest.unwrap()).unwrap();
        assert_eq!(
            nearest_entity.shelter.as_ref().unwrap().shelter_type,
            ShelterType::Cave
        );
    }

    #[test]
    fn test_shelter_info_generation() {
        let mut entities = Vec::new();
        let mut next_id = 0;

        // Create player
        let player_id = next_id;
        next_id += 1;
        let player = GameEntity {
            id: player_id,
            position: Position { x: 0.0, y: 0.0 },
            velocity: Some(Velocity { x: 0.0, y: 0.0 }),
            entity_type: EntityType::Player,
            health: Some(Health {
                current: 100.0,
                max: 100.0,
            }),
            combat_stats: None,
            ai_state: AIState::Idle,
            blood_meter: None,
            vampire_abilities: None,
            shelter: None,
            shelter_occupancy: None,
            color: RED,
        };
        entities.push(player);

        // Create discovered shelter
        let shelter_id = ShelterSystem::spawn_shelter(
            &mut entities,
            &mut next_id,
            ShelterType::Building,
            30.0,
            40.0,
            None,
            Some("Town Hall".to_string()),
        );

        // Mark as discovered
        if let Some(shelter_entity) = entities.iter_mut().find(|e| e.id == shelter_id) {
            if let Some(shelter) = &mut shelter_entity.shelter {
                shelter.discover();
            }
        }

        let shelter_info = ShelterSystem::get_nearby_shelter_info(&entities, player_id, 100.0);

        assert_eq!(shelter_info.len(), 1);
        assert_eq!(shelter_info[0].name, Some("Town Hall".to_string()));
        assert!(shelter_info[0].discovered);
    }
}
