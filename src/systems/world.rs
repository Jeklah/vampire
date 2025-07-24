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
pub const HORIZON_MOVEMENT_THRESHOLD: f32 = 5.0; // Minimum movement toward horizon to trigger effect
pub const GROUND_SHIFT_DISTANCE: f32 = 200.0; // How far to shift ground tiles during horizon movement

/// World system responsible for entity spawning and world management
pub struct WorldSystem;

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

        // Spawn hostile infected creatures
        Self::spawn_hostile_infected_group(entities, next_entity_id, 8);

        // Spawn animals (blood sources)
        Self::spawn_animal_group(entities, next_entity_id, 12);

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
        (0..count).for_each(|_| {
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
    pub fn spawn_animal_group(
        entities: &mut Vec<GameEntity>,
        next_entity_id: &mut u32,
        count: usize,
    ) {
        (0..count).for_each(|_| {
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

    /// Initialize ground terrain tiles
    pub fn initialize_ground_terrain(ground_tiles: &mut Vec<GroundTile>) {
        ground_tiles.clear();

        let tile_size = 64.0;

        // Calculate how many tiles we need to cover the full game world width
        let tiles_across = (GAME_WORLD_WIDTH / tile_size).ceil() as i32;
        // Calculate how many tiles we need from ground level to bottom of game world
        let tiles_down = ((GAME_WORLD_HEIGHT - GROUND_LEVEL) / tile_size).ceil() as i32;

        for x in (0..tiles_across).map(|i| i as f32 * tile_size) {
            // Ensure tiles start exactly at ground level
            let start_tile_y = ((GROUND_LEVEL / tile_size).ceil() as i32) * tile_size as i32;
            // Generate tiles from ground level to bottom of game world
            for y in (0..tiles_down)
                .map(|i| start_tile_y + (i * tile_size as i32))
                .map(|i| i as f32)
            {
                let tile_type = Self::determine_tile_type();
                ground_tiles.push(GroundTile::new(x, y, tile_type));
            }
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
        player_y: f32,
        movement_distance: f32,
        debug_messages: &mut Vec<String>,
    ) {
        let initial_tile_count = ground_tiles.len();

        // Calculate screen-based bounds for better tile management
        let screen_width = 1280.0; // Approximate screen width
        let screen_height = 720.0; // Approximate screen height
        let tile_size = 64.0;

        // More conservative cleanup bounds based on visible area
        let cleanup_x_range = screen_width * 1.5; // Keep tiles 1.5 screen widths around player
        let cleanup_y_range = screen_height * 2.0; // Keep tiles 2 screen heights around player

        // Only shift tiles if movement is significant enough
        if movement_distance.abs() > 1.0 {
            // Shift existing ground tiles away from player
            for tile in ground_tiles.iter_mut() {
                tile.y += movement_distance;
            }
        }

        // Generate new ground tiles more conservatively
        let tiles_to_generate_x = (cleanup_x_range as f32 / tile_size).ceil() as i32;
        let start_x = ((player_x - cleanup_x_range as f32 / 2.0) / tile_size).floor() * tile_size;

        let mut tiles_added = 0;

        // Add new tiles at the horizon area where player is moving
        for i in 0..tiles_to_generate_x {
            let x = start_x + (i as f32 * tile_size);

            // Add tiles at the horizon line and slightly below
            for row in 0..2 {
                let y = HORIZON_LINE + (row as f32 * tile_size);

                // Check if we already have a tile at this position with better tolerance
                let tile_exists = ground_tiles.iter().any(|tile| {
                    (tile.x - x).abs() < tile_size * 0.8 && (tile.y - y).abs() < tile_size * 0.8
                });

                if !tile_exists {
                    let tile_type = Self::determine_tile_type();
                    ground_tiles.push(GroundTile::new(x, y, tile_type));
                    tiles_added += 1;
                }
            }
        }

        // Clean up tiles using screen-relative bounds instead of fixed distance
        let before_cleanup = ground_tiles.len();
        ground_tiles.retain(|tile| {
            let x_in_range = (tile.x - player_x).abs() < cleanup_x_range;
            let y_in_range = (tile.y - player_y).abs() < cleanup_y_range;
            let not_too_far_down = tile.y < GAME_WORLD_HEIGHT + 200.0;

            x_in_range && y_in_range && not_too_far_down
        });
        let after_cleanup = ground_tiles.len();
        let tiles_removed = before_cleanup - after_cleanup;

        // Add debug information
        if tiles_added > 0 || tiles_removed > 0 {
            debug_messages.push(format!(
                "Ground tiles: {} → {} (added: {}, removed: {})",
                initial_tile_count,
                ground_tiles.len(),
                tiles_added,
                tiles_removed
            ));
        }
    }

    /// Check if a position has ground (is within the ground area)
    pub fn has_ground_at_position(x: f32, y: f32) -> bool {
        // Check if position is within world bounds and at or below ground level
        x >= 0.0 && x <= GAME_WORLD_WIDTH && y >= GROUND_LEVEL && y <= GAME_WORLD_HEIGHT
    }

    /// Generate a random position within the ground area
    pub fn generate_random_ground_position() -> (f32, f32) {
        // Generate random position within ground area with some padding from edges
        let padding = 64.0;
        let x = rand::gen_range(padding, GAME_WORLD_WIDTH - padding);
        let y = rand::gen_range(GROUND_LEVEL + padding, GAME_WORLD_HEIGHT - padding);

        (x, y)
    }

    /// Check if a position is close enough to ground area to be relocated
    pub fn is_relocatable_to_ground(x: f32, y: f32) -> bool {
        // Only relocate if:
        // 1. X coordinate is within world bounds
        // 2. Y coordinate is not too far above ground (within 100 units)
        x >= 0.0 && x <= GAME_WORLD_WIDTH && y >= (GROUND_LEVEL - 100.0) && y < GROUND_LEVEL
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
        // Return (min_x, max_x, min_y, max_y) based on entity type
        match entity_type {
            EntityType::Player => (350.0, 450.0, GROUND_LEVEL, 740.0),
            EntityType::ClanLeader(_) => (200.0, GAME_WORLD_WIDTH - 400.0, GROUND_LEVEL, 750.0),
            EntityType::ClanMember(_) => (100.0, GAME_WORLD_WIDTH - 200.0, GROUND_LEVEL, 800.0),
            EntityType::HostileInfected => (50.0, GAME_WORLD_WIDTH - 250.0, GROUND_LEVEL, 850.0),
            EntityType::Animal => (
                50.0,
                GAME_WORLD_WIDTH - 400.0,
                650.0,
                GAME_WORLD_HEIGHT - 50.0,
            ),
            EntityType::Shelter => (0.0, GAME_WORLD_WIDTH, 0.0, 800.0),
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
        assert_eq!(bounds, (350.0, 450.0, 640.0, 740.0));

        let bounds = WorldSystem::get_spawn_bounds(&EntityType::Animal);
        assert_eq!(
            bounds,
            (
                50.0,
                GAME_WORLD_WIDTH - 400.0,
                650.0,
                GAME_WORLD_HEIGHT - 50.0
            )
        );
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
