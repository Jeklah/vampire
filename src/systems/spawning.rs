//! Spawning System for Dynamic Entity Management
//!
//! This module provides a spawning system that manages entity populations
//! by respawning entities that have been cleaned up, maintaining a dynamic
//! and engaging game world.

use crate::components::*;
use crate::systems::world::WorldSystem;
use macroquad::prelude::*;
use std::collections::HashMap;

/// Configuration for different entity spawn types
#[derive(Debug, Clone)]
pub struct SpawnConfig {
    /// Maximum number of entities of this type in the world
    pub max_count: usize,
    /// Minimum time between spawn attempts (in seconds)
    pub spawn_interval: f32,
    /// Maximum distance from player to spawn entities
    pub max_spawn_distance: f32,
    /// Minimum distance from player to spawn entities
    pub min_spawn_distance: f32,
    /// Number to spawn per spawn event
    pub spawn_batch_size: usize,
    /// Whether this entity type should be spawned
    pub enabled: bool,
}

impl SpawnConfig {
    /// Create a new spawn configuration
    pub fn new(
        max_count: usize,
        spawn_interval: f32,
        min_spawn_distance: f32,
        max_spawn_distance: f32,
        spawn_batch_size: usize,
    ) -> Self {
        Self {
            max_count,
            spawn_interval,
            max_spawn_distance,
            min_spawn_distance,
            spawn_batch_size,
            enabled: true,
        }
    }

    /// Create configuration for hostile entities
    pub fn hostile_default() -> Self {
        Self::new(15, 3.0, 150.0, 400.0, 4) // Max 15, spawn every 3s, 4 at a time (more frequent spawning)
    }

    /// Create configuration for animals
    pub fn animal_default() -> Self {
        Self::new(12, 5.0, 150.0, 350.0, 3) // Max 12, spawn every 5s, 3 at a time
    }

    /// Create configuration for clan members
    pub fn clan_member_default() -> Self {
        Self::new(8, 15.0, 300.0, 500.0, 1) // Max 8, spawn every 15s, 1 at a time
    }

    /// Adjust config for performance mode
    pub fn performance_mode(&mut self) -> &mut Self {
        self.max_count = (self.max_count as f32 * 0.7) as usize; // 30% fewer entities
        self.spawn_interval *= 1.5; // Spawn 50% less frequently
        self
    }
}

/// Spawn timer tracking for each entity type
#[derive(Debug, Clone)]
struct SpawnTimer {
    last_spawn_time: f32,
    config: SpawnConfig,
}

impl SpawnTimer {
    fn new(config: SpawnConfig) -> Self {
        Self {
            last_spawn_time: 0.0,
            config,
        }
    }

    fn should_spawn(&self, current_time: f32) -> bool {
        self.config.enabled && (current_time - self.last_spawn_time) >= self.config.spawn_interval
    }

    fn mark_spawned(&mut self, current_time: f32) {
        self.last_spawn_time = current_time;
    }
}

/// Main spawning system managing entity populations
pub struct SpawningSystem {
    /// Spawn timers for each entity type
    spawn_timers: HashMap<String, SpawnTimer>,
    /// Performance mode flag
    performance_mode: bool,
    /// Random spawn offset to prevent predictable patterns
    spawn_randomness: f32,
    /// Statistics for monitoring
    stats: SpawnStats,
}

/// Statistics for spawn system monitoring
#[derive(Debug, Clone, Default)]
pub struct SpawnStats {
    pub total_spawned: usize,
    pub spawns_by_type: HashMap<String, usize>,
    pub last_spawn_time: f32,
    pub spawn_attempts: usize,
    pub failed_spawns: usize,
}

impl SpawningSystem {
    /// Create a new spawning system with default configurations
    pub fn new() -> Self {
        let mut system = Self {
            spawn_timers: HashMap::new(),
            performance_mode: false,
            spawn_randomness: 0.3, // 30% randomness in spawn timing
            stats: SpawnStats::default(),
        };

        // Initialize with default configurations
        system.add_spawn_type("hostile", SpawnConfig::hostile_default());
        system.add_spawn_type("animal", SpawnConfig::animal_default());
        system.add_spawn_type("clan_member", SpawnConfig::clan_member_default());

        system
    }

