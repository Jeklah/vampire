//! World System Module
//!
//! Handles world initialization, entity spawning, and world setup utilities.
//! This system is responsible for creating the initial game world state.

use crate::components::*;
use macroquad::prelude::*;
use std::collections::HashMap;

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
            velocity: Velocity { x: 0.0, y: 0.0 },
            health: Some(Health {
                current: 100.0,
                maximum: 100.0,
            }),
            blood_meter: Some(BloodMeter {
                current: 50.0,
                maximum: 100.0,
                drain_rate: 1.0,
            }),
            abilities: Some(VampireAbilities {
                strength: 1.0,
                speed: 1.0,
                blood_sense: 0.0,
                shadow_movement: 0.0,
            }),
            combat_stats: Some(CombatStats::new(25.0, 10.0)),
            entity_type: EntityType::Player,
            color: RED,
            ai_target: None,
            ai_state: AIState::Idle,
            facing_direction: 0.0,
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
            700.0,
            PURPLE,
        );
        Self::spawn_clan_leader(
            entities,
            next_entity_id,
            "Silentfang",
            "Night-Bloods",
            800.0,
            620.0,
            DARKBLUE,
        );
    }

    /// Spawn a single clan leader
    pub fn spawn_clan_leader(
        entities: &mut Vec<GameEntity>,
        next_entity_id: &mut u32,
        _name: &str,
        clan: &str,
        x: f32,
        y: f32,
        color: Color,
    ) -> u32 {
        let entity_id = *next_entity_id;
        let entity = GameEntity {
            id: entity_id,
            position: Position { x, y },
            velocity: Velocity { x: 0.0, y: 0.0 },
            health: Some(Health {
                current: 120.0,
                maximum: 120.0,
            }),
            blood_meter: None,
            abilities: None,
            combat_stats: Some(CombatStats::new(30.0, 15.0)),
            entity_type: EntityType::ClanLeader(clan.to_string()),
            color,
            ai_target: None,
            ai_state: AIState::Idle,
            facing_direction: 0.0,
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
        for _ in 0..count {
            let x = rand::gen_range(100.0, 1000.0);
            let y = rand::gen_range(610.0, 1100.0);
            Self::spawn_hostile_infected(entities, next_entity_id, x, y);
        }
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
            velocity: Velocity { x: 0.0, y: 0.0 },
            health: Some(Health {
                current: 50.0,
                maximum: 50.0,
            }),
            blood_meter: None,
            abilities: None,
            combat_stats: Some(CombatStats::new(15.0, 5.0)),
            entity_type: EntityType::HostileInfected,
            color: Color::new(0.5, 0.1, 0.1, 1.0),
            ai_target: None,
            ai_state: AIState::Hostile,
            facing_direction: 0.0,
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
        for _ in 0..count {
            let x = rand::gen_range(50.0, 1200.0);
            let y = rand::gen_range(610.0, 1150.0);
            Self::spawn_animal(entities, next_entity_id, x, y);
        }
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
            velocity: Velocity { x: 0.0, y: 0.0 },
            health: Some(Health {
                current: 25.0,
                maximum: 25.0,
            }),
            blood_meter: None,
            abilities: None,
            combat_stats: None,
            entity_type: EntityType::Animal,
            color: BROWN,
            ai_target: None,
            ai_state: AIState::Idle,
            facing_direction: 0.0,
        };

        entities.push(entity);
        *next_entity_id += 1;
        entity_id
    }

    /// Initialize the starfield background
    pub fn initialize_starfield(stars: &mut Vec<Star>) {
        stars.clear();
        for _ in 0..200 {
            let x = rand::gen_range(0.0, 1600.0);
            let y = rand::gen_range(0.0, 1200.0);
            stars.push(Star::new(x, y));
        }
    }

    /// Initialize the moon
    pub fn initialize_moon(moon: &mut Moon) {
        *moon = Moon::new();
    }

    /// Initialize ground terrain tiles
    pub fn initialize_ground_terrain(ground_tiles: &mut Vec<GroundTile>) {
        ground_tiles.clear();

        let tile_size = 64.0;
        let world_width = 1600.0;
        let world_height = 1200.0;
        let ground_level = 600.0; // Ground starts at y = 600

        for x in (0..((world_width / tile_size) as i32)).map(|i| i as f32 * tile_size) {
            for y in (((ground_level / tile_size) as i32)..((world_height / tile_size) as i32))
                .map(|i| i as f32 * tile_size)
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

    /// Spawn a clan member at a specific location
    pub fn spawn_clan_member(
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
            velocity: Velocity { x: 0.0, y: 0.0 },
            health: Some(Health {
                current: 80.0,
                maximum: 80.0,
            }),
            blood_meter: None,
            abilities: None,
            combat_stats: Some(CombatStats::new(20.0, 8.0)),
            entity_type: EntityType::ClanMember(clan_name.to_string()),
            color,
            ai_target: None,
            ai_state: AIState::Idle,
            facing_direction: 0.0,
        };

        entities.push(entity);
        *next_entity_id += 1;
        entity_id
    }

    /// Get spawn bounds for different entity types
    pub fn get_spawn_bounds(entity_type: &EntityType) -> (f32, f32, f32, f32) {
        match entity_type {
            EntityType::Player => (300.0, 500.0, 600.0, 700.0),
            EntityType::ClanLeader(_) => (100.0, 900.0, 600.0, 750.0),
            EntityType::ClanMember(_) => (100.0, 900.0, 600.0, 750.0),
            EntityType::HostileInfected => (50.0, 1000.0, 610.0, 1100.0),
            EntityType::Animal => (50.0, 1200.0, 610.0, 1150.0),
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

    /// Find a safe spawn position for an entity type
    pub fn find_safe_spawn_position(
        entities: &[GameEntity],
        entity_type: &EntityType,
        min_distance: f32,
        max_attempts: u32,
    ) -> Option<(f32, f32)> {
        let (min_x, max_x, min_y, max_y) = Self::get_spawn_bounds(entity_type);

        for _ in 0..max_attempts {
            let x = rand::gen_range(min_x, max_x);
            let y = rand::gen_range(min_y, max_y);

            if Self::is_valid_spawn_position(entities, x, y, min_distance) {
                return Some((x, y));
            }
        }

        None
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
        assert_eq!(bounds, (300.0, 500.0, 600.0, 700.0));

        let bounds = WorldSystem::get_spawn_bounds(&EntityType::Animal);
        assert_eq!(bounds, (50.0, 1200.0, 610.0, 1150.0));
    }

    #[test]
    fn test_valid_spawn_position() {
        let entities = vec![GameEntity {
            id: 0,
            position: Position { x: 100.0, y: 100.0 },
            velocity: Velocity { x: 0.0, y: 0.0 },
            health: None,
            blood_meter: None,
            abilities: None,
            combat_stats: None,
            entity_type: EntityType::Animal,
            color: WHITE,
            ai_target: None,
            ai_state: AIState::Idle,
            facing_direction: 0.0,
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
