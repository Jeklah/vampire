//! Blood System Module
//!
//! Handles blood mechanics, feeding logic, blood meter updates, and starvation effects.
//! This system manages the core vampire survival mechanics around blood consumption.

use crate::components::*;
use macroquad::prelude::*;

/// Blood system responsible for blood mechanics and vampire survival
pub struct BloodSystem;

impl BloodSystem {
    /// Update blood system for all entities
    pub fn update_blood_system(
        entities: &mut Vec<GameEntity>,
        is_day: bool,
        sunlight_intensity: f32,
        delta_time: f32,
    ) {
        for entity in entities.iter_mut() {
            if let Some(blood_meter) = &mut entity.blood_meter {
                // Drain blood over time
                Self::update_blood_drain(blood_meter, delta_time);

                // Sunlight damage is now handled by the new shelter-aware function
                // after the main entity loop to avoid borrowing issues

                // Apply starvation damage when blood is low
                Self::apply_starvation_damage(entity, delta_time);
            }
        }

        // Apply sunlight damage with shelter protection (separate pass to avoid borrowing issues)
        if is_day && sunlight_intensity > 0.0 {
            Self::apply_sunlight_damage_with_shelter(entities, sunlight_intensity, delta_time);
        }
    }

    /// Update blood drain over time
    fn update_blood_drain(blood_meter: &mut BloodMeter, delta_time: f32) {
        blood_meter.current -= blood_meter.drain_rate * delta_time;
        blood_meter.current = blood_meter.current.max(0.0);
    }

    /// Apply sunlight damage to vampires (modified to work with shelter system)
    fn apply_sunlight_damage(entity: &mut GameEntity, sunlight_intensity: f32, delta_time: f32) {
        if let Some(health) = &mut entity.health {
            let base_damage = 3.0 * sunlight_intensity * delta_time;
            health.current = (health.current - base_damage).max(0.0);
        }
    }

    /// Apply sunlight damage with shelter protection consideration
    pub fn apply_sunlight_damage_with_shelter(
        entities: &mut Vec<GameEntity>,
        sunlight_intensity: f32,
        delta_time: f32,
    ) {
        // Collect entity IDs and base damage for entities with blood meters using iterator
        let damage_calculations: Vec<(u32, f32)> = entities
            .iter()
            .filter(|entity| entity.blood_meter.is_some() && entity.health.is_some())
            .map(|entity| (entity.id, 3.0 * sunlight_intensity * delta_time))
            .collect();

        // Apply calculated damage
        for (entity_id, base_damage) in damage_calculations {
            let protected_damage = crate::systems::ShelterSystem::calculate_shelter_protection(
                entities,
                entity_id,
                base_damage,
            );

            if let Some(entity) = entities.iter_mut().find(|e| e.id == entity_id) {
                if let Some(health) = &mut entity.health {
                    health.current = (health.current - protected_damage).max(0.0);
                }
            }
        }
    }

    /// Apply starvation damage when blood is critically low
    fn apply_starvation_damage(entity: &mut GameEntity, delta_time: f32) {
        if let Some(blood_meter) = &entity.blood_meter {
            if blood_meter.current < blood_meter.maximum * 0.2 {
                if let Some(health) = &mut entity.health {
                    health.current -= 2.0 * delta_time;
                    health.current = health.current.max(0.0);
                }
            }
        }
    }

    /// Create blood particle effects at a position
    pub fn create_blood_particles(
        blood_particles: &mut Vec<BloodParticle>,
        x: f32,
        y: f32,
        intensity: u32,
        debug_messages: &mut Vec<String>,
    ) {
        debug_messages.push(format!(
            "Creating {} blood particles at ({}, {})",
            intensity, x, y
        ));
        (0..intensity).for_each(|_| {
            blood_particles.push(BloodParticle::new(x, y));
        });
        debug_messages.push(format!(
            "Blood particles vector now has {} particles",
            blood_particles.len()
        ));
    }