    /// Add or update a spawn configuration for an entity type
    pub fn add_spawn_type(&mut self, type_name: &str, config: SpawnConfig) {
        let timer = SpawnTimer::new(config);
        self.spawn_timers.insert(type_name.to_string(), timer);
    }

    /// Enable or disable performance mode
    pub fn set_performance_mode(&mut self, enabled: bool) {
        if self.performance_mode == enabled {
            return; // No change needed
        }

        self.performance_mode = enabled;

        // Adjust all spawn configurations for performance
        for timer in self.spawn_timers.values_mut() {
            if enabled {
                timer.config.performance_mode();
            } else {
                // Reset to defaults (this is a simplified approach)
                match timer.config.max_count {
                    count if count <= 8 => timer.config = SpawnConfig::hostile_default(),
                    count if count <= 11 => timer.config = SpawnConfig::animal_default(),
                    _ => timer.config = SpawnConfig::clan_member_default(),
                }
            }
        }
    }

    /// Update the spawning system
    pub fn update(
        &mut self,
        entities: &mut Vec<GameEntity>,
        next_entity_id: &mut u32,
        player_id: u32,
        current_time: f32,
        max_total_entities: usize,
    ) {
        // Skip if we're at entity limit
        if entities.len() >= max_total_entities {
            return;
        }

        // Get player position for spawn calculations
        let player_pos = self.get_player_position(entities, player_id);
        if player_pos.is_none() {
            return;
        }

        let player_position = player_pos.unwrap();

        // Check each spawn type
        let mut types_to_spawn = Vec::new();
        for (type_name, timer) in &self.spawn_timers {
            if self.should_attempt_spawn(entities, type_name, timer, current_time) {
                types_to_spawn.push(type_name.clone());
            }
        }

        // Spawn entities
        for type_name in types_to_spawn {
            self.attempt_spawn(
                entities,
                next_entity_id,
                &type_name,
                player_position,
                current_time,
                max_total_entities,
            );
        }
    }

    /// Check if we should attempt to spawn entities of this type
    fn should_attempt_spawn(
        &self,
        entities: &[GameEntity],
        type_name: &str,
        timer: &SpawnTimer,
        current_time: f32,
    ) -> bool {
        if !timer.should_spawn(current_time) {
            return false;
        }

        // Count current entities of this type
        let current_count = self.count_entities_of_type(entities, type_name);
        current_count < timer.config.max_count
    }

