//! Player System Module
//!
//! Handles all player-specific logic including movement, actions, and abilities.
//! This system manages player input processing, movement updates, and action execution.

use crate::components::*;
use crate::InputHandler;
use macroquad::prelude::*;

/// Player system responsible for player-specific logic and actions
pub struct PlayerSystem;

impl PlayerSystem {
    /// Handle player input and execute corresponding actions
    pub fn handle_input(
        entities: &mut Vec<GameEntity>,
        input_handler: &InputHandler,
        player_id: u32,
        game_time: f32,
    ) {
        // Player actions
        // Feeding is now handled directly in GameState

        if input_handler.is_key_just_pressed(KeyCode::E) {
            Self::attempt_interaction(entities, player_id);
        }

        if input_handler.is_key_just_pressed(KeyCode::Space) {
            Self::attempt_attack(entities, player_id, game_time);
        }
    }

    /// Update player movement based on input
    pub fn update_movement(
        entities: &mut Vec<GameEntity>,
        input_handler: &InputHandler,
        player_id: u32,
        is_day: bool,
        delta_time: f32,
    ) {
        if let Some(player) = entities.iter_mut().find(|e| e.id == player_id) {
            let mut move_x = 0.0;
            let mut move_y = 0.0;

            // Get movement input
            if input_handler.is_key_pressed(KeyCode::W) {
                move_y = -1.0;
            }
            if input_handler.is_key_pressed(KeyCode::S) {
                move_y = 1.0;
            }
            if input_handler.is_key_pressed(KeyCode::A) {
                move_x = -1.0;
            }
            if input_handler.is_key_pressed(KeyCode::D) {
                move_x = 1.0;
            }

            // Normalize diagonal movement
            if move_x != 0.0 && move_y != 0.0 {
                move_x *= 0.707; // 1/sqrt(2)
                move_y *= 0.707;
            }

            // Calculate speed with ability modifiers
            let base_speed = 260.0;
            let ability_speed_modifier = player
                .vampire_abilities
                .as_ref()
                .map(|abilities| abilities.speed)
                .unwrap_or(1.0);

            // Apply sunlight penalty during day
            let sunlight_penalty = if is_day { 0.5 } else { 1.0 };

            let final_speed = base_speed * ability_speed_modifier * sunlight_penalty;

            // Update velocity
            if let Some(velocity) = &mut player.velocity {
                velocity.x = move_x * final_speed;
                velocity.y = move_y * final_speed;
            }

            // Update position
            if let Some(velocity) = &player.velocity {
                player.position.x += velocity.x * delta_time;
                player.position.y += velocity.y * delta_time;
            }

            // Update facing direction
            // Facing direction calculation removed as field no longer exists
            // Direction is now calculated from velocity when needed for rendering

            // Keep player within world bounds
            player.position.x = player.position.x.clamp(0.0, 1600.0);
            player.position.y = player.position.y.clamp(600.0, 1200.0); // Can't go above ground level
        }
    }

