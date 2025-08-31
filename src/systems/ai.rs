//! AI System Module
//!
//! Handles NPC behavior, decision making, and AI-driven combat logic.
//! This system manages different AI states and behaviors for non-player entities.

use crate::components::*;
use crate::systems::spatial_grid::SpatialGrid;
use macroquad::prelude::*;
use std::collections::HashMap;

/// AI system responsible for NPC behavior and decision making
pub struct AISystem {
    /// Spatial grid for efficient proximity queries
    spatial_grid: SpatialGrid,
    /// Cache for AI processing to reduce per-frame calculations
    ai_cache: HashMap<u32, AICache>,
    /// Frame counter for staggered updates
    frame_counter: u64,
    /// Maximum AI processing range
    ai_range: f32,
    /// Performance mode settings
    performance_mode: bool,
}

/// Cached AI data to reduce calculations
#[derive(Debug, Clone)]
struct AICache {
    last_player_distance: f32,
    last_update_frame: u64,
    target_position: Option<Position>,
    state_change_cooldown: f32,
}

impl AISystem {
    /// Create a new AI system with optimizations
    pub fn new() -> Self {
        Self {
            spatial_grid: SpatialGrid::new(100.0), // 100 unit cells
            ai_cache: HashMap::new(),
            frame_counter: 0,
            ai_range: 300.0, // Only process AI within 300 units of player
            performance_mode: false,
        }
    }

    /// Enable or disable performance mode
    pub fn set_performance_mode(&mut self, enabled: bool) {
        self.performance_mode = enabled;
        if enabled {
            self.ai_range = 200.0; // Reduce range in performance mode
        } else {
            self.ai_range = 300.0;
        }
    }

    /// Update AI for all entities with spatial optimizations
    pub fn update_all_ai(
        &mut self,
        entities: &mut Vec<GameEntity>,
        player_id: u32,
        delta_time: f32,
    ) {
        self.frame_counter += 1;

        // Update spatial grid
        self.spatial_grid.update(entities);

        // Clean old cache entries
        if self.frame_counter % 120 == 0 {
            // Every 2 seconds at 60fps
            self.cleanup_cache(entities);
        }
        let player_pos = Self::get_player_position(entities, player_id);

        if player_pos.is_none() {
            return; // Can't do AI without player position
        }

        let player_position = player_pos.unwrap();

        // Get entities within AI processing range using spatial grid
        let nearby_entity_ids = self
            .spatial_grid
            .get_entities_in_radius(player_position, self.ai_range);

        // Pre-allocate with estimated capacity for better performance
        let mut ai_updates = Vec::with_capacity(nearby_entity_ids.len());

        // Process only nearby entities
        for &entity_id in &nearby_entity_ids {
            if entity_id == player_id {
                continue; // Skip player
            }

            if let Some(entity) = entities.iter().find(|e| e.id == entity_id) {
                // Skip dead entities
                if matches!(entity.ai_state, AIState::Dead) {
                    continue;
                }

                // Check if we should update this entity this frame (staggered updates)
                if self.should_update_entity(entity_id, entity, &player_position) {
                    let update = match entity.ai_state {
                        AIState::Hostile => {
                            self.update_hostile_ai(entity, &player_position, delta_time)
                        }
                        AIState::Fleeing => {
                            self.update_fleeing_ai(entity, &player_position, delta_time)
                        }
                        AIState::Idle => self.update_idle_ai(entity, &player_position, delta_time),
                        AIState::Dead => None,
                    };

                    if let Some(ai_update) = update {
                        ai_updates.push(ai_update);
                    }

                    // Update cache
                    let distance = Self::calculate_distance(&entity.position, &player_position);
                    self.ai_cache.insert(
                        entity_id,
                        AICache {
                            last_player_distance: distance,
                            last_update_frame: self.frame_counter,
                            target_position: Some(player_position),
                            state_change_cooldown: 0.0,
                        },
                    );
                }
            }
        }

        // Apply AI updates
        self.apply_ai_updates(entities, ai_updates, delta_time);
    }

    /// Get the player's current position using optimized entity finder
    fn get_player_position(entities: &[GameEntity], player_id: u32) -> Option<Position> {
        // Use optimized entity finder for better performance
        EntityFinder::by_id(entities, player_id).map(|player| player.position)
    }

