//! World System Module
//!
//! Handles world initialization, entity spawning, and world setup utilities.
//! This system is responsible for creating the initial game world state.

use crate::components::*;
use macroquad::prelude::*;
use std::collections::HashMap;

/// Game world dimension constants
pub const GAME_WORLD_WIDTH: f32 = 2560.0;
pub const GAME_WORLD_HEIGHT: f32 = 1440.0;
pub const GROUND_LEVEL: f32 = 640.0;

/// Horizon behavior constants (preserves current visuals)
pub const HORIZON_LINE: f32 = 640.0; // Same as ground level - no visual change
pub const HORIZON_MOVEMENT_THRESHOLD: f32 = 1.0; // Minimum movement toward horizon to trigger effect
pub const GROUND_SHIFT_DISTANCE: f32 = 200.0; // How far to shift ground tiles during horizon movement
pub const HORIZON_UPDATE_RATE: f32 = 0.033; // Update every ~30fps for responsiveness

/// Fog of war constants
pub const FOG_TILE_SIZE: f32 = 128.0; // Larger tiles for better performance
pub const FOG_COLOR: [f32; 4] = [0.7, 0.7, 0.7, 0.6]; // Light grey with transparency
pub const FOG_EDGE_FADE: f32 = 32.0; // Distance for fog edge fading
pub const FOG_MIN_ALPHA: f32 = 0.3; // Minimum fog transparency
pub const FOG_EXPANSION_RADIUS: f32 = 128.0; // How far to expand fog beyond visible area
pub const TILE_SIZE: f32 = 64.0; // Standard tile size for consistent alignment
pub const FOG_BUFFER_TILES: i32 = 2; // Extra fog tiles around explored areas to prevent gaps

/// World system responsible for entity spawning and world management
pub struct WorldSystem;

impl WorldSystem {
    /// Snap a coordinate to the nearest tile grid position
    pub fn snap_to_grid(coord: f32) -> f32 {
        (coord / TILE_SIZE).floor() * TILE_SIZE
    }

    /// Snap coordinates to tile grid and return grid-aligned position
    pub fn snap_position_to_grid(x: f32, y: f32) -> (f32, f32) {
        (Self::snap_to_grid(x), Self::snap_to_grid(y))
    }
}

impl WorldSystem {
    /// Initialize the game world with all starting entities and environment
    pub fn initialize_world(
        entities: &mut Vec<GameEntity>,
        clans: &mut HashMap<String, Clan>,
        stars: &mut Vec<Star>,
        moon: &mut Moon,
        ground_tiles: &mut Vec<GroundTile>,
        next_entity_id: &mut u32,
        debug_messages: &mut Vec<String>,
    ) -> u32 {
        // Clear existing entities
        entities.clear();

        // Create the player entity
        let player_id = Self::spawn_player(entities, next_entity_id);

        // Initialize clans
        Self::initialize_clans(clans);

        // Spawn clan leaders
        Self::spawn_all_clan_leaders(entities, next_entity_id);

        // Spawn fewer initial entities to prevent early FPS drops
        Self::spawn_hostile_infected_group(entities, next_entity_id, 6); // Reduced from 8
        Self::spawn_animal_group(entities, next_entity_id, 8); // Reduced from 12

        // Spawn shelters throughout the world
        Self::spawn_world_shelters(entities, next_entity_id, debug_messages);

        // Initialize environment
        Self::initialize_starfield(stars);
        Self::initialize_moon(moon);
        Self::initialize_ground_terrain(ground_tiles);

        player_id
    }

    /// Create the player entity
    pub fn spawn_player(entities: &mut Vec<GameEntity>, next_entity_id: &mut u32) -> u32 {
        let player_id = *next_entity_id;
        let player = GameEntity {
            id: player_id,
            position: Position { x: 400.0, y: 650.0 },
            velocity: Some(Velocity { x: 0.0, y: 0.0 }),
            entity_type: EntityType::Player,
            health: Some(Health {
                current: 100.0,
                max: 100.0,
            }),
            combat_stats: Some(CombatStats::new(25.0, 10.0)),
            ai_state: AIState::Idle,
            blood_meter: Some(BloodMeter {
                current: 50.0,
                maximum: 100.0,
                drain_rate: 1.0,
            }),
            vampire_abilities: Some(VampireAbilities {
                strength: 1.0,
                speed: 1.0,
                blood_sense: 0.0,
                shadow_movement: 0.0,
            }),
            shelter: None,
            shelter_occupancy: Some(ShelterOccupancy::new()),
            color: RED,
        };

        entities.push(player);
        *next_entity_id += 1;
        player_id
    }

    /// Initialize all clans in the game
    pub fn initialize_clans(clans: &mut HashMap<String, Clan>) {
        clans.clear();

        clans.insert(
            "Bone-Eaters".to_string(),
            Clan::new("Bone-Eaters", "Grimjaw", 15),
        );
        clans.insert(
            "Flame-Haters".to_string(),
            Clan::new("Flame-Haters", "Shadowmere", 12),
        );
        clans.insert(
            "Night-Bloods".to_string(),
            Clan::new("Night-Bloods", "Silentfang", 10),
        );
    }

