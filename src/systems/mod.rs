//! Game systems for the vampire RPG
//!
//! This module contains all the systems that operate on components to implement game logic.
//! Systems are the "behavior" part of the Entity Component System (ECS).

use crate::components::*;
use crate::resources::*;
use hecs::World;
use macroquad::prelude::*;
use std::collections::HashMap;

pub mod blood;
pub mod movement;

/// Movement system - handles entity movement and physics
pub struct MovementSystem;

impl MovementSystem {
    pub fn update(world: &mut World, delta_time: f32) {
        // Update positions based on velocity
        for (_, (position, velocity)) in world.query_mut::<(&mut Position, &Velocity)>() {
            position.x += velocity.x * delta_time;
            position.y += velocity.y * delta_time;
        }

        // Apply movement constraints and collision detection
        for (_, (position, _)) in world.query_mut::<(&mut Position, &Player)>() {
            // Keep player within world bounds (example bounds)
            position.x = position.x.clamp(0.0, 1600.0);
            position.y = position.y.clamp(0.0, 1200.0);
        }
    }
}

/// Combat system - handles damage, attacks, and combat resolution
pub struct CombatSystem;

impl CombatSystem {
    pub fn update(world: &mut World, delta_time: f32) {
        // Collect combat events
        let mut combat_events = Vec::new();

        // Find entities in combat range
        let mut entities_with_combat = Vec::new();
        for (entity, (position, combat_stats)) in world.query::<(&Position, &CombatStats)>().iter()
        {
            entities_with_combat.push((entity, *position, combat_stats.clone()));
        }

        // Process combat between entities
        for i in 0..entities_with_combat.len() {
            for j in (i + 1)..entities_with_combat.len() {
                let (entity1, pos1, stats1) = &entities_with_combat[i];
                let (entity2, pos2, stats2) = &entities_with_combat[j];

                let distance = pos1.distance_to(pos2);

                // If entities are close enough and hostile, process combat
                if distance < 50.0 {
                    // Check if they should fight (simplified logic)
                    let should_fight = Self::should_entities_fight(world, *entity1, *entity2);

                    if should_fight {
                        combat_events.push((*entity1, *entity2, stats1.clone(), stats2.clone()));
                    }
                }
            }
        }

        // Apply combat events
        for (entity1, entity2, stats1, stats2) in combat_events {
            Self::resolve_combat(world, entity1, entity2, &stats1, &stats2);
        }
    }

    fn should_entities_fight(world: &World, entity1: hecs::Entity, entity2: hecs::Entity) -> bool {
        // Check if one is player and other is hostile AI
        let entity1_is_player = world.get::<&Player>(entity1).is_ok();
        let entity2_is_player = world.get::<&Player>(entity2).is_ok();

        if entity1_is_player && !entity2_is_player {
            if let Ok(ai) = world.get::<&AI>(entity2) {
                return matches!(ai.behavior, AIBehavior::Hostile);
            }
        }

        if entity2_is_player && !entity1_is_player {
            if let Ok(ai) = world.get::<&AI>(entity1) {
                return matches!(ai.behavior, AIBehavior::Hostile);
            }
        }

        false
    }

    fn resolve_combat(
        world: &mut World,
        entity1: hecs::Entity,
        entity2: hecs::Entity,
        stats1: &CombatStats,
        stats2: &CombatStats,
    ) {
        // Calculate damage from entity1 to entity2
        if rand::gen_range(0.0, 1.0) < stats1.accuracy {
            let base_damage = stats1.attack_power;
            let critical_multiplier = if rand::gen_range(0.0, 1.0) < stats1.critical_chance {
                2.0
            } else {
                1.0
            };

            let damage = base_damage * critical_multiplier - stats2.defense;
            let final_damage = damage.max(1.0); // Minimum 1 damage

            // Apply damage to entity2
            if let Ok(mut health) = world.get::<&mut Health>(entity2) {
                health.take_damage(final_damage);
            }
        }

        // Calculate damage from entity2 to entity1
        if rand::gen_range(0.0, 1.0) < stats2.accuracy {
            let base_damage = stats2.attack_power;
            let critical_multiplier = if rand::gen_range(0.0, 1.0) < stats2.critical_chance {
                2.0
            } else {
                1.0
            };

            let damage = base_damage * critical_multiplier - stats1.defense;
            let final_damage = damage.max(1.0);

            // Apply damage to entity1
            if let Ok(mut health) = world.get::<&mut Health>(entity1) {
                health.take_damage(final_damage);
            }
        }
    }
}

/// Blood system - handles blood consumption, regeneration, and effects
pub struct BloodSystem;