    /// Get current AI statistics for debugging
    pub fn get_ai_statistics(&self) -> AIStatistics {
        AIStatistics {
            cached_entities: self.ai_cache.len(),
            spatial_grid_cells: self.spatial_grid.get_occupied_cells().len(),
            ai_range: self.ai_range,
            performance_mode: self.performance_mode,
            frame_counter: self.frame_counter,
            memory_usage: self.spatial_grid.estimated_memory_usage(),
        }
    }

    /// Check if an entity should be updated this frame (for staggered updates)
    fn should_update_entity(
        &self,
        entity_id: u32,
        entity: &GameEntity,
        player_pos: &Position,
    ) -> bool {
        // Always update if no cache exists
        let cache = match self.ai_cache.get(&entity_id) {
            Some(cache) => cache,
            None => return true,
        };

        // In performance mode, update less frequently based on distance
        if self.performance_mode {
            let distance = Self::calculate_distance(&entity.position, player_pos);
            let update_frequency = if distance < 100.0 {
                1 // Update every frame if close
            } else if distance < 200.0 {
                2 // Update every 2 frames if medium distance
            } else {
                4 // Update every 4 frames if far
            };

            return (self.frame_counter - cache.last_update_frame) >= update_frequency;
        }

        // Normal mode: update every frame or every other frame based on distance
        let distance = Self::calculate_distance(&entity.position, player_pos);
        if distance < 150.0 {
            true // Always update close entities
        } else {
            (self.frame_counter - cache.last_update_frame) >= 2 // Update far entities every other frame
        }
    }

    /// Clean up old cache entries for entities that no longer exist
    fn cleanup_cache(&mut self, entities: &[GameEntity]) {
        let existing_ids: std::collections::HashSet<u32> = entities.iter().map(|e| e.id).collect();
        self.ai_cache.retain(|&id, _| existing_ids.contains(&id));
    }

    /// Update hostile AI behavior
    fn update_hostile_ai(
        &mut self,
        entity: &GameEntity,
        player_pos: &Position,
        _delta_time: f32,
    ) -> Option<AIUpdate> {
        let distance = Self::calculate_distance(&entity.position, player_pos);

        // Adjust ranges based on performance mode
        let detection_range = if self.performance_mode { 150.0 } else { 200.0 };
        let attack_range = 30.0;

        if distance < detection_range {
            if distance < attack_range {
                // Close enough to attack
                Some(AIUpdate {
                    entity_id: entity.id,
                    new_velocity: Velocity { x: 0.0, y: 0.0 },
                    new_facing_direction: Some(Self::calculate_direction_to_target(
                        &entity.position,
                        player_pos,
                    )),
                    should_attack: true,
                })
            } else {
                // Move towards player
                let direction = Self::normalize_direction(
                    player_pos.x - entity.position.x,
                    player_pos.y - entity.position.y,
                );

                let speed = 106.0; // Slightly slower than player
                let velocity = Velocity {
                    x: direction.0 * speed,
                    y: direction.1 * speed,
                };

                Some(AIUpdate {
                    entity_id: entity.id,
                    new_velocity: velocity,
                    new_facing_direction: Some(Self::calculate_direction_to_target(
                        &entity.position,
                        player_pos,
                    )),
                    should_attack: false,
                })
            }
        } else {
            // Player out of range, stop moving
            Some(AIUpdate {
                entity_id: entity.id,
                new_velocity: Velocity { x: 0.0, y: 0.0 },
                new_facing_direction: None,
                should_attack: false,
            })
        }
    }

    /// Update fleeing AI behavior
    fn update_fleeing_ai(
        &mut self,
        entity: &GameEntity,
        player_pos: &Position,
        _delta_time: f32,
    ) -> Option<AIUpdate> {
        let distance = Self::calculate_distance(&entity.position, player_pos);
        let flee_range = if self.performance_mode { 120.0 } else { 150.0 };

        if distance < flee_range {
            // Flee away from player
            let direction = Self::normalize_direction(
                entity.position.x - player_pos.x, // Opposite direction
                entity.position.y - player_pos.y,
            );

            let speed = 140.0; // Faster when fleeing
            let velocity = Velocity {
                x: direction.0 * speed,
                y: direction.1 * speed,
            };

            Some(AIUpdate {
                entity_id: entity.id,
                new_velocity: velocity,
                new_facing_direction: Some(Self::calculate_direction_away_from_target(
                    &entity.position,
                    player_pos,
                )),
                should_attack: false,
            })
        } else {
            // Safe distance, stop fleeing
            Some(AIUpdate {
                entity_id: entity.id,
                new_velocity: Velocity { x: 0.0, y: 0.0 },
                new_facing_direction: None,
                should_attack: false,
            })
        }
    }