    /// Spawn all clan leaders at their designated positions
    pub fn spawn_all_clan_leaders(entities: &mut Vec<GameEntity>, next_entity_id: &mut u32) {
        Self::spawn_clan_leader(
            entities,
            next_entity_id,
            "Grimjaw",
            "Bone-Eaters",
            200.0,
            650.0,
            BEIGE,
        );
        Self::spawn_clan_leader(
            entities,
            next_entity_id,
            "Shadowmere",
            "Flame-Haters",
            600.0,
            650.0,
            PURPLE,
        );
        Self::spawn_clan_leader(
            entities,
            next_entity_id,
            "Silentfang",
            "Night-Bloods",
            800.0,
            650.0,
            DARKBLUE,
        );
    }

    /// Spawn a single clan leader
    pub fn spawn_clan_leader(
        entities: &mut Vec<GameEntity>,
        next_entity_id: &mut u32,
        _name: &str,
        clan_name: &str,
        x: f32,
        y: f32,
        color: Color,
    ) -> u32 {
        // Validate ground position
        if !Self::has_ground_at_position(x, y) {
            eprintln!(
                "Warning: Clan leader '{}' spawning above ground at ({}, {}) - adjusting position",
                _name, x, y
            );
            // Find a safe position using spawn bounds
            if let Some((safe_x, safe_y)) = Self::find_safe_spawn_position(
                entities,
                &EntityType::ClanLeader(clan_name.to_string()),
                30.0,
                10,
            ) {
                return Self::spawn_clan_leader_at_position(
                    entities,
                    next_entity_id,
                    _name,
                    clan_name,
                    safe_x,
                    safe_y,
                    color,
                );
            } else {
                // Fallback to minimum ground level
                let safe_y = 650.0; // Ground level + padding
                eprintln!("Using fallback ground position: ({}, {})", x, safe_y);
                return Self::spawn_clan_leader_at_position(
                    entities,
                    next_entity_id,
                    _name,
                    clan_name,
                    x,
                    safe_y,
                    color,
                );
            }
        }

        Self::spawn_clan_leader_at_position(entities, next_entity_id, _name, clan_name, x, y, color)
    }

    /// Internal function to spawn clan leader at verified position
    fn spawn_clan_leader_at_position(
        entities: &mut Vec<GameEntity>,
        next_entity_id: &mut u32,
        _name: &str,
        clan_name: &str,
        x: f32,
        y: f32,
        color: Color,
    ) -> u32 {
        let entity_id = *next_entity_id;
        let entity = GameEntity {
            id: entity_id,
            position: Position { x, y },
            velocity: Some(Velocity { x: 0.0, y: 0.0 }),
            entity_type: EntityType::ClanLeader(clan_name.to_string()),
            health: Some(Health {
                current: 120.0,
                max: 120.0,
            }),
            combat_stats: Some(CombatStats::new(30.0, 15.0)),
            ai_state: AIState::Idle,
            blood_meter: None,
            vampire_abilities: None,
            shelter: None,
            shelter_occupancy: None,
            color,
        };

        entities.push(entity);
        *next_entity_id += 1;
        entity_id
    }

    /// Spawn a group of hostile infected creatures
    pub fn spawn_hostile_infected_group(
        entities: &mut Vec<GameEntity>,
        next_entity_id: &mut u32,
        count: usize,
    ) {
        let max_entities = 100; // Entity limit to prevent performance issues

        (0..count).for_each(|_| {
            // Check entity limit before spawning
            if entities.len() >= max_entities {
                return;
            }

            let (min_x, max_x, min_y, max_y) = Self::get_spawn_bounds(&EntityType::HostileInfected);
            let x = rand::gen_range(min_x, max_x);
            let y = rand::gen_range(min_y, max_y);
            Self::spawn_hostile_infected(entities, next_entity_id, x, y);
        });
    }

    /// Spawn a single hostile infected creature
    pub fn spawn_hostile_infected(
        entities: &mut Vec<GameEntity>,
        next_entity_id: &mut u32,
        x: f32,
        y: f32,
    ) -> u32 {
        let entity_id = *next_entity_id;
        let entity = GameEntity {
            id: entity_id,
            position: Position { x, y },
            velocity: Some(Velocity { x: 0.0, y: 0.0 }),
            entity_type: EntityType::HostileInfected,
            health: Some(Health {
                current: 50.0,
                max: 50.0,
            }),
            combat_stats: Some(CombatStats::new(20.0, 8.0)),
            ai_state: AIState::Hostile,
            blood_meter: None,
            vampire_abilities: None,
            shelter: None,
            shelter_occupancy: None,
            color: DARKGREEN,
        };

        entities.push(entity);
        *next_entity_id += 1;
        entity_id
    }

    /// Spawn a group of animals
    /// Spawn a group of animals for blood sources
    pub fn spawn_animal_group(
        entities: &mut Vec<GameEntity>,
        next_entity_id: &mut u32,
        count: usize,
    ) {
        let max_entities = 100; // Entity limit to prevent performance issues

        (0..count).for_each(|_| {
            // Check entity limit before spawning
            if entities.len() >= max_entities {
                return;
            }

            let (min_x, max_x, min_y, max_y) = Self::get_spawn_bounds(&EntityType::Animal);
            let x = rand::gen_range(min_x, max_x);
            let y = rand::gen_range(min_y, max_y);
            Self::spawn_animal(entities, next_entity_id, x, y);
        });
    }