    /// Attempt to feed on a nearby entity
    pub fn attempt_feeding(
        entities: &mut Vec<GameEntity>,
        player_id: u32,
        debug_messages: &mut Vec<String>,
    ) -> Option<Position> {
        debug_messages.push("Attempting to feed...".to_string());
        let player_index = entities.iter().position(|e| e.id == player_id);
        let player_pos = if let Some(idx) = player_index {
            entities[idx].position
        } else {
            debug_messages.push("No player entity found for feeding!".to_string());
            return None;
        };

        let feed_range = 50.0;

        // Find the first valid target index
        debug_messages.push(format!(
            "Searching for feeding targets within range {}...",
            feed_range
        ));
        let target_index = entities.iter().enumerate().find_map(|(idx, entity)| {
            if entity.id == player_id {
                return None;
            }
            let distance = Self::calculate_distance(&player_pos, &entity.position);
            let has_health = entity.health.as_ref().map_or(false, |h| h.current > 0.0);
            debug_messages.push(format!(
                "Checking entity {:?} at ({:.1}, {:.1}), distance: {:.1}, has_health: {}, in_range: {}",
                entity.entity_type, entity.position.x, entity.position.y, distance, has_health, distance <= feed_range
            ));
            if distance <= feed_range && has_health {
                Some(idx)
            } else {
                None
            }
        });
        debug_messages.push(format!("Target index found: {:?}", target_index));

        if let (Some(player_idx), Some(target_idx)) = (player_index, target_index) {
            debug_messages.push(format!(
                "Found player at index {} and target at index {}",
                player_idx, target_idx
            ));
            // Safe split for double mutable borrow
            let (first, second) = if player_idx < target_idx {
                debug_messages.push("Player index < target index, splitting at target".to_string());
                let (first, second) = entities.split_at_mut(target_idx);
                (&mut first[player_idx], &mut second[0])
            } else if player_idx > target_idx {
                debug_messages.push("Player index > target index, splitting at player".to_string());
                let (first, second) = entities.split_at_mut(player_idx);
                (&mut second[0], &mut first[target_idx])
            } else {
                // Should never happen: player cannot be their own target
                debug_messages.push("ERROR: Player and target have same index!".to_string());
                return None;
            };

            if let Some(health) = &mut second.health {
                debug_messages.push(format!(
                    "Target found for feeding: {:?} at ({}, {}), health: {}",
                    second.entity_type, second.position.x, second.position.y, health.current
                ));
                let target_pos = second.position;
                let blood_amount = health.current * 0.6;
                health.current = 0.0; // Feeding is lethal
                second.ai_state = AIState::Dead;

                // Apply benefits to player
                if let Some(blood_meter) = &mut first.blood_meter {
                    blood_meter.current =
                        (blood_meter.current + blood_amount).min(blood_meter.maximum);
                }
                if let Some(player_health) = &mut first.health {
                    player_health.current =
                        (player_health.current + blood_amount * 0.2).min(player_health.max);
                }
                debug_messages.push(format!(
                    "Feeding successful! Returning target position: ({}, {})",
                    target_pos.x, target_pos.y
                ));
                return Some(target_pos);
            } else {
                debug_messages.push("ERROR: Target has no health component!".to_string());
            }
        } else {
            debug_messages.push("No valid target found for feeding".to_string());
        }
        None
    }

    /// Execute feeding on a target entity
    fn feed_on_target(entities: &mut Vec<GameEntity>, player_id: u32, target_id: u32) -> bool {
        let blood_gained = {
            if let Some(target) = entities.iter_mut().find(|e| e.id == target_id) {
                if let Some(health) = &mut target.health {
                    let blood_amount = health.current * 0.6;
                    health.current = 0.0; // Feeding is lethal
                    target.ai_state = AIState::Dead;
                    blood_amount
                } else {
                    0.0
                }
            } else {
                0.0
            }
        };

        // Apply benefits to player
        if let Some(player) = entities.iter_mut().find(|e| e.id == player_id) {
            // Restore blood
            if let Some(blood_meter) = &mut player.blood_meter {
                blood_meter.current = (blood_meter.current + blood_gained).min(blood_meter.maximum);
            }

            // Heal player
            if let Some(health) = &mut player.health {
                health.current = (health.current + blood_gained * 0.3).min(health.max);
            }

            // Improve abilities
            if let Some(abilities) = &mut player.vampire_abilities {
                abilities.strength += 0.01;
                abilities.speed += 0.005;
                abilities.blood_sense += 0.02;
            }

            return true;
        }

        false
    }