    /// Update idle AI behavior
    fn update_idle_ai(
        &mut self,
        entity: &GameEntity,
        player_pos: &Position,
        _delta_time: f32,
    ) -> Option<AIUpdate> {
        let distance = Self::calculate_distance(&entity.position, player_pos);

        // Check if entity should become hostile or flee based on entity type
        match entity.entity_type {
            EntityType::HostileInfected => {
                if distance < 100.0 {
                    // Become hostile when player is nearby
                    return Some(AIUpdate {
                        entity_id: entity.id,
                        new_velocity: Velocity { x: 0.0, y: 0.0 },
                        new_facing_direction: None,
                        should_attack: false,
                    });
                }
            }
            EntityType::Animal => {
                if distance < 80.0 {
                    // Animals flee when player approaches
                    return Some(AIUpdate {
                        entity_id: entity.id,
                        new_velocity: Velocity { x: 0.0, y: 0.0 },
                        new_facing_direction: None,
                        should_attack: false,
                    });
                }
            }
            _ => {
                // Other entities remain idle
            }
        }

        None
    }

    /// Apply AI updates to entities
    fn apply_ai_updates(
        &mut self,
        entities: &mut Vec<GameEntity>,
        updates: Vec<AIUpdate>,
        delta_time: f32,
    ) {
        for update in updates {
            if let Some(entity) = entities.iter_mut().find(|e| e.id == update.entity_id) {
                // Update velocity and position
                entity.velocity = Some(update.new_velocity);
                if let Some(velocity) = &entity.velocity {
                    entity.position.x += velocity.x * delta_time;
                    entity.position.y += velocity.y * delta_time;
                }

                // Update facing direction
                // Note: facing_direction field removed from GameEntity
                // Facing direction now calculated from velocity when needed

                // Allow entities to move in expanded world horizontally
                // Restrict Y movement to horizon line and below
                entity.position.y = entity.position.y.clamp(
                    crate::systems::world::GROUND_LEVEL, // Can't go above horizon line
                    crate::systems::world::GAME_WORLD_HEIGHT * 2.0, // Expand downward limit
                );
                // No X constraints - entities can move infinitely left and right

                // Update AI state based on behavior
                match entity.entity_type {
                    EntityType::HostileInfected => {
                        if update.should_attack {
                            entity.ai_state = AIState::Hostile;
                        } else if let Some(velocity) = &entity.velocity {
                            if velocity.x.abs() > 0.1 || velocity.y.abs() > 0.1 {
                                entity.ai_state = AIState::Hostile;
                            } else {
                                entity.ai_state = AIState::Idle;
                            }
                        }
                    }
                    EntityType::Animal => {
                        if let Some(velocity) = &entity.velocity {
                            if velocity.x.abs() > 0.1 || velocity.y.abs() > 0.1 {
                                entity.ai_state = AIState::Fleeing;
                            } else {
                                entity.ai_state = AIState::Idle;
                            }
                        }
                    }
                    _ => {
                        // Clan leaders and members maintain their state
                    }
                }
            }
        }
    }

    /// Calculate distance between two positions
    fn calculate_distance(pos1: &Position, pos2: &Position) -> f32 {
        ((pos1.x - pos2.x).powi(2) + (pos1.y - pos2.y).powi(2)).sqrt()
    }

    /// Normalize a direction vector
    fn normalize_direction(dx: f32, dy: f32) -> (f32, f32) {
        let length = (dx * dx + dy * dy).sqrt();
        if length > 0.0 {
            (dx / length, dy / length)
        } else {
            (0.0, 0.0)
        }
    }

    /// Calculate direction angle towards a target
    fn calculate_direction_to_target(from: &Position, to: &Position) -> f32 {
        (to.y - from.y).atan2(to.x - from.x)
    }

    /// Calculate direction angle away from a target
    fn calculate_direction_away_from_target(from: &Position, to: &Position) -> f32 {
        (from.y - to.y).atan2(from.x - to.x)
    }

    /// Check if an entity should start combat with the player
    pub fn should_initiate_combat(
        entity: &GameEntity,
        player_pos: &Position,
        aggression_level: f32,
    ) -> bool {
        let distance = Self::calculate_distance(&entity.position, player_pos);
        let combat_range = 40.0 * aggression_level;

        distance < combat_range && matches!(entity.ai_state, AIState::Hostile)
    }