    /// Attempt to spawn entities of the specified type
    fn attempt_spawn(
        &mut self,
        entities: &mut Vec<GameEntity>,
        next_entity_id: &mut u32,
        type_name: &str,
        player_position: Position,
        current_time: f32,
        max_total_entities: usize,
    ) {
        // Extract config values to avoid borrowing issues
        let (
            spawn_interval,
            spawn_randomness,
            max_count,
            spawn_batch_size,
            min_spawn_distance,
            max_spawn_distance,
            last_spawn_time,
        ) = {
            let timer = match self.spawn_timers.get(type_name) {
                Some(timer) => timer,
                None => return,
            };

            (
                timer.config.spawn_interval,
                self.spawn_randomness,
                timer.config.max_count,
                timer.config.spawn_batch_size,
                timer.config.min_spawn_distance,
                timer.config.max_spawn_distance,
                timer.last_spawn_time,
            )
        };

        self.stats.spawn_attempts += 1;

        // Add randomness to spawn timing
        let random_delay = rand::gen_range(0.0, spawn_interval * spawn_randomness);
        if (current_time - last_spawn_time) < (spawn_interval - random_delay) {
            return;
        }

        // Determine how many to spawn
        let current_count = self.count_entities_of_type(entities, type_name);
        let needed = max_count.saturating_sub(current_count);
        let to_spawn = needed.min(spawn_batch_size);

        if to_spawn == 0 {
            return;
        }

        // Create spawn config for position finding
        let spawn_config = SpawnConfig {
            max_count,
            spawn_interval,
            max_spawn_distance,
            min_spawn_distance,
            spawn_batch_size,
            enabled: true,
        };

        // Spawn entities
        let mut spawned_count = 0;
        for _ in 0..to_spawn {
            if entities.len() >= max_total_entities {
                break;
            }

            if let Some((spawn_x, spawn_y)) =
                self.find_spawn_position(player_position, &spawn_config)
            {
                match type_name {
                    "hostile" => {
                        WorldSystem::spawn_hostile_infected(
                            entities,
                            next_entity_id,
                            spawn_x,
                            spawn_y,
                        );
                        spawned_count += 1;
                    }
                    "animal" => {
                        WorldSystem::spawn_animal(entities, next_entity_id, spawn_x, spawn_y);
                        spawned_count += 1;
                    }
                    "clan_member" => {
                        // Spawn a random clan member
                        let clan_names = ["Bone-Eaters", "Flame-Haters", "Night-Bloods"];
                        let clan_name = clan_names[rand::gen_range(0, clan_names.len())];
                        let color = match clan_name {
                            "Bone-Eaters" => LIGHTGRAY,
                            "Flame-Haters" => VIOLET,
                            "Night-Bloods" => BLUE,
                            _ => WHITE,
                        };
                        WorldSystem::spawn_clan_member(
                            entities,
                            next_entity_id,
                            clan_name,
                            spawn_x,
                            spawn_y,
                            color,
                        );
                        spawned_count += 1;
                    }
                    _ => {
                        self.stats.failed_spawns += 1;
                    }
                }
            } else {
                self.stats.failed_spawns += 1;
            }
        }

        // Update statistics and timer
        if spawned_count > 0 {
            self.stats.total_spawned += spawned_count;
            self.stats.last_spawn_time = current_time;
            *self
                .stats
                .spawns_by_type
                .entry(type_name.to_string())
                .or_insert(0) += spawned_count;

            // Mark timer as spawned
            if let Some(timer) = self.spawn_timers.get_mut(type_name) {
                timer.mark_spawned(current_time);
            }
        }
    }

    /// Find a valid spawn position around the player
    fn find_spawn_position(
        &self,
        player_pos: Position,
        config: &SpawnConfig,
    ) -> Option<(f32, f32)> {
        const MAX_ATTEMPTS: usize = 20; // Increased attempts for better success rate

        for _ in 0..MAX_ATTEMPTS {
            // Generate random angle and distance
            let angle = rand::gen_range(0.0, 2.0 * std::f32::consts::PI);
            let distance = rand::gen_range(config.min_spawn_distance, config.max_spawn_distance);

            let spawn_x = player_pos.x + angle.cos() * distance;
            let spawn_y = player_pos.y + angle.sin() * distance;

            // Improved validation - use proper ground level bounds
            // GROUND_LEVEL is 640.0, so allow spawning at and below ground level
            if spawn_y >= 640.0 && spawn_y <= 1200.0 {
                // Within proper ground bounds (aligned with GROUND_LEVEL system)
                return Some((spawn_x, spawn_y));
            }
        }

        // Fallback: try spawning directly at ground level near player
        for _ in 0..5 {
            let angle = rand::gen_range(0.0, 2.0 * std::f32::consts::PI);
            let distance = rand::gen_range(config.min_spawn_distance, config.max_spawn_distance);

            let spawn_x = player_pos.x + angle.cos() * distance;
            let spawn_y = 650.0; // Just below ground level

            return Some((spawn_x, spawn_y));
        }

        None
    }

    /// Count entities of a specific type
    fn count_entities_of_type(&self, entities: &[GameEntity], type_name: &str) -> usize {
        entities
            .iter()
            .filter(|entity| {
                // Don't count dead entities
                if matches!(entity.ai_state, AIState::Dead) {
                    return false;
                }

                // Don't count entities with zero health
                if entity.health.as_ref().map_or(false, |h| h.current <= 0.0) {
                    return false;
                }

                // Match entity type
                match type_name {
                    "hostile" => matches!(entity.entity_type, EntityType::HostileInfected),
                    "animal" => matches!(entity.entity_type, EntityType::Animal),
                    "clan_member" => matches!(entity.entity_type, EntityType::ClanMember(_)),
                    _ => false,
                }
            })
            .count()
    }