    /// Attempt to attack a nearby hostile entity
    pub fn attempt_attack(
        entities: &mut Vec<GameEntity>,
        player_id: u32,
        game_time: f32,
    ) -> Option<Position> {
        let player_index = entities.iter().position(|e| e.id == player_id);
        let player_pos = if let Some(idx) = player_index {
            entities[idx].position
        } else {
            return None;
        };

        let attack_range = 60.0;
        // Find the first valid target index
        let target_index = entities.iter().position(|entity| {
            entity.id != player_id
                && matches!(
                    entity.entity_type,
                    EntityType::HostileInfected | EntityType::Animal
                )
                && Self::calculate_distance(&player_pos, &entity.position) <= attack_range
                && entity.health.as_ref().map_or(false, |h| h.current > 0.0)
        });

        if let (Some(player_idx), Some(target_idx)) = (player_index, target_index) {
            // Safe split for double mutable borrow
            let (first, second) = if player_idx < target_idx {
                let (first, second) = entities.split_at_mut(target_idx);
                (&mut first[player_idx], &mut second[0])
            } else if player_idx > target_idx {
                let (first, second) = entities.split_at_mut(player_idx);
                (&mut second[0], &mut first[target_idx])
            } else {
                // Should never happen: player cannot be their own target
                return None;
            };

            // Extract attack power and check cooldown
            let attack_power = if let Some(combat_stats) = &first.combat_stats {
                if combat_stats.can_attack(game_time) {
                    combat_stats.attack_power
                } else {
                    return None; // Still on cooldown
                }
            } else {
                20.0 // Default attack power
            };

            if let Some(health) = &mut second.health {
                println!(
                    "Target found for attack: {:?} at ({}, {}), health: {}",
                    second.entity_type, second.position.x, second.position.y, health.current
                );
                let target_pos = second.position;
                // Apply damage to target
                let defense = second.combat_stats.as_ref().map_or(0.0, |cs| cs.defense);
                let final_damage = (attack_power - defense).max(5.0); // Minimum damage

                health.current -= final_damage;
                health.current = health.current.max(0.0);

                if health.current <= 0.0 {
                    second.ai_state = AIState::Dead;
                }

                // Update player attack cooldown
                if let Some(combat_stats) = &mut first.combat_stats {
                    combat_stats.last_attack_time = game_time;
                }

                return Some(target_pos);
            }
        }
        None
    }

    /// Execute an attack on a target entity
    fn attack_entity(
        entities: &mut Vec<GameEntity>,
        player_id: u32,
        target_id: u32,
        game_time: f32,
    ) -> bool {
        // Get player attack power and check cooldown
        let attack_power = if let Some(player) = entities.iter().find(|e| e.id == player_id) {
            if let Some(combat_stats) = &player.combat_stats {
                if combat_stats.can_attack(game_time) {
                    combat_stats.attack_power
                } else {
                    return false; // Still on cooldown
                }
            } else {
                20.0 // Default attack power
            }
        } else {
            return false;
        };

        // Apply damage to target
        let mut target_killed = false;
        if let Some(target) = entities.iter_mut().find(|e| e.id == target_id) {
            if let Some(health) = &mut target.health {
                let defense = target.combat_stats.as_ref().map_or(0.0, |cs| cs.defense);
                let final_damage = (attack_power - defense).max(5.0); // Minimum damage

                health.current -= final_damage;
                health.current = health.current.max(0.0);

                if health.current <= 0.0 {
                    target.ai_state = AIState::Dead;
                    target_killed = true;
                }
            }
        }

        // Update player attack cooldown
        if let Some(player) = entities.iter_mut().find(|e| e.id == player_id) {
            if let Some(combat_stats) = &mut player.combat_stats {
                combat_stats.last_attack_time = game_time;
            }
        }

        target_killed
    }

    /// Attempt to interact with nearby entities (clan leaders, NPCs)
    pub fn attempt_interaction(entities: &mut Vec<GameEntity>, player_id: u32) -> Option<String> {
        let player_pos = if let Some(player) = entities.iter().find(|e| e.id == player_id) {
            player.position
        } else {
            return None;
        };

        let interact_range = 70.0;

        // Find nearby clan leaders
        for entity in entities.iter() {
            if let EntityType::ClanLeader(clan_name) = &entity.entity_type {
                let distance = Self::calculate_distance(&player_pos, &entity.position);
                if distance <= interact_range {
                    return Some(clan_name.clone());
                }
            }
        }

        None
    }

    /// Apply sunlight damage to the player during daytime
    pub fn apply_sunlight_damage(
        entities: &mut Vec<GameEntity>,
        player_id: u32,
        sunlight_intensity: f32,
        delta_time: f32,
    ) -> f32 {
        if let Some(player) = entities.iter_mut().find(|e| e.id == player_id) {
            if let Some(health) = &mut player.health {
                let damage = 3.0 * sunlight_intensity * delta_time;
                health.current = (health.current - damage).max(0.0);
                return damage;
            }
        }
        0.0
    }

    /// Get player's current status information
    pub fn get_player_status(entities: &[GameEntity], player_id: u32) -> Option<PlayerStatus> {
        entities
            .iter()
            .find(|e| e.id == player_id)
            .map(|player| PlayerStatus {
                health: player.health.clone(),
                blood_meter: player.blood_meter.clone(),
                abilities: player.vampire_abilities.clone(),
                position: player.position,
                facing_direction: 0.0, // Default facing direction
                is_alive: player.health.as_ref().map_or(false, |h| h.current > 0.0),
            })
    }