    /// Calculate blood gain from feeding on a specific entity type
    pub fn calculate_blood_gain(target_entity: &GameEntity) -> f32 {
        match target_entity.entity_type {
            EntityType::Animal => {
                // Animals provide moderate blood
                if let Some(health) = &target_entity.health {
                    health.current * 0.6
                } else {
                    0.0
                }
            }
            EntityType::HostileInfected => {
                // Infected provide less blood but still viable
                if let Some(health) = &target_entity.health {
                    health.current * 0.4
                } else {
                    0.0
                }
            }
            EntityType::ClanMember(_) => {
                // Clan members provide good blood
                if let Some(health) = &target_entity.health {
                    health.current * 0.8
                } else {
                    0.0
                }
            }
            EntityType::ClanLeader(_) => {
                // Clan leaders provide excellent blood
                if let Some(health) = &target_entity.health {
                    health.current * 1.0
                } else {
                    0.0
                }
            }
            EntityType::Player => {
                // Players can't feed on themselves
                0.0
            }
            EntityType::Shelter => {
                // Can't feed on shelters
                0.0
            }
        }
    }

    /// Check if an entity is a valid feeding target
    pub fn is_valid_feeding_target(entity: &GameEntity) -> bool {
        // Must be alive
        if let Some(health) = &entity.health {
            if health.current <= 0.0 {
                return false;
            }
        } else {
            return false;
        }

        // Must not already be dead
        if matches!(entity.ai_state, AIState::Dead) {
            return false;
        }

        // Check entity type validity
        match entity.entity_type {
            EntityType::Animal => true,
            EntityType::HostileInfected => true,
            EntityType::ClanMember(_) => true,
            EntityType::ClanLeader(_) => true,
            EntityType::Player => false, // Players can't feed on themselves
            EntityType::Shelter => false, // Can't feed on shelters
        }
    }

    /// Apply feeding effects to the vampire
    pub fn apply_feeding_effects(
        vampire: &mut GameEntity,
        blood_gained: f32,
        feeding_count: &mut u32,
    ) -> bool {
        if blood_gained <= 0.0 {
            return false;
        }

        // Restore blood
        if let Some(blood_meter) = &mut vampire.blood_meter {
            blood_meter.current = (blood_meter.current + blood_gained).min(blood_meter.maximum);
        }

        // Heal vampire based on blood consumed
        if let Some(health) = &mut vampire.health {
            let healing = blood_gained * 0.3;
            health.current = (health.current + healing).min(health.max);
        }

        // Improve vampire abilities from feeding experience
        Self::improve_abilities_from_feeding(vampire, blood_gained);

        // Increment feeding counter
        *feeding_count += 1;

        true
    }

    /// Improve vampire abilities based on feeding experience
    fn improve_abilities_from_feeding(vampire: &mut GameEntity, blood_gained: f32) {
        if let Some(abilities) = &mut vampire.vampire_abilities {
            // Scale improvements based on blood gained
            let improvement_factor = (blood_gained / 100.0).min(1.0);

            abilities.strength += 0.01 * improvement_factor;
            abilities.speed += 0.005 * improvement_factor;
            abilities.blood_sense += 0.02 * improvement_factor;

            // Cap abilities to prevent infinite growth
            abilities.strength = abilities.strength.min(3.0);
            abilities.speed = abilities.speed.min(2.5);
            abilities.blood_sense = abilities.blood_sense.min(5.0);
            abilities.shadow_movement = abilities.shadow_movement.min(3.0);
        }
    }

    /// Check blood meter status and return warnings
    pub fn check_blood_status(entity: &GameEntity) -> BloodStatus {
        if let Some(blood_meter) = &entity.blood_meter {
            let percentage = blood_meter.current / blood_meter.maximum;

            if percentage <= 0.0 {
                BloodStatus::Empty
            } else if percentage <= 0.1 {
                BloodStatus::Critical
            } else if percentage <= 0.2 {
                BloodStatus::Low
            } else if percentage <= 0.5 {
                BloodStatus::Moderate
            } else if percentage <= 0.8 {
                BloodStatus::Good
            } else {
                BloodStatus::Full
            }
        } else {
            BloodStatus::None
        }
    }