    /// Spawn a single animal
    pub fn spawn_animal(
        entities: &mut Vec<GameEntity>,
        next_entity_id: &mut u32,
        x: f32,
        y: f32,
    ) -> u32 {
        let entity_id = *next_entity_id;
        let entity = GameEntity {
            id: entity_id,
            position: Position { x, y },
            velocity: Some(Velocity { x: 0.0, y: 0.0 }),
            entity_type: EntityType::Animal,
            health: Some(Health {
                current: 25.0,
                max: 25.0,
            }),
            combat_stats: None,
            ai_state: AIState::Idle,
            blood_meter: None,
            vampire_abilities: None,
            shelter: None,
            shelter_occupancy: None,
            color: BROWN,
        };

        entities.push(entity);
        *next_entity_id += 1;
        entity_id
    }

    /// Initialize the starfield background
    pub fn initialize_starfield(stars: &mut Vec<Star>) {
        stars.clear();
        (0..300).for_each(|_| {
            let x = rand::gen_range(0.0, GAME_WORLD_WIDTH);
            let y = rand::gen_range(0.0, GAME_WORLD_HEIGHT);
            stars.push(Star::new(x, y));
        });
    }

    /// Initialize the moon
    pub fn initialize_moon(moon: &mut Moon) {
        *moon = Moon::new();
    }

    /// Initialize ground terrain tiles with proper grid alignment
    pub fn initialize_ground_terrain(ground_tiles: &mut Vec<GroundTile>) {
        ground_tiles.clear();

        // Generate initial ground around the player spawn area (center of world)
        let center_x = GAME_WORLD_WIDTH / 2.0;
        let center_y = GROUND_LEVEL;

        // Generate tiles in a reasonable area around the spawn point, grid-aligned
        let initial_area_size = 800.0;
        let start_x = Self::snap_to_grid(center_x - initial_area_size);
        let end_x = Self::snap_to_grid(center_x + initial_area_size);

        // Generate from horizon down to well below ground level, grid-aligned
        let start_y = Self::snap_to_grid(HORIZON_LINE);
        let end_y = Self::snap_to_grid(center_y + initial_area_size);

        let mut x = start_x;
        while x <= end_x {
            let mut y = start_y;
            while y <= end_y {
                if y >= HORIZON_LINE {
                    let tile_type = Self::determine_tile_type();
                    ground_tiles.push(GroundTile::new(x, y, tile_type));
                }
                y += TILE_SIZE;
            }
            x += TILE_SIZE;
        }
    }

    /// Determine the type of tile to place based on random generation
    fn determine_tile_type() -> TileType {
        match rand::gen_range(0, 100) {
            0..=60 => TileType::Grass,
            61..=80 => TileType::DeadGrass,
            81..=95 => TileType::Dirt,
            _ => TileType::Stone,
        }
    }

    /// Shift ground tiles during horizon movement (preserves current visuals)
    pub fn shift_ground_for_horizon_movement(
        ground_tiles: &mut Vec<GroundTile>,
        player_x: f32,
        camera_y: f32,
        movement_distance: f32,
        debug_messages: &mut Vec<String>,
    ) {
        let initial_tile_count = ground_tiles.len();

        // Enhanced screen-based bounds for infinite walking
        let screen_width = 1280.0;
        let screen_height = 720.0;

        // Expand generation area for seamless infinite walking
        let generation_x_range = screen_width * 2.0; // Wider coverage
        let generation_y_range = screen_height * 1.5; // Taller coverage

        // Only shift tiles if movement is significant enough
        if movement_distance.abs() > 0.5 {
            // Shift existing ground tiles downward to simulate movement
            for tile in ground_tiles.iter_mut() {
                tile.y += movement_distance;
            }
        }

        // Generate new ground tiles ahead of the player
        let tiles_to_generate_x = (generation_x_range as f32 / TILE_SIZE).ceil() as i32;
        let start_x = Self::snap_to_grid(player_x - generation_x_range / 2.0);

        let mut tiles_added = 0;

        // Generate tiles at and below the horizon line
        for i in 0..tiles_to_generate_x {
            let x = start_x + (i as f32 * TILE_SIZE);

            // Generate multiple rows starting from horizon
            for row in 0..8 {
                let y = Self::snap_to_grid(HORIZON_LINE) + (row as f32 * TILE_SIZE);

                // Check if we already have a tile at this position
                let tile_exists = ground_tiles.iter().any(|tile| {
                    (tile.x - x).abs() < TILE_SIZE * 0.1 && (tile.y - y).abs() < TILE_SIZE * 0.1
                });

                if !tile_exists {
                    let tile_type = Self::determine_tile_type();
                    ground_tiles.push(GroundTile::new(x, y, tile_type));
                    tiles_added += 1;
                }
            }
        }

        // More aggressive cleanup to prevent performance issues
        let cleanup_x_range = generation_x_range * 1.5;
        let cleanup_y_range = generation_y_range * 2.0;

        let before_cleanup = ground_tiles.len();
        ground_tiles.retain(|tile| {
            let x_in_range = (tile.x - player_x).abs() < cleanup_x_range;
            let y_relative_to_camera = tile.y - camera_y;

            // Keep tiles that are visible or just off-screen
            let y_in_range =
                y_relative_to_camera > -cleanup_y_range && y_relative_to_camera < cleanup_y_range;

            // Don't keep tiles too far above the horizon
            let not_too_far_up = tile.y >= HORIZON_LINE - TILE_SIZE;

            x_in_range && y_in_range && not_too_far_up
        });
        let after_cleanup = ground_tiles.len();
        let tiles_removed = before_cleanup - after_cleanup;

        // Ensure grid alignment after cleanup and generation
        for tile in ground_tiles.iter_mut() {
            let (aligned_x, aligned_y) = Self::snap_position_to_grid(tile.x, tile.y);
            tile.x = aligned_x;
            tile.y = aligned_y;
        }

        // Add debug information for significant changes
        if tiles_added > 0 || tiles_removed > 0 {
            debug_messages.push(format!(
                "Horizon tiles: {} → {} (+{}, -{}) at player({:.0}, cam_y:{:.0})",
                initial_tile_count,
                ground_tiles.len(),
                tiles_added,
                tiles_removed,
                player_x,
                camera_y
            ));
        }

        // Debug tile generation activity
        if movement_distance.abs() > 0.5 {
            debug_messages.push(format!(
                "Horizon shift: {:.1} units, infinite walking active",
                movement_distance
            ));
        }
    }