    /// Get player position from entity list
    fn get_player_position(&self, entities: &[GameEntity], player_id: u32) -> Option<Position> {
        entities
            .iter()
            .find(|e| e.id == player_id)
            .map(|e| e.position)
    }

    /// Get current spawn statistics
    pub fn get_stats(&self) -> &SpawnStats {
        &self.stats
    }

    /// Get spawn configuration for a type
    pub fn get_config(&self, type_name: &str) -> Option<&SpawnConfig> {
        self.spawn_timers.get(type_name).map(|timer| &timer.config)
    }

    /// Enable or disable spawning for a specific type
    pub fn set_spawn_enabled(&mut self, type_name: &str, enabled: bool) {
        if let Some(timer) = self.spawn_timers.get_mut(type_name) {
            timer.config.enabled = enabled;
        }
    }

    /// Reset all spawn timers (useful for level changes)
    pub fn reset_timers(&mut self, current_time: f32) {
        for timer in self.spawn_timers.values_mut() {
            timer.last_spawn_time = current_time;
        }
    }

    /// Clear all statistics
    pub fn clear_stats(&mut self) {
        self.stats = SpawnStats::default();
    }

    /// Configure spawn settings for a specific entity type
    pub fn configure_spawn_type(&mut self, type_name: &str, config: SpawnConfig) {
        if let Some(timer) = self.spawn_timers.get_mut(type_name) {
            timer.config = config;
        } else {
            let timer = SpawnTimer::new(config);
            self.spawn_timers.insert(type_name.to_string(), timer);
        }
    }

    /// Adjust spawn rates globally (multiplier applied to all intervals)
    pub fn set_global_spawn_rate_multiplier(&mut self, multiplier: f32) {
        for timer in self.spawn_timers.values_mut() {
            timer.config.spawn_interval *= multiplier;
        }
    }

    /// Enable/disable spawning temporarily (useful for cutscenes, etc.)
    pub fn set_spawning_enabled(&mut self, enabled: bool) {
        for timer in self.spawn_timers.values_mut() {
            timer.config.enabled = enabled;
        }
    }

    /// Get a list of all configured spawn types
    pub fn get_spawn_types(&self) -> Vec<String> {
        self.spawn_timers.keys().cloned().collect()
    }
}

impl Default for SpawningSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_config_creation() {
        let config = SpawnConfig::hostile_default();
        assert_eq!(config.max_count, 12);
        assert_eq!(config.spawn_interval, 8.0);
        assert!(config.enabled);
    }

    #[test]
    fn test_performance_mode() {
        let mut config = SpawnConfig::hostile_default();
        let original_count = config.max_count;
        config.performance_mode();
        assert!(config.max_count < original_count);
    }

    #[test]
    fn test_spawn_timer_logic() {
        let config = SpawnConfig::new(5, 2.0, 100.0, 200.0, 1);
        let mut timer = SpawnTimer::new(config);

        // Should not spawn immediately
        assert!(!timer.should_spawn(0.0));

        // Should spawn after interval
        assert!(timer.should_spawn(2.1));

        // Mark as spawned and check again
        timer.mark_spawned(2.1);
        assert!(!timer.should_spawn(2.5));
        assert!(timer.should_spawn(4.2));
    }

    #[test]
    fn test_spawning_system_creation() {
        let system = SpawningSystem::new();
        assert!(system.spawn_timers.contains_key("hostile"));
        assert!(system.spawn_timers.contains_key("animal"));
        assert!(system.spawn_timers.contains_key("clan_member"));
    }

    #[test]
    fn test_entity_counting() {
        let system = SpawningSystem::new();
        let entities = vec![
            GameEntity {
                id: 1,
                entity_type: EntityType::HostileInfected,
                ai_state: AIState::Idle,
                health: Some(Health::new(50.0)),
                ..Default::default()
            },
            GameEntity {
                id: 2,
                entity_type: EntityType::Animal,
                ai_state: AIState::Idle,
                health: Some(Health::new(25.0)),
                ..Default::default()
            },
        ];

        assert_eq!(system.count_entities_of_type(&entities, "hostile"), 1);
        assert_eq!(system.count_entities_of_type(&entities, "animal"), 1);
        assert_eq!(system.count_entities_of_type(&entities, "clan_member"), 0);
    }
}