    /// Calculate blood drain rate based on various factors
    pub fn calculate_blood_drain_rate(
        entity: &GameEntity,
        is_day: bool,
        activity_level: ActivityLevel,
    ) -> f32 {
        let mut base_drain = if let Some(blood_meter) = &entity.blood_meter {
            blood_meter.drain_rate
        } else {
            1.0
        };

        // Increase drain during day (vampires are less efficient in sunlight)
        if is_day {
            base_drain *= 1.5;
        }

        // Adjust for activity level
        match activity_level {
            ActivityLevel::Resting => base_drain * 0.5,
            ActivityLevel::Normal => base_drain,
            ActivityLevel::Active => base_drain * 1.2,
            ActivityLevel::Combat => base_drain * 2.0,
            ActivityLevel::UsingAbilities => base_drain * 2.5,
        }
    }

    /// Update blood particle effects
    pub fn update_blood_particles(blood_particles: &mut Vec<BloodParticle>, delta_time: f32) {
        blood_particles.retain_mut(|particle| particle.update(delta_time));
    }

    /// Get blood efficiency based on vampire abilities
    pub fn get_blood_efficiency(abilities: &VampireAbilities) -> f32 {
        // Higher blood sense means more efficient feeding
        1.0 + (abilities.blood_sense * 0.1)
    }

    /// Check if vampire needs urgent feeding
    pub fn needs_urgent_feeding(entity: &GameEntity) -> bool {
        matches!(
            Self::check_blood_status(entity),
            BloodStatus::Critical | BloodStatus::Empty
        )
    }

    /// Calculate days survived without feeding (for achievements/scoring)
    pub fn calculate_survival_score(
        feeding_count: u32,
        day_count: u32,
        kills: u32,
    ) -> SurvivalScore {
        let feeding_efficiency = if day_count > 0 {
            feeding_count as f32 / day_count as f32
        } else {
            0.0
        };

        let combat_effectiveness = if feeding_count > 0 {
            kills as f32 / feeding_count as f32
        } else {
            0.0
        };

        SurvivalScore {
            days_survived: day_count,
            total_feedings: feeding_count,
            total_kills: kills,
            feeding_efficiency,
            combat_effectiveness,
            overall_score: (day_count as f32 * 10.0)
                + (feeding_efficiency * 100.0)
                + (combat_effectiveness * 50.0),
        }
    }
}

/// Blood meter status levels
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BloodStatus {
    None,     // Entity has no blood meter
    Empty,    // 0% blood
    Critical, // 1-10% blood
    Low,      // 11-20% blood
    Moderate, // 21-50% blood
    Good,     // 51-80% blood
    Full,     // 81-100% blood
}

impl BloodStatus {
    /// Get the color associated with this blood status for UI display
    pub fn get_color(&self) -> Color {
        match self {
            BloodStatus::None => GRAY,
            BloodStatus::Empty => Color::new(0.5, 0.0, 0.0, 1.0),
            BloodStatus::Critical => Color::new(0.8, 0.0, 0.0, 1.0),
            BloodStatus::Low => Color::new(1.0, 0.3, 0.0, 1.0),
            BloodStatus::Moderate => Color::new(1.0, 0.8, 0.0, 1.0),
            BloodStatus::Good => Color::new(0.0, 0.8, 0.2, 1.0),
            BloodStatus::Full => Color::new(0.0, 1.0, 0.0, 1.0),
        }
    }

    /// Get a descriptive message for this blood status
    pub fn get_message(&self) -> &'static str {
        match self {
            BloodStatus::None => "No blood meter",
            BloodStatus::Empty => "Blood depleted! Find sustenance immediately!",
            BloodStatus::Critical => "Blood critically low! You must feed soon!",
            BloodStatus::Low => "Blood running low. Seek nourishment.",
            BloodStatus::Moderate => "Blood levels acceptable.",
            BloodStatus::Good => "Blood levels good.",
            BloodStatus::Full => "Blood reserves full.",
        }
    }
}