    /// Remove fog areas that overlap with newly spawned ground tiles
    pub fn remove_overlapping_fog(
        fog_areas: &mut Vec<(f32, f32, f32, f32, f32)>,
        ground_tiles: &[GroundTile],
    ) -> usize {
        let initial_count = fog_areas.len();
        let _tile_size = 64.0;

        fog_areas.retain(|(fog_x, fog_y, fog_width, fog_height, _alpha)| {
            let fog_right = fog_x + fog_width;
            let fog_bottom = fog_y + fog_height;

            // Keep fog only if it doesn't overlap with any ground tile
            !ground_tiles.iter().any(|tile| {
                let tile_right = tile.x + TILE_SIZE;
                let tile_bottom = tile.y + TILE_SIZE;

                // Check for any overlap
                !(tile_right <= *fog_x
                    || tile.x >= fog_right
                    || tile_bottom <= *fog_y
                    || tile.y >= fog_bottom)
            })
        });

        initial_count - fog_areas.len()
    }

    /// Check if an area has been explored (contains ground tiles) - optimized for no overlaps
    pub fn is_area_explored(ground_tiles: &[GroundTile], fog_x: f32, fog_y: f32) -> bool {
        let _tile_size = TILE_SIZE;

        // Use expanded overlap detection to prevent any fog near ground tiles
        // This ensures fog never overlaps with existing ground
        ground_tiles.iter().any(|tile| {
            let tile_right = tile.x + TILE_SIZE;
            let tile_bottom = tile.y + TILE_SIZE;
            let fog_right = fog_x + FOG_TILE_SIZE;
            let fog_bottom = fog_y + FOG_TILE_SIZE;

            // Check for any overlap between ground tile and fog area
            !(tile_right <= fog_x
                || tile.x >= fog_right
                || tile_bottom <= fog_y
                || tile.y >= fog_bottom)
        })
    }

    /// Calculate fog transparency based on distance to explored areas (optimized for grid alignment)
    pub fn calculate_fog_alpha(
        ground_tiles: &[GroundTile],
        fog_x: f32,
        fog_y: f32,
        _fog_size: f32,
    ) -> f32 {
        let mut min_distance = f32::MAX;

        // Check neighboring grid positions for closest explored tile
        let search_radius = FOG_EDGE_FADE + FOG_TILE_SIZE;

        for tile in ground_tiles.iter() {
            // Quick distance check to avoid expensive calculations
            let dx = tile.x - fog_x;
            let dy = tile.y - fog_y;

            if dx.abs() <= search_radius && dy.abs() <= search_radius {
                let distance = (dx * dx + dy * dy).sqrt();
                min_distance = min_distance.min(distance);
            }
        }

        // Calculate alpha with smooth transition near explored areas
        if min_distance >= FOG_EDGE_FADE {
            FOG_COLOR[3] // Full fog opacity for distant areas
        } else if min_distance <= FOG_TILE_SIZE {
            FOG_MIN_ALPHA // Minimum opacity near explored areas
        } else {
            // Smooth interpolation between near and far
            let fade_factor = (min_distance - FOG_TILE_SIZE) / (FOG_EDGE_FADE - FOG_TILE_SIZE);
            FOG_MIN_ALPHA + (FOG_COLOR[3] - FOG_MIN_ALPHA) * fade_factor
        }
    }