impl BloodSystem {
    pub fn update(world: &mut World, time: &TimeResource, delta_time: f32) {
        // Drain blood over time
        for (_, blood_meter) in world.query_mut::<&mut BloodMeter>() {
            blood_meter.current -= blood_meter.drain_rate * delta_time;
            blood_meter.current = blood_meter.current.max(0.0);
        }

        // Apply blood starvation effects
        for (entity, (blood_meter, health)) in world.query_mut::<(&BloodMeter, &mut Health)>() {
            if blood_meter.is_starving() {
                // Take damage when starving
                health.take_damage(2.0 * delta_time);
            }
        }

        // Sunlight damage for vampires
        if time.is_day {
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
    }

    pub fn feed_on_target(
        world: &mut World,
        vampire_entity: hecs::Entity,
        target_entity: hecs::Entity,
    ) -> bool {
        // Check if vampire can feed
        let can_feed = world.get::<BloodMeter>(vampire_entity).is_ok()
            && world.get::<Health>(target_entity).is_ok();

        if !can_feed {
            return false;
        }

        // Get blood amount from target
        let blood_amount = if let Ok(health) = world.get::<Health>(target_entity) {
            health.current * 0.5 // Take half of target's health as blood
        } else {
            return false;
        };

        // Drain target's health
        if let Ok(mut target_health) = world.get_mut::<Health>(target_entity) {
            target_health.take_damage(blood_amount);
        }

        // Add blood to vampire
        if let Ok(mut blood_meter) = world.get_mut::<BloodMeter>(vampire_entity) {
            blood_meter.add_blood(blood_amount);
        }

        // Heal vampire slightly from feeding
        if let Ok(mut vampire_health) = world.get_mut::<Health>(vampire_entity) {
            vampire_health.heal(blood_amount * 0.2);
        }

        true
    }
}

/// Time system - manages day/night cycle and time-based effects
pub struct TimeSystem;

impl TimeSystem {
    pub fn update(time: &mut TimeResource, delta_time: f32) {
        time.update(delta_time);
    }

    pub fn update_sunlight_exposure(world: &mut World, time: &TimeResource) {
        // Update sunlight exposure for all entities with vulnerability
        for (_, vulnerability) in world.query_mut::<&mut SunlightVulnerability>() {
            // Simplified: assume all vampires are in sunlight during day
            // In a real game, you'd check if they're in shelter/caves
            vulnerability.in_sunlight = time.is_day;
        }
    }
}

/// AI system - handles NPC behavior and decision making
pub struct AISystem;

impl AISystem {
    pub fn update(world: &mut World, delta_time: f32) {
        // Get player position for AI reference
        let player_pos = Self::get_player_position(world);

        // Update AI behaviors
        for (entity, (position, ai, velocity)) in
            world.query_mut::<(&Position, &mut AI, &mut Velocity)>()
        {
            match ai.state {
                AIState::Idle => {
                    Self::handle_idle_ai(entity, position, ai, velocity, player_pos);
                }
                AIState::Patrolling => {
                    Self::handle_patrolling_ai(entity, position, ai, velocity, delta_time);
                }
                AIState::Hunting => {
                    Self::handle_hunting_ai(entity, position, ai, velocity, player_pos);
                }
                AIState::Fleeing => {
                    Self::handle_fleeing_ai(entity, position, ai, velocity, player_pos);
                }
                AIState::Following => {
                    Self::handle_following_ai(entity, position, ai, velocity, player_pos);
                }
                AIState::Attacking => {
                    Self::handle_attacking_ai(entity, position, ai, velocity, player_pos);
                }
            }
        }
    }

    fn get_player_position(world: &World) -> Option<Position> {
        for (_, (position, _)) in world.query::<(&Position, &Player)>().iter() {
            return Some(*position);
        }
        None
    }

    fn handle_idle_ai(
        entity: hecs::Entity,
        position: &Position,
        ai: &mut AI,
        velocity: &mut Velocity,
        player_pos: Option<Position>,
    ) {
        *velocity = Velocity::zero();

        if let Some(player_pos) = player_pos {
            let distance = position.distance_to(&player_pos);

            if distance < ai.awareness_radius {
                match ai.behavior {
                    AIBehavior::Hostile => {
                        ai.state = AIState::Hunting;
                        ai.target = Some(player_pos);
                    }
                    AIBehavior::Fearful => {
                        ai.state = AIState::Fleeing;
                        ai.target = Some(player_pos);
                    }
                    AIBehavior::Loyal => {
                        ai.state = AIState::Following;
                        ai.target = Some(player_pos);
                    }
                    _ => {}
                }
            }
        }
    }