/// Activity levels that affect blood drain
#[derive(Debug, Clone, Copy)]
pub enum ActivityLevel {
    Resting,        // Not moving, minimal drain
    Normal,         // Walking around normally
    Active,         // Running, frequent movement
    Combat,         // Fighting, high energy expenditure
    UsingAbilities, // Using vampire powers, highest drain
}

/// Survival score calculation results
#[derive(Debug, Clone)]
pub struct SurvivalScore {
    pub days_survived: u32,
    pub total_feedings: u32,
    pub total_kills: u32,
    pub feeding_efficiency: f32,
    pub combat_effectiveness: f32,
    pub overall_score: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_vampire() -> GameEntity {
        GameEntity {
            id: 0,
            position: Position { x: 0.0, y: 0.0 },
            velocity: Some(Velocity { x: 0.0, y: 0.0 }),
            entity_type: EntityType::Player,
            health: Some(Health {
                current: 100.0,
                max: 100.0,
            }),
            combat_stats: None,
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

    fn create_test_animal() -> GameEntity {
        GameEntity {
            id: 1,
            position: Position { x: 0.0, y: 0.0 },
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
        }
    }

    #[test]
    fn test_blood_status() {
        let mut vampire = create_test_vampire();

        // Test normal blood level
        assert_eq!(
            BloodSystem::check_blood_status(&vampire),
            BloodStatus::Moderate
        );

        // Test critical blood level
        vampire.blood_meter.as_mut().unwrap().current = 5.0;
        assert_eq!(
            BloodSystem::check_blood_status(&vampire),
            BloodStatus::Critical
        );

        // Test full blood level
        vampire.blood_meter.as_mut().unwrap().current = 90.0;
        assert_eq!(BloodSystem::check_blood_status(&vampire), BloodStatus::Full);
    }

    #[test]
    fn test_valid_feeding_target() {
        let animal = create_test_animal();
        assert!(BloodSystem::is_valid_feeding_target(&animal));

        let vampire = create_test_vampire();
        assert!(!BloodSystem::is_valid_feeding_target(&vampire)); // Can't feed on player
    }

    #[test]
    fn test_blood_gain_calculation() {
        let animal = create_test_animal();
        let blood_gain = BloodSystem::calculate_blood_gain(&animal);
        assert!((blood_gain - 15.0).abs() < 0.001); // 25.0 * 0.6 with floating point tolerance
    }

    #[test]
    fn test_blood_drain_rate() {
        let vampire = create_test_vampire();

        let normal_rate =
            BloodSystem::calculate_blood_drain_rate(&vampire, false, ActivityLevel::Normal);
        assert_eq!(normal_rate, 1.0);

        let day_rate =
            BloodSystem::calculate_blood_drain_rate(&vampire, true, ActivityLevel::Normal);
        assert_eq!(day_rate, 1.5);

        let combat_rate =
            BloodSystem::calculate_blood_drain_rate(&vampire, false, ActivityLevel::Combat);
        assert_eq!(combat_rate, 2.0);
    }

    #[test]
    fn test_survival_score() {
        let score = BloodSystem::calculate_survival_score(10, 5, 8);
        assert_eq!(score.days_survived, 5);
        assert_eq!(score.total_feedings, 10);
        assert_eq!(score.total_kills, 8);
        assert_eq!(score.feeding_efficiency, 2.0); // 10/5
        assert_eq!(score.combat_effectiveness, 0.8); // 8/10
    }

    #[test]
    fn test_needs_urgent_feeding() {
        let mut vampire = create_test_vampire();

        // Normal blood level - no urgent need
        assert!(!BloodSystem::needs_urgent_feeding(&vampire));

        // Critical blood level - urgent need
        vampire.blood_meter.as_mut().unwrap().current = 5.0;
        assert!(BloodSystem::needs_urgent_feeding(&vampire));
    }
}