    /// Generate fog areas for unexplored regions with transparency
    pub fn calculate_fog_areas(
        ground_tiles: &[GroundTile],
        camera_x: f32,
        camera_y: f32,
        screen_width: f32,
        screen_height: f32,
        zoom_level: f32,
    ) -> Vec<(f32, f32, f32, f32, f32)> {
        let mut fog_areas = Vec::new();

        // Expand visible area to ensure complete fog coverage with buffer
        let world_left = camera_x - (screen_width / (2.0 * zoom_level)) - FOG_EXPANSION_RADIUS;
        let world_right = camera_x + (screen_width / (2.0 * zoom_level)) + FOG_EXPANSION_RADIUS;
        let world_top = camera_y - (screen_height / (2.0 * zoom_level)) - FOG_EXPANSION_RADIUS;
        let world_bottom = camera_y + (screen_height / (2.0 * zoom_level)) + FOG_EXPANSION_RADIUS;

        // Align fog grid to tile boundaries to prevent gaps
        let grid_start_x = Self::snap_to_grid(world_left);
        let grid_start_y = Self::snap_to_grid(HORIZON_LINE.max(world_top));

        // Generate fog grid aligned with ground tiles, with extra buffer coverage
        let fog_cols =
            ((world_right - grid_start_x) / FOG_TILE_SIZE).ceil() as i32 + FOG_BUFFER_TILES;
        let fog_rows =
            ((world_bottom - grid_start_y) / FOG_TILE_SIZE).ceil() as i32 + FOG_BUFFER_TILES;

        for col in 0..fog_cols {
            for row in 0..fog_rows {
                let fog_x = grid_start_x + col as f32 * FOG_TILE_SIZE;
                let fog_y = grid_start_y + row as f32 * FOG_TILE_SIZE;

                // Only add fog if area hasn't been explored and is below horizon
                // Use the already optimized exploration check
                if fog_y >= HORIZON_LINE && !Self::is_area_explored(ground_tiles, fog_x, fog_y) {
                    fog_areas.push((fog_x, fog_y, FOG_TILE_SIZE, FOG_TILE_SIZE, FOG_COLOR[3]));
                }
            }
        }

        // Add gap filling to ensure seamless coverage
        Self::fill_coverage_gaps(
            &mut fog_areas,
            ground_tiles,
            grid_start_x,
            grid_start_y,
            fog_cols,
            fog_rows,
        );

        fog_areas
    }

    /// Fill any gaps between fog and ground to ensure seamless coverage
    fn fill_coverage_gaps(
        fog_areas: &mut Vec<(f32, f32, f32, f32, f32)>,
        ground_tiles: &[GroundTile],
        grid_start_x: f32,
        grid_start_y: f32,
        fog_cols: i32,
        fog_rows: i32,
    ) {
        // Check for gaps around existing ground tiles
        for tile in ground_tiles {
            // Check 8 directions around each ground tile for gaps
            let directions = [
                (-1, -1),
                (-1, 0),
                (-1, 1),
                (0, -1),
                (0, 1),
                (1, -1),
                (1, 0),
                (1, 1),
            ];

            for (dx, dy) in directions.iter() {
                let check_x = Self::snap_to_grid(tile.x + (*dx as f32 * TILE_SIZE));
                let check_y = Self::snap_to_grid(tile.y + (*dy as f32 * TILE_SIZE));

                // Only fill gaps below horizon
                if check_y >= HORIZON_LINE {
                    // Check if this position needs coverage
                    let has_ground = ground_tiles.iter().any(|t| {
                        (t.x - check_x).abs() < TILE_SIZE * 0.5
                            && (t.y - check_y).abs() < TILE_SIZE * 0.5
                    });

                    let has_fog = fog_areas.iter().any(|(fx, fy, _, _, _)| {
                        (fx - check_x).abs() < FOG_TILE_SIZE * 0.5
                            && (fy - check_y).abs() < FOG_TILE_SIZE * 0.5
                    });

                    // Fill gap if neither ground nor fog exists
                    if !has_ground && !has_fog {
                        fog_areas.push((
                            check_x,
                            check_y,
                            FOG_TILE_SIZE,
                            FOG_TILE_SIZE,
                            FOG_COLOR[3],
                        ));
                    }
                }
            }
        }

        // Fill any remaining grid gaps in the fog area
        for col in 0..fog_cols {
            for row in 0..fog_rows {
                let grid_x = grid_start_x + col as f32 * FOG_TILE_SIZE;
                let grid_y = grid_start_y + row as f32 * FOG_TILE_SIZE;

                if grid_y >= HORIZON_LINE {
                    let has_coverage = ground_tiles.iter().any(|tile| {
                        let tile_right = tile.x + TILE_SIZE;
                        let tile_bottom = tile.y + TILE_SIZE;
                        let grid_right = grid_x + FOG_TILE_SIZE;
                        let grid_bottom = grid_y + FOG_TILE_SIZE;

                        !(tile_right <= grid_x
                            || tile.x >= grid_right
                            || tile_bottom <= grid_y
                            || tile.y >= grid_bottom)
                    }) || fog_areas.iter().any(|(fx, fy, _, _, _)| {
                        (fx - grid_x).abs() < FOG_TILE_SIZE * 0.5
                            && (fy - grid_y).abs() < FOG_TILE_SIZE * 0.5
                    });

                    // Add fog if no coverage exists
                    if !has_coverage {
                        fog_areas.push((
                            grid_x,
                            grid_y,
                            FOG_TILE_SIZE,
                            FOG_TILE_SIZE,
                            FOG_COLOR[3],
                        ));
                    }
                }
            }
        }
    }

