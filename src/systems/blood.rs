//! Blood system for handling vampire blood mechanics
//!
//! This module contains the blood system that manages blood consumption,
//! regeneration, feeding mechanics, and blood-related effects.

use crate::components::*;
use crate::resources::*;
use hecs::World;
use macroquad::prelude::*;

/// Blood system that handles all blood-related mechanics
pub struct BloodSystem;

impl BloodSystem {
    /// Update blood-related mechanics for all entities
    pub fn update(world: &mut World, time: &TimeResource, delta_time: f32) {
        // Drain blood over time
        Self::drain_blood_over_time(world, delta_time);

        // Apply blood starvation effects
        Self::apply_starvation_effects(world, delta_time);

        // Apply sunlight damage to vampires
        Self::apply_sunlight_damage(world, time, delta_time);

        // Regenerate health for well-fed vampires
        Self::regenerate_health_from_blood(world, delta_time);
    }

    /// Drain blood from all entities with blood meters over time
    fn drain_blood_over_time(world: &mut World, delta_time: f32) {
        for (_, blood_meter) in world.query_mut::<&mut BloodMeter>() {
            let drain_amount = blood_meter.drain_rate * delta_time;
            blood_meter.current = (blood_meter.current - drain_amount).max(0.0);
        }
    }

    /// Apply negative effects when vampires are starving
    fn apply_starvation_effects(world: &mut World, delta_time: f32) {
        for (entity, (blood_meter, health)) in world.query_mut::<(&BloodMeter, &mut Health)>() {
            if blood_meter.is_starving() {
                // Take damage when starving
                let starvation_damage = 5.0 * delta_time;
                health.take_damage(starvation_damage);

                // Reduce vampire abilities when starving
                if let Ok(mut abilities) = world.get_mut::<VampireAbilities>(entity) {
                    let starvation_penalty = 0.5; // 50% penalty when starving
                    abilities.strength *= starvation_penalty;
                    abilities.speed *= starvation_penalty;
                }
            }
        }
    }

    /// Apply sunlight damage to vampires
    fn apply_sunlight_damage(world: &mut World, time: &TimeResource, delta_time: f32) {
        if !time.is_day {
            return;
        }

        let sunlight_intensity = time.get_sunlight_intensity();

        for (_, (vulnerability, health)) in
            world.query_mut::<(&SunlightVulnerability, &mut Health)>()
        {
            if vulnerability.in_sunlight {
                let damage = vulnerability.damage_rate * sunlight_intensity * delta_time;
                health.take_damage(damage);
            }
        }
    }

    /// Regenerate health for vampires with sufficient blood
    fn regenerate_health_from_blood(world: &mut World, delta_time: f32) {
        for (entity, (blood_meter, health)) in world.query_mut::<(&BloodMeter, &mut Health)>() {
            // Only regenerate if blood is above 50% and not at full health
            if blood_meter.blood_percentage() > 0.5 && health.current < health.maximum {
                let regen_rate = 2.0 * delta_time; // 2 HP per second
                health.heal(regen_rate);

                // Consume a small amount of blood for regeneration
                if let Ok(mut blood_meter_mut) = world.get_mut::<BloodMeter>(entity) {
                    blood_meter_mut.consume(0.5 * delta_time);
                }
            }
        }
    }

    /// Attempt to feed on a target entity
    pub fn feed_on_target(
        world: &mut World,
        vampire_entity: hecs::Entity,
        target_entity: hecs::Entity,
    ) -> bool {
        // Validate that both entities exist and vampire can feed
        if !Self::can_feed_on_target(world, vampire_entity, target_entity) {
            return false;
        }

        // Calculate blood amount to drain
        let blood_amount = Self::calculate_blood_gain(world, target_entity);

        if blood_amount <= 0.0 {
            return false;
        }

        // Drain target's health
        if let Ok(mut target_health) = world.get_mut::<Health>(target_entity) {
            target_health.take_damage(blood_amount * 2.0); // Feeding is lethal
        }

        // Add blood to vampire
        if let Ok(mut blood_meter) = world.get_mut::<BloodMeter>(vampire_entity) {
            blood_meter.add_blood(blood_amount);
        }

        // Heal vampire from feeding
        if let Ok(mut vampire_health) = world.get_mut::<Health>(vampire_entity) {
            vampire_health.heal(blood_amount * 0.3);
        }

        // Improve vampire abilities from feeding
        Self::improve_abilities_from_feeding(world, vampire_entity, blood_amount);

        // Handle feeding consequences (fear, reputation, etc.)
        Self::handle_feeding_consequences(world, vampire_entity, target_entity);

        true
    }