    /// Get AI behavior description for debugging
    pub fn get_ai_behavior_description(entity: &GameEntity) -> String {
        match entity.ai_state {
            AIState::Idle => "Wandering peacefully".to_string(),
            AIState::Hostile => "Hunting for prey".to_string(),
            AIState::Fleeing => "Fleeing in terror".to_string(),
            AIState::Dead => "Lifeless".to_string(),
        }
    }

    /// Update AI decision making based on environmental factors
    pub fn update_ai_decisions(
        entities: &mut Vec<GameEntity>,
        is_day: bool,
        player_reputation: f32,
    ) {
        for entity in entities.iter_mut() {
            match entity.entity_type {
                EntityType::HostileInfected => {
                    // More aggressive at night
                    if !is_day {
                        // Increase detection range at night
                        if matches!(entity.ai_state, AIState::Idle) {
                            entity.ai_state = AIState::Hostile;
                        }
                    }
                }
                EntityType::Animal => {
                    // More fearful based on player reputation
                    if player_reputation > 0.5 {
                        if matches!(entity.ai_state, AIState::Idle) {
                            entity.ai_state = AIState::Fleeing;
                        }
                    }
                }
                EntityType::ClanLeader(_) | EntityType::ClanMember(_) => {
                    // Clan members react to player reputation
                    // This could affect their willingness to interact
                }
                _ => {}
            }
        }
    }
}

/// AI update structure for batching changes
#[derive(Debug)]
struct AIUpdate {
    entity_id: u32,
    new_velocity: Velocity,
    new_facing_direction: Option<f32>,
    should_attack: bool,
}

/// AI personality traits that affect behavior
#[derive(Debug, Clone)]
pub struct AIPersonality {
    pub aggression: f32,      // 0.0 to 1.0
    pub fear_threshold: f32,  // 0.0 to 1.0
    pub detection_range: f32, // Multiplier for base detection range
    pub loyalty: f32,         // 0.0 to 1.0 (for clan members)
}

impl Default for AIPersonality {
    fn default() -> Self {
        Self {
            aggression: 0.5,
            fear_threshold: 0.5,
            detection_range: 1.0,
            loyalty: 0.5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_entity(id: u32, entity_type: EntityType, ai_state: AIState) -> GameEntity {
        GameEntity {
            id,
            position: Position { x: 100.0, y: 100.0 },
            velocity: Some(Velocity { x: 0.0, y: 0.0 }),
            entity_type,
            health: Some(Health {
                current: 50.0,
                max: 50.0,
            }),
            combat_stats: Some(CombatStats::new(10.0, 5.0)),
            ai_state,
            blood_meter: None,
            vampire_abilities: None,
            shelter: None,
            shelter_occupancy: None,
            color: WHITE,
        }
    }

    #[test]
    fn test_calculate_distance() {
        let pos1 = Position { x: 0.0, y: 0.0 };
        let pos2 = Position { x: 3.0, y: 4.0 };

        let distance = AISystem::calculate_distance(&pos1, &pos2);
        assert_eq!(distance, 5.0);
    }

    #[test]
    fn test_normalize_direction() {
        let (dx, dy) = AISystem::normalize_direction(3.0, 4.0);
        assert!((dx - 0.6).abs() < 0.01);
        assert!((dy - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_should_initiate_combat() {
        let entity = create_test_entity(1, EntityType::HostileInfected, AIState::Hostile);
        let player_pos = Position { x: 120.0, y: 120.0 };

        // Close enough for combat
        assert!(AISystem::should_initiate_combat(&entity, &player_pos, 1.0));

        // Too far for combat
        let far_pos = Position { x: 200.0, y: 200.0 };
        assert!(!AISystem::should_initiate_combat(&entity, &far_pos, 1.0));
    }

    #[test]
    fn test_get_ai_behavior_description() {
        let entity = create_test_entity(1, EntityType::HostileInfected, AIState::Hostile);
        let description = AISystem::get_ai_behavior_description(&entity);
        assert_eq!(description, "Hunting for prey");
    }
}

/// AI system statistics for monitoring performance
#[derive(Debug, Clone)]
pub struct AIStatistics {
    pub cached_entities: usize,
    pub spatial_grid_cells: usize,
    pub ai_range: f32,
    pub performance_mode: bool,
    pub frame_counter: u64,
    pub memory_usage: usize,
}

impl Default for AISystem {
    fn default() -> Self {
        Self::new()
    }
}