    /// Check if a position has ground (respects horizon line boundary)
    pub fn has_ground_at_position(_x: f32, y: f32) -> bool {
        // Only allow ground at or below horizon line
        y >= GROUND_LEVEL
    }

    /// Generate a random position within an expanded ground area (below horizon)
    pub fn generate_random_ground_position() -> (f32, f32) {
        // Generate random position in expanded area, but only below horizon
        let x = rand::gen_range(-1000.0, GAME_WORLD_WIDTH + 1000.0); // Extend beyond world bounds
        let y = rand::gen_range(GROUND_LEVEL, GAME_WORLD_HEIGHT * 2.0); // Only below horizon line

        (x, y)
    }

    /// Check if a position is close enough to ground area to be relocated (expanded world)
    pub fn is_relocatable_to_ground(_x: f32, _y: f32) -> bool {
        // In expanded world, allow relocation from anywhere
        true // Always allow relocation in expanded world
    }

    /// DEPRECATED: Legacy ground generation method - now handled by ExplorationSystem
    /// This method is kept for backwards compatibility but does minimal work to prevent
    /// conflicts with the new ExplorationSystem ground generation.
    pub fn ensure_ground_near_player(
        _ground_tiles: &mut Vec<GroundTile>,
        player_x: f32,
        player_y: f32,
        debug_messages: &mut Vec<String>,
    ) {
        // DEPRECATED: This legacy method is now a no-op to prevent conflicts
        // All ground generation is handled by ExplorationSystem to eliminate
        // the "every other column" spawning issue caused by dual systems.

        // Only add a debug message to indicate the method was called
        debug_messages.push(format!(
            "LEGACY: WorldSystem::ensure_ground_near_player called at ({:.0}, {:.0}) - using ExplorationSystem instead",
            player_x, player_y
        ));
    }

    /// DEPRECATED: Legacy directional ground spawning - now handled by ExplorationSystem
    fn spawn_directional_ground(
        _ground_tiles: &mut Vec<GroundTile>,
        player_x: f32,
        player_y: f32,
        debug_messages: &mut Vec<String>,
    ) {
        // DEPRECATED: This method is now a no-op to prevent dual-system conflicts
        // ExplorationSystem handles all directional ground generation
        debug_messages.push(format!(
            "LEGACY: spawn_directional_ground called at ({:.0}, {:.0}) - using ExplorationSystem instead",
            player_x, player_y
        ));
    }

    /// Spawn a clan member at a specific location
    pub fn spawn_clan_member(
        entities: &mut Vec<GameEntity>,
        next_entity_id: &mut u32,
        clan_name: &str,
        x: f32,
        y: f32,
        color: Color,
    ) -> u32 {
        // Validate ground position
        if !Self::has_ground_at_position(x, y) {
            eprintln!(
                "Warning: Clan member spawning above ground at ({}, {}) - adjusting position",
                x, y
            );
            // Find a safe position using spawn bounds
            if let Some((safe_x, safe_y)) = Self::find_safe_spawn_position(
                entities,
                &EntityType::ClanMember(clan_name.to_string()),
                30.0,
                10,
            ) {
                return Self::spawn_clan_member_at_position(
                    entities,
                    next_entity_id,
                    clan_name,
                    safe_x,
                    safe_y,
                    color,
                );
            } else {
                // Fallback to minimum ground level
                let safe_y = 650.0; // Ground level + padding
                eprintln!("Using fallback ground position: ({}, {})", x, safe_y);
                return Self::spawn_clan_member_at_position(
                    entities,
                    next_entity_id,
                    clan_name,
                    x,
                    safe_y,
                    color,
                );
            }
        }

        Self::spawn_clan_member_at_position(entities, next_entity_id, clan_name, x, y, color)
    }

    /// Internal function to spawn clan member at verified position
    fn spawn_clan_member_at_position(
        entities: &mut Vec<GameEntity>,
        next_entity_id: &mut u32,
        clan_name: &str,
        x: f32,
        y: f32,
        color: Color,
    ) -> u32 {
        let entity_id = *next_entity_id;
        let entity = GameEntity {
            id: entity_id,
            position: Position { x, y },
            velocity: Some(Velocity { x: 0.0, y: 0.0 }),
            entity_type: EntityType::ClanMember(clan_name.to_string()),
            health: Some(Health {
                current: 80.0,
                max: 80.0,
            }),
            combat_stats: Some(CombatStats::new(15.0, 5.0)),
            ai_state: AIState::Idle,
            blood_meter: None,
            vampire_abilities: None,
            shelter: None,
            shelter_occupancy: None,
            color,
        };

        entities.push(entity);
        *next_entity_id += 1;
        entity_id
    }