    fn handle_patrolling_ai(
        entity: hecs::Entity,
        position: &Position,
        ai: &mut AI,
        velocity: &mut Velocity,
        delta_time: f32,
    ) {
        // Simple patrol behavior - move in a pattern
        let patrol_speed = 50.0;
        let time_factor = get_time() as f32;

        velocity.x = (time_factor * 0.5).sin() * patrol_speed;
        velocity.y = (time_factor * 0.3).cos() * patrol_speed;
    }

    fn handle_hunting_ai(
        entity: hecs::Entity,
        position: &Position,
        ai: &mut AI,
        velocity: &mut Velocity,
        player_pos: Option<Position>,
    ) {
        if let Some(target) = ai.target {
            let dx = target.x - position.x;
            let dy = target.y - position.y;
            let distance = (dx * dx + dy * dy).sqrt();

            if distance > 0.0 {
                let speed = 100.0 * ai.aggression;
                velocity.x = (dx / distance) * speed;
                velocity.y = (dy / distance) * speed;

                // Switch to attacking if very close
                if distance < 30.0 {
                    ai.state = AIState::Attacking;
                }
            }
        } else {
            ai.state = AIState::Idle;
        }
    }

    fn handle_fleeing_ai(
        entity: hecs::Entity,
        position: &Position,
        ai: &mut AI,
        velocity: &mut Velocity,
        player_pos: Option<Position>,
    ) {
        if let Some(target) = ai.target {
            let dx = target.x - position.x;
            let dy = target.y - position.y;
            let distance = (dx * dx + dy * dy).sqrt();

            if distance > 0.0 {
                let speed = 120.0; // Flee faster than hunting
                velocity.x = -(dx / distance) * speed;
                velocity.y = -(dy / distance) * speed;

                // Stop fleeing if far enough away
                if distance > ai.awareness_radius * 2.0 {
                    ai.state = AIState::Idle;
                    ai.target = None;
                }
            }
        } else {
            ai.state = AIState::Idle;
        }
    }

    fn handle_following_ai(
        entity: hecs::Entity,
        position: &Position,
        ai: &mut AI,
        velocity: &mut Velocity,
        player_pos: Option<Position>,
    ) {
        if let Some(target) = ai.target {
            let dx = target.x - position.x;
            let dy = target.y - position.y;
            let distance = (dx * dx + dy * dy).sqrt();

            if distance > 60.0 {
                // Follow if too far
                let speed = 80.0;
                velocity.x = (dx / distance) * speed;
                velocity.y = (dy / distance) * speed;
            } else if distance < 40.0 {
                // Stop if too close
                *velocity = Velocity::zero();
            }
        } else {
            ai.state = AIState::Idle;
        }
    }

    fn handle_attacking_ai(
        entity: hecs::Entity,
        position: &Position,
        ai: &mut AI,
        velocity: &mut Velocity,
        player_pos: Option<Position>,
    ) {
        *velocity = Velocity::zero(); // Stop to attack

        if let Some(target) = ai.target {
            let distance = position.distance_to(&target);

            if distance > 50.0 {
                // Target moved away, go back to hunting
                ai.state = AIState::Hunting;
            }
        } else {
            ai.state = AIState::Idle;
        }
    }
}

/// Render system - handles drawing all visual elements
pub struct RenderSystem;

impl RenderSystem {
    pub fn draw(world: &World, camera: &CameraResource) {
        // Set up camera transform
        let camera_transform = Mat4::from_translation(Vec3::new(
            -camera.position.0 + screen_width() / 2.0,
            -camera.position.1 + screen_height() / 2.0,
            0.0,
        )) * Mat4::from_scale(Vec3::new(camera.zoom, camera.zoom, 1.0));

        // Draw all renderable entities
        for (entity, (position, render)) in world.query::<(&Position, &Render)>().iter() {
            if !render.visible {
                continue;
            }

            let world_pos =
                camera_transform.transform_point3(Vec3::new(position.x, position.y, 0.0));

            if let Some(texture) = render.texture {
                draw_texture_ex(
                    texture,
                    world_pos.x - texture.width() / 2.0,
                    world_pos.y - texture.height() / 2.0,
                    render.color,
                    DrawTextureParams {
                        dest_size: Some(Vec2::new(
                            texture.width() * render.scale,
                            texture.height() * render.scale,
                        )),
                        rotation: render.rotation,
                        ..Default::default()
                    },
                );
            } else {
                // Draw a simple colored rectangle if no texture
                draw_rectangle(
                    world_pos.x - 10.0,
                    world_pos.y - 10.0,
                    20.0 * render.scale,
                    20.0 * render.scale,
                    render.color,
                );
            }
        }
    }