    /// Check if vampire can feed on target
    fn can_feed_on_target(
        world: &World,
        vampire_entity: hecs::Entity,
        target_entity: hecs::Entity,
    ) -> bool {
        // Check if vampire has blood meter
        if world.get::<BloodMeter>(vampire_entity).is_err() {
            return false;
        }

        // Check if target has health
        if let Ok(target_health) = world.get::<Health>(target_entity) {
            if !target_health.is_alive() {
                return false;
            }
        } else {
            return false;
        }

        // Check if target is already a vampire (can't feed on other vampires easily)
        if world.get::<BloodMeter>(target_entity).is_ok() {
            return false;
        }

        true
    }

    /// Calculate how much blood can be gained from a target
    fn calculate_blood_gain(world: &World, target_entity: hecs::Entity) -> f32 {
        if let Ok(health) = world.get::<Health>(target_entity) {
            // Base blood gain is proportional to target's current health
            let base_gain = health.current * 0.6;

            // Bonus for feeding on healthy targets
            let health_bonus = if health.health_percentage() > 0.8 {
                base_gain * 0.5
            } else {
                0.0
            };

            base_gain + health_bonus
        } else {
            0.0
        }
    }

    /// Improve vampire abilities from feeding
    fn improve_abilities_from_feeding(
        world: &mut World,
        vampire_entity: hecs::Entity,
        blood_amount: f32,
    ) {
        if let Ok(mut abilities) = world.get_mut::<VampireAbilities>(vampire_entity) {
            let improvement_factor = blood_amount * 0.01; // Small improvements per feeding

            abilities.strength += improvement_factor;
            abilities.speed += improvement_factor * 0.5;
            abilities.blood_sense += improvement_factor * 2.0;

            // Cap abilities to prevent infinite growth
            abilities.strength = abilities.strength.min(3.0);
            abilities.speed = abilities.speed.min(2.5);
            abilities.blood_sense = abilities.blood_sense.min(5.0);
        }
    }

    /// Handle consequences of feeding (reputation, fear, etc.)
    fn handle_feeding_consequences(
        world: &mut World,
        vampire_entity: hecs::Entity,
        target_entity: hecs::Entity,
    ) {
        // Check if target was part of a clan
        if let Ok(clan) = world.get::<Clan>(target_entity) {
            // Feeding on clan members has serious consequences
            // This would be handled by the clan system
        }

        // Increase player reputation as a fearsome predator
        if let Ok(mut player) = world.get_mut::<Player>(vampire_entity) {
            player.reputation += 0.1;
        }
    }

    /// Use blood to power vampire abilities
    pub fn use_blood_for_ability(
        world: &mut World,
        vampire_entity: hecs::Entity,
        blood_cost: f32,
    ) -> bool {
        if let Ok(mut blood_meter) = world.get_mut::<BloodMeter>(vampire_entity) {
            blood_meter.consume(blood_cost)
        } else {
            false
        }
    }

    /// Get blood level percentage for UI display
    pub fn get_blood_percentage(world: &World, entity: hecs::Entity) -> Option<f32> {
        world
            .get::<BloodMeter>(entity)
            .ok()
            .map(|blood| blood.blood_percentage())
    }

    /// Check if entity is starving
    pub fn is_starving(world: &World, entity: hecs::Entity) -> bool {
        world
            .get::<BloodMeter>(entity)
            .map_or(false, |blood| blood.is_starving())
    }