    /// Get spawn bounds for different entity types
    pub fn get_spawn_bounds(entity_type: &EntityType) -> (f32, f32, f32, f32) {
        match entity_type {
            EntityType::Player => (
                -1000.0,
                GAME_WORLD_WIDTH + 1000.0,
                GROUND_LEVEL,
                GAME_WORLD_HEIGHT * 2.0,
            ),
            EntityType::ClanLeader(_) => (
                -1000.0,
                GAME_WORLD_WIDTH + 1000.0,
                GROUND_LEVEL,
                GAME_WORLD_HEIGHT * 2.0,
            ),
            EntityType::ClanMember(_) => (
                -1000.0,
                GAME_WORLD_WIDTH + 1000.0,
                GROUND_LEVEL,
                GAME_WORLD_HEIGHT * 2.0,
            ),
            EntityType::HostileInfected => (
                -1000.0,
                GAME_WORLD_WIDTH + 1000.0,
                GROUND_LEVEL,
                GAME_WORLD_HEIGHT * 2.0,
            ),
            EntityType::Animal => (
                -1000.0,
                GAME_WORLD_WIDTH + 1000.0,
                GROUND_LEVEL,
                GAME_WORLD_HEIGHT * 2.0,
            ),
            EntityType::Shelter => (
                -1000.0,
                GAME_WORLD_WIDTH + 1000.0,
                GROUND_LEVEL,
                GAME_WORLD_HEIGHT * 2.0,
            ),
        }
    }

    /// Check if a position is valid for spawning (not overlapping with other entities)
    pub fn is_valid_spawn_position(
        entities: &[GameEntity],
        x: f32,
        y: f32,
        min_distance: f32,
    ) -> bool {
        for entity in entities {
            let distance =
                ((x - entity.position.x).powi(2) + (y - entity.position.y).powi(2)).sqrt();
            if distance < min_distance {
                return false;
            }
        }
        true
    }

    /// Spawn shelters throughout the world for vampire protection
    fn spawn_world_shelters(
        entities: &mut Vec<GameEntity>,
        next_entity_id: &mut u32,
        debug_messages: &mut Vec<String>,
    ) {
        use crate::components::{ShelterCondition, ShelterType};
        use crate::systems::ShelterSystem;

        // Spawn a variety of shelters across the map
        let shelter_locations = [
            // Caves - high protection, scattered around edges
            (
                200.0,
                150.0,
                ShelterType::Cave,
                Some(ShelterCondition::Good),
                Some("Ancient Cave"),
            ),
            (
                800.0,
                100.0,
                ShelterType::Cave,
                Some(ShelterCondition::Pristine),
                Some("Deep Cavern"),
            ),
            (
                1200.0,
                400.0,
                ShelterType::Cave,
                Some(ShelterCondition::Damaged),
                None,
            ),
            // Buildings - medium protection, more central
            (
                500.0,
                300.0,
                ShelterType::Building,
                Some(ShelterCondition::Good),
                Some("Abandoned House"),
            ),
            (
                700.0,
                500.0,
                ShelterType::Building,
                Some(ShelterCondition::Damaged),
                Some("Old Warehouse"),
            ),
            (
                900.0,
                250.0,
                ShelterType::Building,
                Some(ShelterCondition::Poor),
                None,
            ),
            // Underground bunkers - maximum protection, rare
            (
                350.0,
                650.0,
                ShelterType::Underground,
                Some(ShelterCondition::Pristine),
                Some("Emergency Bunker"),
            ),
            (
                1000.0,
                700.0,
                ShelterType::Underground,
                Some(ShelterCondition::Good),
                None,
            ),
            // Ruins - atmospheric, medium protection
            (
                150.0,
                400.0,
                ShelterType::Ruins,
                Some(ShelterCondition::Damaged),
                Some("Temple Ruins"),
            ),
            (
                650.0,
                200.0,
                ShelterType::Ruins,
                Some(ShelterCondition::Poor),
                Some("Castle Remains"),
            ),
            (
                1100.0,
                650.0,
                ShelterType::Ruins,
                Some(ShelterCondition::Good),
                None,
            ),
            // Sheds - common, lower protection
            (
                450.0,
                450.0,
                ShelterType::Shed,
                Some(ShelterCondition::Good),
                None,
            ),
            (
                750.0,
                350.0,
                ShelterType::Shed,
                Some(ShelterCondition::Damaged),
                None,
            ),
            (
                550.0,
                650.0,
                ShelterType::Shed,
                Some(ShelterCondition::Good),
                None,
            ),
            // Tree cover - temporary protection, natural
            (
                300.0,
                500.0,
                ShelterType::TreeCover,
                Some(ShelterCondition::Good),
                Some("Dense Grove"),
            ),
            (
                850.0,
                450.0,
                ShelterType::TreeCover,
                Some(ShelterCondition::Good),
                None,
            ),
            (
                1150.0,
                200.0,
                ShelterType::TreeCover,
                Some(ShelterCondition::Damaged),
                None,
            ),
            // Bridge underpasses - urban shelter
            (
                600.0,
                400.0,
                ShelterType::BridgeUnderpass,
                Some(ShelterCondition::Good),
                Some("Highway Underpass"),
            ),
            (
                950.0,
                550.0,
                ShelterType::BridgeUnderpass,
                Some(ShelterCondition::Damaged),
                None,
            ),
        ];

        // Spawn shelters with better distribution
        let mut spawned_shelters = Vec::new();

        for (desired_x, desired_y, shelter_type, condition, name) in shelter_locations.iter() {
            let (spawn_x, spawn_y) = if Self::has_ground_at_position(*desired_x, *desired_y) {
                // If already on valid ground, use original position
                (*desired_x, *desired_y)
            } else if Self::is_relocatable_to_ground(*desired_x, *desired_y) {
                // If close to ground area, relocate to a random ground position
                // but avoid clustering by checking against already spawned shelters
                let mut attempts = 0;
                let max_attempts = 10;

                loop {
                    let (candidate_x, candidate_y) = Self::generate_random_ground_position();

                    // Check if this position is too close to existing shelters
                    let min_distance = 120.0; // Minimum distance between shelters
                    let too_close = spawned_shelters.iter().any(|(sx, sy)| {
                        let dx = candidate_x - sx;
                        let dy = candidate_y - sy;
                        (dx * dx + dy * dy).sqrt() < min_distance
                    });

                    if !too_close || attempts >= max_attempts {
                        break (candidate_x, candidate_y);
                    }

                    attempts += 1;
                }
            } else {
                // If too far from ground area, skip this shelter
                debug_messages.push(format!(
                    "Skipping shelter '{}' at ({}, {}) - too far from ground area",
                    name.as_ref().unwrap_or(&"Unnamed"),
                    desired_x,
                    desired_y
                ));
                continue;
            };

            // Track spawned position to avoid clustering
            spawned_shelters.push((spawn_x, spawn_y));

            // Spawn the shelter
            ShelterSystem::spawn_shelter(
                entities,
                next_entity_id,
                shelter_type.clone(),
                spawn_x,
                spawn_y,
                condition.clone(),
                name.map(|s| s.to_string()),
            );
        }
    }