    pub fn draw_ui(world: &World, time: &TimeResource, clan_registry: &ClanRegistry) {
        // Draw time UI
        draw_text(
            &format!("Time: {} - Day {}", time.get_time_string(), time.day_count),
            20.0,
            30.0,
            24.0,
            WHITE,
        );

        // Draw day/night indicator
        let day_text = if time.is_day { "DAY" } else { "NIGHT" };
        let day_color = if time.is_day { YELLOW } else { DARKBLUE };
        draw_text(day_text, 20.0, 60.0, 24.0, day_color);

        // Draw player stats
        for (_, (blood_meter, health, player)) in
            world.query::<(&BloodMeter, &Health, &Player)>().iter()
        {
            let y_offset = 100.0;

            // Health bar
            draw_rectangle(20.0, y_offset, 200.0, 20.0, Color::new(0.3, 0.0, 0.0, 1.0));
            draw_rectangle(
                20.0,
                y_offset,
                200.0 * health.health_percentage(),
                20.0,
                RED,
            );
            draw_text("Health", 20.0, y_offset - 5.0, 16.0, WHITE);

            // Blood bar
            draw_rectangle(
                20.0,
                y_offset + 30.0,
                200.0,
                20.0,
                Color::new(0.0, 0.0, 0.3, 1.0),
            );
            draw_rectangle(
                20.0,
                y_offset + 30.0,
                200.0 * blood_meter.blood_percentage(),
                20.0,
                BLUE,
            );
            draw_text("Blood", 20.0, y_offset + 25.0, 16.0, WHITE);

            // Player info
            draw_text(
                &format!("Phase: {:?}", player.phase),
                20.0,
                y_offset + 60.0,
                16.0,
                WHITE,
            );
            draw_text(
                &format!("Reputation: {:.1}", player.reputation),
                20.0,
                y_offset + 80.0,
                16.0,
                WHITE,
            );
        }

        // Draw clan information
        let mut y = 200.0;
        draw_text("Clans:", 20.0, y, 20.0, WHITE);
        y += 25.0;

        for clan in clan_registry.clans.values() {
            let status = if clan.is_allied {
                "Allied"
            } else if clan.is_defeated {
                "Defeated"
            } else {
                "Neutral"
            };

            draw_text(
                &format!(
                    "{}: {} (Trust: {:.1}, Fear: {:.1})",
                    clan.name, status, clan.trust_towards_player, clan.fear_of_player
                ),
                20.0,
                y,
                16.0,
                WHITE,
            );
            y += 20.0;
        }
    }
}

/// Phase management system
pub struct PhaseSystem;

impl PhaseSystem {
    pub fn update(world: &mut World, game_phase: &mut GamePhase, clan_registry: &ClanRegistry) {
        // Check objectives based on current phase
        match game_phase.current_phase {
            crate::resources::Phase::SurvivalAndDiscovery => {
                Self::check_survival_objectives(world, game_phase);
            }
            crate::resources::Phase::ClanEncounters => {
                Self::check_clan_objectives(world, game_phase, clan_registry);
            }
            crate::resources::Phase::EmpireBuilding => {
                Self::check_empire_objectives(world, game_phase, clan_registry);
            }
            crate::resources::Phase::WorldReaction => {
                Self::check_world_reaction_objectives(world, game_phase);
            }
        }
    }

    fn check_survival_objectives(world: &World, game_phase: &mut GamePhase) {
        // Check if player has survived and discovered abilities
        for (_, (player, abilities)) in world.query::<(&Player, &VampireAbilities)>().iter() {
            if player.hibernation_cycles >= 1 {
                game_phase.complete_objective("Survive your first week");
            }

            if abilities.strength > 1.0 || abilities.speed > 1.0 {
                game_phase.complete_objective("Discover your vampire abilities");
            }
        }
    }

    fn check_clan_objectives(
        world: &World,
        game_phase: &mut GamePhase,
        clan_registry: &ClanRegistry,
    ) {
        let allied_count = clan_registry.allied_clans().len();

        if allied_count >= 1 {
            game_phase.complete_objective("Locate the first clan");
            game_phase.complete_objective("Establish contact with clan leaders");
        }

        if allied_count >= 2 {
            game_phase.complete_objective("Gain control of at least 2 clans");
        }
    }

    fn check_empire_objectives(
        world: &World,
        game_phase: &mut GamePhase,
        clan_registry: &ClanRegistry,
    ) {
        // Check empire building progress
        // This would check territory control, lair building, etc.
        // Simplified for now
    }

    fn check_world_reaction_objectives(world: &World, game_phase: &mut GamePhase) {
        // Check final phase objectives
        // This would involve checking for hunter encounters, ancient discoveries, etc.
        // Simplified for now
    }
}
