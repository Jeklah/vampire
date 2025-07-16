//! AI System Module
//!
//! Handles NPC behavior, decision making, and AI-driven combat logic.
//! This system manages different AI states and behaviors for non-player entities.

use crate::components::*;
use macroquad::prelude::*;

/// AI system responsible for NPC behavior and decision making
pub struct AISystem;

impl AISystem {
    /// Update AI for all entities
    pub fn update_all_ai(entities: &mut Vec<GameEntity>, player_id: u32, delta_time: f32) {
        let player_pos = Self::get_player_position(entities, player_id);

        // Collect AI updates to avoid borrowing issues
        let mut ai_updates = Vec::new();

        for entity in entities.iter() {
            if entity.id == player_id {
                continue;
            }

            if let Some(health) = &entity.health {
                if health.current <= 0.0 || matches!(entity.ai_state, AIState::Dead) {
                    continue;
                }
            }

            match entity.ai_state {
                AIState::Hostile => {
                    if let Some(update) = Self::update_hostile_ai(entity, &player_pos, delta_time) {
                        ai_updates.push(update);
                    }
                }
                AIState::Fleeing => {
                    if let Some(update) = Self::update_fleeing_ai(entity, &player_pos, delta_time) {
                        ai_updates.push(update);
                    }
                }
                AIState::Idle => {
                    if let Some(update) = Self::update_idle_ai(entity, &player_pos) {
                        ai_updates.push(update);
                    }
                }
                AIState::Dead => {
                    // Dead entities don't need AI updates
                }
            }
        }

        // Apply AI updates
        Self::apply_ai_updates(entities, ai_updates);
    }

    /// Get the player's current position
    fn get_player_position(entities: &[GameEntity], player_id: u32) -> Option<Position> {
        entities
            .iter()
            .find(|e| e.id == player_id)
            .map(|player| player.position)
    }

    /// Update hostile AI behavior
    fn update_hostile_ai(
        entity: &GameEntity,
        player_pos: &Option<Position>,
        _delta_time: f32,
    ) -> Option<AIUpdate> {
        if let Some(player_pos) = player_pos {
            let distance = Self::calculate_distance(&entity.position, player_pos);

            // Detection range for hostile entities
            let detection_range = 200.0;
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

                    let speed = 53.0; // Slightly slower than player
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
        } else {
            None
        }
    }

    /// Update fleeing AI behavior
    fn update_fleeing_ai(
        entity: &GameEntity,
        player_pos: &Option<Position>,
        _delta_time: f32,
    ) -> Option<AIUpdate> {
        if let Some(player_pos) = player_pos {
            let distance = Self::calculate_distance(&entity.position, player_pos);
            let flee_range = 150.0;

            if distance < flee_range {
                // Flee away from player
                let direction = Self::normalize_direction(
                    entity.position.x - player_pos.x, // Opposite direction
                    entity.position.y - player_pos.y,
                );

                let speed = 70.0; // Faster when fleeing
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
        } else {
            None
        }
    }

    /// Update idle AI behavior
    fn update_idle_ai(entity: &GameEntity, player_pos: &Option<Position>) -> Option<AIUpdate> {
        if let Some(player_pos) = player_pos {
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
        }

        None
    }

    /// Apply AI updates to entities
    fn apply_ai_updates(entities: &mut Vec<GameEntity>, updates: Vec<AIUpdate>) {
        for update in updates {
            if let Some(entity) = entities.iter_mut().find(|e| e.id == update.entity_id) {
                // Update velocity and position
                entity.velocity = Some(update.new_velocity);
                if let Some(velocity) = &entity.velocity {
                    entity.position.x += velocity.x * (1.0 / 60.0); // Assume 60 FPS
                    entity.position.y += velocity.y * (1.0 / 60.0);
                }

                // Update facing direction
                // Note: facing_direction field removed from GameEntity
                // Facing direction now calculated from velocity when needed

                // Keep entities within world bounds
                entity.position.x = entity.position.x.clamp(0.0, 1600.0);
                entity.position.y = entity.position.y.clamp(600.0, 1200.0);

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