    /// Check if player can perform an action (has enough blood, not dead, etc.)
    pub fn can_perform_action(
        entities: &[GameEntity],
        player_id: u32,
        action: PlayerAction,
    ) -> bool {
        if let Some(player) = entities.iter().find(|e| e.id == player_id) {
            // Check if player is alive
            if let Some(health) = &player.health {
                if health.current <= 0.0 {
                    return false;
                }
            }

            // Check blood requirements for specific actions
            match action {
                PlayerAction::Feed => true, // No blood cost for feeding
                PlayerAction::Attack => {
                    // Check if player has enough blood for combat
                    if let Some(blood_meter) = &player.blood_meter {
                        blood_meter.current > 10.0
                    } else {
                        false
                    }
                }
                PlayerAction::Interact => true, // No blood cost for interaction
                PlayerAction::SpecialAbility => {
                    // Check if player has enough blood for special abilities
                    if let Some(blood_meter) = &player.blood_meter {
                        blood_meter.current > 20.0
                    } else {
                        false
                    }
                }
            }
        } else {
            false
        }
    }

    /// Calculate distance between two positions
    fn calculate_distance(pos1: &Position, pos2: &Position) -> f32 {
        ((pos1.x - pos2.x).powi(2) + (pos1.y - pos2.y).powi(2)).sqrt()
    }

    /// Level up player abilities based on experience
    pub fn level_up_abilities(
        entities: &mut Vec<GameEntity>,
        player_id: u32,
        experience_type: ExperienceType,
    ) {
        if let Some(player) = entities.iter_mut().find(|e| e.id == player_id) {
            if let Some(abilities) = &mut player.vampire_abilities {
                match experience_type {
                    ExperienceType::Combat => {
                        abilities.strength += 0.05;
                    }
                    ExperienceType::Feeding => {
                        abilities.blood_sense += 0.03;
                    }
                    ExperienceType::Stealth => {
                        abilities.shadow_movement += 0.02;
                    }
                    ExperienceType::Survival => {
                        abilities.speed += 0.02;
                    }
                }
            }
        }
    }
}

/// Player status information
#[derive(Debug, Clone)]
pub struct PlayerStatus {
    pub health: Option<Health>,
    pub blood_meter: Option<BloodMeter>,
    pub abilities: Option<VampireAbilities>,
    pub position: Position,
    pub facing_direction: f32,
    pub is_alive: bool,
}

/// Types of actions the player can perform
#[derive(Debug, Clone, Copy)]
pub enum PlayerAction {
    Feed,
    Attack,
    Interact,
    SpecialAbility,
}

/// Types of experience for leveling up abilities
#[derive(Debug, Clone, Copy)]
pub enum ExperienceType {
    Combat,
    Feeding,
    Stealth,
    Survival,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_player() -> GameEntity {
        GameEntity {
            id: 0,
            position: Position { x: 100.0, y: 100.0 },
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
            shelter_occupancy: None,
            color: RED,
        }
    }

    #[test]
    fn test_can_perform_action() {
        let entities = vec![create_test_player()];

        assert!(PlayerSystem::can_perform_action(
            &entities,
            0,
            PlayerAction::Feed
        ));
        assert!(PlayerSystem::can_perform_action(
            &entities,
            0,
            PlayerAction::Attack
        ));
        assert!(PlayerSystem::can_perform_action(
            &entities,
            0,
            PlayerAction::Interact
        ));
    }

    #[test]
    fn test_get_player_status() {
        let entities = vec![create_test_player()];

        let status = PlayerSystem::get_player_status(&entities, 0).unwrap();
        assert!(status.is_alive);
        assert!(status.health.is_some());
        assert!(status.blood_meter.is_some());
    }

    #[test]
    fn test_calculate_distance() {
        let pos1 = Position { x: 0.0, y: 0.0 };
        let pos2 = Position { x: 3.0, y: 4.0 };

        let distance = PlayerSystem::calculate_distance(&pos1, &pos2);
        assert_eq!(distance, 5.0); // 3-4-5 triangle
    }
}