    /// Restore blood to maximum (for debugging or special events)
    pub fn restore_blood_to_max(world: &mut World, entity: hecs::Entity) {
        if let Ok(mut blood_meter) = world.get_mut::<BloodMeter>(entity) {
            blood_meter.current = blood_meter.maximum;
        }
    }

    /// Increase maximum blood capacity
    pub fn increase_blood_capacity(world: &mut World, entity: hecs::Entity, amount: f32) {
        if let Ok(mut blood_meter) = world.get_mut::<BloodMeter>(entity) {
            blood_meter.maximum += amount;
            // Also increase current blood proportionally
            blood_meter.current += amount * 0.5;
        }
    }

    /// Apply blood drain from using powerful abilities
    pub fn drain_blood_from_ability_use(
        world: &mut World,
        entity: hecs::Entity,
        ability_type: VampireAbilityType,
    ) -> bool {
        let blood_cost = match ability_type {
            VampireAbilityType::SuperStrength => 5.0,
            VampireAbilityType::SuperSpeed => 3.0,
            VampireAbilityType::BloodSense => 2.0,
            VampireAbilityType::ShadowMovement => 8.0,
            VampireAbilityType::MistForm => 15.0,
            VampireAbilityType::ShadowCommand => 10.0,
        };

        Self::use_blood_for_ability(world, entity, blood_cost)
    }

    /// Calculate blood income from territories
    pub fn calculate_territory_blood_income(territory_manager: &TerritoryManager) -> f32 {
        territory_manager.get_total_blood_income()
    }

    /// Apply passive blood gain from controlled territories
    pub fn apply_territory_blood_income(
        world: &mut World,
        player_entity: hecs::Entity,
        income: f32,
        delta_time: f32,
    ) {
        if let Ok(mut blood_meter) = world.get_mut::<BloodMeter>(player_entity) {
            blood_meter.add_blood(income * delta_time);
        }
    }
}

/// Types of vampire abilities that consume blood
#[derive(Debug, Clone)]
pub enum VampireAbilityType {
    SuperStrength,
    SuperSpeed,
    BloodSense,
    ShadowMovement,
    MistForm,
    ShadowCommand,
}

/// Blood feeding result for better feedback
#[derive(Debug, Clone)]
pub enum FeedingResult {
    Success {
        blood_gained: f32,
        health_restored: f32,
    },
    TargetDead,
    TargetIsVampire,
    VampireNotHungry,
    TooFarAway,
    CannotFeed,
}

impl BloodSystem {
    /// Enhanced feeding method with detailed result
    pub fn attempt_feeding(
        world: &mut World,
        vampire_entity: hecs::Entity,
        target_entity: hecs::Entity,
        max_distance: f32,
    ) -> FeedingResult {
        // Check distance
        if let (Ok(vampire_pos), Ok(target_pos)) = (
            world.get::<Position>(vampire_entity),
            world.get::<Position>(target_entity),
        ) {
            if vampire_pos.distance_to(target_pos) > max_distance {
                return FeedingResult::TooFarAway;
            }
        }

        // Check if target is alive
        if let Ok(target_health) = world.get::<Health>(target_entity) {
            if !target_health.is_alive() {
                return FeedingResult::TargetDead;
            }
        } else {
            return FeedingResult::CannotFeed;
        }

        // Check if target is vampire
        if world.get::<BloodMeter>(target_entity).is_ok() {
            return FeedingResult::TargetIsVampire;
        }

        // Check if vampire needs blood
        if let Ok(blood_meter) = world.get::<BloodMeter>(vampire_entity) {
            if blood_meter.blood_percentage() > 0.9 {
                return FeedingResult::VampireNotHungry;
            }
        }

        // Perform feeding
        let blood_gained = Self::calculate_blood_gain(world, target_entity);

        if Self::feed_on_target(world, vampire_entity, target_entity) {
            FeedingResult::Success {
                blood_gained,
                health_restored: blood_gained * 0.3,
            }
        } else {
            FeedingResult::CannotFeed
        }
    }
}
