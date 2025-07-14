//! Movement system for handling entity movement and physics
//!
//! This module contains the movement system that updates entity positions
//! based on their velocities and handles basic collision detection.

use crate::components::*;
use crate::resources::*;
use hecs::World;
use macroquad::prelude::*;

/// Movement system that handles all entity movement
pub struct MovementSystem;

impl MovementSystem {
    /// Update all entity positions based on their velocities
    pub fn update(world: &mut World, delta_time: f32) {
        // Basic movement update
        for (_, (position, velocity)) in world.query_mut::<(&mut Position, &Velocity)>() {
            position.x += velocity.x * delta_time;
            position.y += velocity.y * delta_time;
        }

        // Apply world boundaries
        Self::apply_world_boundaries(world);

        // Apply movement modifiers
        Self::apply_movement_modifiers(world, delta_time);
    }

    /// Keep entities within world boundaries
    fn apply_world_boundaries(world: &mut World) {
        const WORLD_WIDTH: f32 = 1600.0;
        const WORLD_HEIGHT: f32 = 1200.0;
        const BOUNDARY_MARGIN: f32 = 10.0;

        for (_, position) in world.query_mut::<&mut Position>() {
            position.x = position
                .x
                .clamp(BOUNDARY_MARGIN, WORLD_WIDTH - BOUNDARY_MARGIN);
            position.y = position
                .y
                .clamp(BOUNDARY_MARGIN, WORLD_HEIGHT - BOUNDARY_MARGIN);
        }
    }

    /// Apply movement speed modifiers based on abilities and conditions
    fn apply_movement_modifiers(world: &mut World, delta_time: f32) {
        // Apply vampire speed bonuses
        for (entity, (velocity, abilities)) in
            world.query_mut::<(&mut Velocity, &VampireAbilities)>()
        {
            let speed_multiplier = abilities.speed;

            // Apply speed bonus
            velocity.x *= speed_multiplier;
            velocity.y *= speed_multiplier;
        }

        // Apply sunlight movement penalties
        for (entity, (velocity, vulnerability)) in
            world.query_mut::<(&mut Velocity, &SunlightVulnerability)>()
        {
            if vulnerability.in_sunlight {
                let penalty = 1.0 - vulnerability.movement_penalty;
                velocity.x *= penalty;
                velocity.y *= penalty;
            }
        }
    }

    /// Calculate distance between two positions
    pub fn distance_between(pos1: &Position, pos2: &Position) -> f32 {
        let dx = pos1.x - pos2.x;
        let dy = pos1.y - pos2.y;
        (dx * dx + dy * dy).sqrt()
    }

    /// Check if two entities are within interaction range
    pub fn entities_in_range(
        world: &World,
        entity1: hecs::Entity,
        entity2: hecs::Entity,
        range: f32,
    ) -> bool {
        if let (Ok(pos1), Ok(pos2)) = (
            world.get::<Position>(entity1),
            world.get::<Position>(entity2),
        ) {
            Self::distance_between(pos1, pos2) <= range
        } else {
            false
        }
    }

    /// Move entity towards a target position
    pub fn move_towards(
        position: &Position,
        target: &Position,
        speed: f32,
        delta_time: f32,
    ) -> Velocity {
        let dx = target.x - position.x;
        let dy = target.y - position.y;
        let distance = (dx * dx + dy * dy).sqrt();

        if distance > 0.0 {
            Velocity::new(
                (dx / distance) * speed * delta_time,
                (dy / distance) * speed * delta_time,
            )
        } else {
            Velocity::zero()
        }
    }

    /// Stop an entity's movement
    pub fn stop_entity(world: &mut World, entity: hecs::Entity) {
        if let Ok(mut velocity) = world.get_mut::<Velocity>(entity) {
            *velocity = Velocity::zero();
        }
    }

    /// Set entity velocity directly
    pub fn set_velocity(world: &mut World, entity: hecs::Entity, new_velocity: Velocity) {
        if let Ok(mut velocity) = world.get_mut::<Velocity>(entity) {
            *velocity = new_velocity;
        }
    }

    /// Apply knockback effect to an entity
    pub fn apply_knockback(world: &mut World, entity: hecs::Entity, force_x: f32, force_y: f32) {
        if let Ok(mut velocity) = world.get_mut::<Velocity>(entity) {
            velocity.x += force_x;
            velocity.y += force_y;
        }
    }

    /// Get all entities within a certain radius of a position
    pub fn get_entities_in_radius(
        world: &World,
        center: &Position,
        radius: f32,
    ) -> Vec<hecs::Entity> {
        let mut entities = Vec::new();

        for (entity, position) in world.query::<&Position>().iter() {
            if Self::distance_between(center, position) <= radius {
                entities.push(entity);
            }
        }

        entities
    }

    /// Check if a position is within world boundaries
    pub fn is_valid_position(position: &Position) -> bool {
        const WORLD_WIDTH: f32 = 1600.0;
        const WORLD_HEIGHT: f32 = 1200.0;

        position.x >= 0.0
            && position.x <= WORLD_WIDTH
            && position.y >= 0.0
            && position.y <= WORLD_HEIGHT
    }

    /// Clamp position to world boundaries
    pub fn clamp_to_world(position: &mut Position) {
        const WORLD_WIDTH: f32 = 1600.0;
        const WORLD_HEIGHT: f32 = 1200.0;

        position.x = position.x.clamp(0.0, WORLD_WIDTH);
        position.y = position.y.clamp(0.0, WORLD_HEIGHT);
    }
}