    /// Find a safe spawn position for an entity type
    pub fn find_safe_spawn_position(
        entities: &[GameEntity],
        entity_type: &EntityType,
        min_distance: f32,
        max_attempts: u32,
    ) -> Option<(f32, f32)> {
        let (min_x, max_x, min_y, max_y) = Self::get_spawn_bounds(entity_type);

        (0..max_attempts).find_map(|_| {
            let x = rand::gen_range(min_x, max_x);
            let y = rand::gen_range(min_y, max_y);

            if Self::is_valid_spawn_position(entities, x, y, min_distance) {
                Some((x, y))
            } else {
                None
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_spawn() {
        let mut entities = Vec::new();
        let mut next_id = 0;

        let player_id = WorldSystem::spawn_player(&mut entities, &mut next_id);

        assert_eq!(entities.len(), 1);
        assert_eq!(player_id, 0);
        assert_eq!(next_id, 1);
        assert!(matches!(entities[0].entity_type, EntityType::Player));
    }

    #[test]
    fn test_clan_initialization() {
        let mut clans = HashMap::new();

        WorldSystem::initialize_clans(&mut clans);

        assert_eq!(clans.len(), 3);
        assert!(clans.contains_key("Bone-Eaters"));
        assert!(clans.contains_key("Flame-Haters"));
        assert!(clans.contains_key("Night-Bloods"));
    }

    #[test]
    fn test_spawn_bounds() {
        let bounds = WorldSystem::get_spawn_bounds(&EntityType::Player);
        assert_eq!(
            bounds,
            (
                -1000.0,
                GAME_WORLD_WIDTH + 1000.0,
                GROUND_LEVEL,
                GAME_WORLD_HEIGHT * 2.0
            )
        );

        let bounds = WorldSystem::get_spawn_bounds(&EntityType::Animal);
        assert_eq!(
            bounds,
            (
                -1000.0,
                GAME_WORLD_WIDTH + 1000.0,
                GROUND_LEVEL,
                GAME_WORLD_HEIGHT * 2.0
            )
        );

        // Test that all entity types return consistent expanded bounds
        let hostile_bounds = WorldSystem::get_spawn_bounds(&EntityType::HostileInfected);
        let clan_bounds =
            WorldSystem::get_spawn_bounds(&EntityType::ClanMember("Test".to_string()));
        let shelter_bounds = WorldSystem::get_spawn_bounds(&EntityType::Shelter);

        // All should have the same expanded bounds for dynamic spawning
        assert_eq!(hostile_bounds, bounds);
        assert_eq!(clan_bounds, bounds);
        assert_eq!(shelter_bounds, bounds);
    }

    #[test]
    fn test_valid_spawn_position() {
        let entities = vec![GameEntity {
            id: 0,
            position: Position { x: 100.0, y: 100.0 },
            velocity: Some(Velocity { x: 0.0, y: 0.0 }),
            entity_type: EntityType::Animal,
            health: None,
            combat_stats: None,
            ai_state: AIState::Idle,
            blood_meter: None,
            vampire_abilities: None,
            shelter: None,
            shelter_occupancy: None,
            color: WHITE,
        }];

        // Position too close should be invalid
        assert!(!WorldSystem::is_valid_spawn_position(
            &entities, 101.0, 101.0, 50.0
        ));

        // Position far enough should be valid
        assert!(WorldSystem::is_valid_spawn_position(
            &entities, 200.0, 200.0, 50.0
        ));
    }
}
