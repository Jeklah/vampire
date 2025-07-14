//! Game State Module
//!
//! This module contains the core game state and logic for the Vampire RPG.

use crate::components::*;
use macroquad::prelude::*;
use std::collections::HashMap;

/// Time system for day/night cycle
#[derive(Debug, Clone)]
pub struct TimeSystem {
    pub current_time: f32, // 0.0 to 24.0 hours
    pub day_length: f32,   // Real-time seconds for a full day
    pub day_count: u32,
    pub is_day: bool,
}

impl TimeSystem {
    pub fn new() -> Self {
        Self {
            current_time: 20.0, // Start at night
            day_length: 120.0,  // 2 minutes per day
            day_count: 0,
            is_day: false,
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        self.current_time += (delta_time / self.day_length) * 24.0;

        if self.current_time >= 24.0 {
            self.current_time -= 24.0;
            self.day_count += 1;
        }

        self.is_day = self.current_time >= 6.0 && self.current_time < 18.0;
    }

    pub fn get_time_string(&self) -> String {
        let hours = self.current_time as u32;
        let minutes = ((self.current_time - hours as f32) * 60.0) as u32;
        format!("{:02}:{:02}", hours, minutes)
    }

    pub fn get_sunlight_intensity(&self) -> f32 {
        if !self.is_day {
            return 0.0;
        }
        let noon_distance = (self.current_time - 12.0).abs();
        let max_distance = 6.0;
        if noon_distance > max_distance {
            0.0
        } else {
            1.0 - (noon_distance / max_distance)
        }
    }
}

pub struct GameState {
    pub entities: Vec<GameEntity>,
    pub next_entity_id: u32,
    pub player_id: u32,
    pub time: TimeSystem,
    pub phase: GamePhase,
    pub clans: HashMap<String, Clan>,
    pub camera_x: f32,
    pub camera_y: f32,
    pub phase_objectives: Vec<String>,
    pub completed_objectives: Vec<String>,
    pub paused: bool,
    pub show_clan_menu: bool,
    pub show_legend: bool,
    pub show_quick_start: bool,
    pub game_time: f32,
    pub kills: u32,
    pub feeding_count: u32,
    pub stars: Vec<Star>,
    pub moon: Moon,
    pub blood_particles: Vec<BloodParticle>,
    pub ground_tiles: Vec<GroundTile>,
}

impl GameState {
    pub fn new() -> Self {
        let mut state = Self {
            entities: Vec::new(),
            next_entity_id: 0,
            player_id: 0,
            time: TimeSystem::new(),
            phase: GamePhase::SurvivalAndDiscovery,
            clans: HashMap::new(),
            camera_x: 0.0,
            camera_y: 0.0,
            phase_objectives: vec![
                "Survive your first week".to_string(),
                "Discover your vampire abilities".to_string(),
                "Find shelter from sunlight".to_string(),
                "Feed on blood sources".to_string(),
            ],
            completed_objectives: Vec::new(),
            paused: false,
            show_clan_menu: false,
            show_legend: false,
            show_quick_start: true,
            game_time: 0.0,
            kills: 0,
            feeding_count: 0,
            stars: Vec::new(),
            moon: Moon::new(),
            blood_particles: Vec::new(),
            ground_tiles: Vec::new(),
        };

        state.initialize_world();
        state.initialize_stars();
        state.initialize_ground();
        state
    }

    fn initialize_world(&mut self) {
        // Create player
        let player = GameEntity {
            id: self.next_entity_id,
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

        self.player_id = self.next_entity_id;
        self.entities.push(player);
        self.next_entity_id += 1;

        // Initialize clans
        self.clans.insert(
            "Bone-Eaters".to_string(),
            Clan::new("Bone-Eaters", "Grimjaw", 15),
        );
        self.clans.insert(
            "Flame-Haters".to_string(),
            Clan::new("Flame-Haters", "Shadowmere", 12),
        );
        self.clans.insert(
            "Night-Bloods".to_string(),
            Clan::new("Night-Bloods", "Silentfang", 10),
        );

        // Create clan leaders (on ground)
        self.spawn_clan_leader("Grimjaw", "Bone-Eaters", 200.0, 650.0, BEIGE);
        self.spawn_clan_leader("Shadowmere", "Flame-Haters", 600.0, 700.0, PURPLE);
        self.spawn_clan_leader("Silentfang", "Night-Bloods", 800.0, 620.0, DARKBLUE);

        // Create some hostile infected (on ground)
        for _i in 0..8 {
            let x = rand::gen_range(100.0, 1000.0);
            let y = rand::gen_range(610.0, 1100.0);
            self.spawn_hostile_infected(x, y);
        }

        // Create some animals (blood sources on ground)
        for _i in 0..12 {
            let x = rand::gen_range(50.0, 1200.0);
            let y = rand::gen_range(610.0, 1150.0);
            self.spawn_animal(x, y);
        }
    }

    fn spawn_clan_leader(&mut self, _name: &str, clan: &str, x: f32, y: f32, color: Color) {
        let entity = GameEntity {
            id: self.next_entity_id,
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

        self.entities.push(entity);
        self.next_entity_id += 1;
    }

    fn initialize_stars(&mut self) {
        // Create a starfield background
        for _ in 0..200 {
            let x = rand::gen_range(0.0, 1600.0);
            let y = rand::gen_range(0.0, 1200.0);
            self.stars.push(Star::new(x, y));
        }
    }

    fn initialize_ground(&mut self) {
        // Create ground tiles covering the world
        let tile_size = 64.0;
        let world_width = 1600.0;
        let world_height = 1200.0;

        // Ground starts at y = 600 (lower portion of screen)
        let ground_level = 600.0;

        for x in (0..((world_width / tile_size) as i32)).map(|i| i as f32 * tile_size) {
            for y in (((ground_level / tile_size) as i32)..((world_height / tile_size) as i32))
                .map(|i| i as f32 * tile_size)
            {
                let tile_type = match rand::gen_range(0, 100) {
                    0..=60 => TileType::Grass,
                    61..=80 => TileType::DeadGrass,
                    81..=95 => TileType::Dirt,
                    _ => TileType::Stone,
                };
                self.ground_tiles.push(GroundTile::new(x, y, tile_type));
            }
        }
    }

    fn spawn_hostile_infected(&mut self, x: f32, y: f32) {
        let entity = GameEntity {
            id: self.next_entity_id,
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

        self.entities.push(entity);
        self.next_entity_id += 1;
    }

    fn spawn_animal(&mut self, x: f32, y: f32) {
        let entity = GameEntity {
            id: self.next_entity_id,
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

        self.entities.push(entity);
        self.next_entity_id += 1;
    }

    pub fn update(&mut self, input_handler: &crate::InputHandler, delta_time: f32) {
        // Handle input
        self.handle_input(input_handler, delta_time);

        if self.paused || self.show_clan_menu || self.show_legend || self.show_quick_start {
            return;
        }

        // Update game time
        self.game_time += delta_time;
        self.time.update(delta_time);

        // Update stars and moon
        for star in &mut self.stars {
            star.update(self.game_time);
        }
        self.moon.update(self.game_time);

        // Update blood particles
        self.blood_particles.retain(|particle| {
            let mut p = particle.clone();
            p.update(delta_time)
        });
        for particle in &mut self.blood_particles {
            particle.update(delta_time);
        }

        // Update player movement
        self.update_player_movement(input_handler, delta_time);

        // Update AI and combat
        self.update_ai_and_combat(delta_time);

        // Update blood system
        self.update_blood_system(delta_time);

        // Update camera to follow player
        if let Some(player) = self.entities.iter().find(|e| e.id == self.player_id) {
            self.camera_x = player.position.x;
            self.camera_y = player.position.y;
        }

        // Check objectives
        self.check_objectives();
    }

    fn handle_input(&mut self, input_handler: &crate::InputHandler, _delta_time: f32) {
        // Menu toggles
        if input_handler.is_key_just_pressed(KeyCode::Escape) {
            self.paused = !self.paused;
        }

        if input_handler.is_key_just_pressed(KeyCode::Tab) {
            self.show_clan_menu = !self.show_clan_menu;
        }

        if input_handler.is_key_just_pressed(KeyCode::L) {
            self.show_legend = !self.show_legend;
        }

        if input_handler.is_key_just_pressed(KeyCode::H) {
            self.show_quick_start = !self.show_quick_start;
        }

        // Close quick start guide on any movement
        if self.show_quick_start
            && (input_handler.is_key_pressed(KeyCode::W)
                || input_handler.is_key_pressed(KeyCode::A)
                || input_handler.is_key_pressed(KeyCode::S)
                || input_handler.is_key_pressed(KeyCode::D))
        {
            self.show_quick_start = false;
        }

        // Player actions
        if input_handler.is_key_just_pressed(KeyCode::R) {
            self.attempt_feeding();
        }

        if input_handler.is_key_just_pressed(KeyCode::E) {
            self.attempt_interaction();
        }

        if input_handler.is_key_just_pressed(KeyCode::Space) {
            self.attempt_attack();
        }
    }

    fn update_player_movement(&mut self, input_handler: &crate::InputHandler, delta_time: f32) {
        if let Some(player) = self.entities.iter_mut().find(|e| e.id == self.player_id) {
            let mut move_x = 0.0;
            let mut move_y = 0.0;

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
                move_x *= 0.707;
                move_y *= 0.707;
            }

            let speed = if let Some(abilities) = &player.abilities {
                130.0 * abilities.speed // Reduced for zoom level
            } else {
                130.0
            };

            // Apply sunlight penalty
            let speed_modifier = if self.time.is_day { 0.5 } else { 1.0 };

            player.velocity.x = move_x * speed * speed_modifier;
            player.velocity.y = move_y * speed * speed_modifier;

            // Update position
            player.position.x += player.velocity.x * delta_time;
            player.position.y += player.velocity.y * delta_time;

            // Update facing direction
            if player.velocity.x.abs() > 0.1 || player.velocity.y.abs() > 0.1 {
                player.facing_direction = player.velocity.y.atan2(player.velocity.x);
            }

            // Keep in bounds (prevent going above ground or off edges)
            player.position.x = player.position.x.clamp(0.0, 1600.0);
            player.position.y = player.position.y.clamp(600.0, 1200.0); // Can't go above ground level
        }
    }

    fn update_ai_and_combat(&mut self, delta_time: f32) {
        let player_pos = if let Some(player) = self.entities.iter().find(|e| e.id == self.player_id)
        {
            player.position
        } else {
            return;
        };

        // Simple AI: hostile entities move towards player
        let mut ai_updates = Vec::new();
        for entity in &self.entities {
            if matches!(entity.ai_state, AIState::Hostile) && entity.id != self.player_id {
                if let Some(health) = &entity.health {
                    if health.current > 0.0 {
                        let dx = player_pos.x - entity.position.x;
                        let dy = player_pos.y - entity.position.y;
                        let distance = (dx * dx + dy * dy).sqrt();

                        if distance < 200.0 {
                            // Detection range
                            ai_updates.push((entity.id, dx, dy, distance));
                        }
                    }
                }
            }
        }

        // Apply AI movement
        for (entity_id, dx, dy, distance) in ai_updates {
            if let Some(entity) = self.entities.iter_mut().find(|e| e.id == entity_id) {
                if distance > 0.0 {
                    let speed = 53.0 * delta_time; // Reduced for zoom level
                    entity.velocity.x = (dx / distance) * speed;
                    entity.velocity.y = (dy / distance) * speed;

                    entity.position.x += entity.velocity.x;
                    entity.position.y += entity.velocity.y;

                    // Update facing direction
                    if entity.velocity.x.abs() > 0.1 || entity.velocity.y.abs() > 0.1 {
                        entity.facing_direction = entity.velocity.y.atan2(entity.velocity.x);
                    }

                    // Keep in bounds (on ground)
                    entity.position.x = entity.position.x.clamp(0.0, 1600.0);
                    entity.position.y = entity.position.y.clamp(600.0, 1200.0);
                }
            }
        }
    }

    fn update_blood_system(&mut self, delta_time: f32) {
        for entity in &mut self.entities {
            if let Some(blood_meter) = &mut entity.blood_meter {
                // Drain blood over time
                blood_meter.current -= blood_meter.drain_rate * delta_time;
                blood_meter.current = blood_meter.current.max(0.0);

                // Apply sunlight damage
                if self.time.is_day {
                    let sunlight_damage = 3.0 * self.time.get_sunlight_intensity() * delta_time;
                    if let Some(health) = &mut entity.health {
                        health.current -= sunlight_damage;
                        health.current = health.current.max(0.0);
                    }
                }

                // Apply starvation damage
                if blood_meter.current < blood_meter.maximum * 0.2 {
                    if let Some(health) = &mut entity.health {
                        health.current -= 2.0 * delta_time;
                        health.current = health.current.max(0.0);
                    }
                }
            }
        }
    }

    fn attempt_feeding(&mut self) {
        let player_pos = if let Some(player) = self.entities.iter().find(|e| e.id == self.player_id)
        {
            player.position
        } else {
            return;
        };

        // Find nearby entities to feed on
        let feed_range = 50.0;
        let mut target_id = None;

        for entity in &self.entities {
            if entity.id == self.player_id {
                continue;
            }

            let distance = ((player_pos.x - entity.position.x).powi(2)
                + (player_pos.y - entity.position.y).powi(2))
            .sqrt();

            if distance <= feed_range {
                if let Some(health) = &entity.health {
                    if health.current > 0.0 {
                        target_id = Some(entity.id);
                        break;
                    }
                }
            }
        }

        if let Some(target_id) = target_id {
            // Get position for blood effect
            let target_pos = if let Some(target) = self.entities.iter().find(|e| e.id == target_id)
            {
                target.position
            } else {
                return;
            };

            self.feed_on_target(target_id);

            // Create blood particle effect
            for _ in 0..8 {
                self.blood_particles
                    .push(BloodParticle::new(target_pos.x, target_pos.y));
            }
        }
    }

    fn attempt_attack(&mut self) {
        let player_pos = if let Some(player) = self.entities.iter().find(|e| e.id == self.player_id)
        {
            player.position
        } else {
            return;
        };

        // Find nearby hostile entities to attack
        let attack_range = 60.0;
        let mut target_id = None;

        for entity in &self.entities {
            if entity.id == self.player_id {
                continue;
            }

            if matches!(entity.ai_state, AIState::Hostile) {
                let distance = ((player_pos.x - entity.position.x).powi(2)
                    + (player_pos.y - entity.position.y).powi(2))
                .sqrt();

                if distance <= attack_range {
                    if let Some(health) = &entity.health {
                        if health.current > 0.0 {
                            target_id = Some(entity.id);
                            break;
                        }
                    }
                }
            }
        }

        if let Some(target_id) = target_id {
            self.attack_entity(target_id);
        }
    }

    fn attack_entity(&mut self, target_id: u32) {
        // Get player attack power
        let attack_power =
            if let Some(player) = self.entities.iter().find(|e| e.id == self.player_id) {
                if let Some(combat_stats) = &player.combat_stats {
                    if combat_stats.can_attack(self.game_time) {
                        combat_stats.attack_power
                    } else {
                        return; // Still on cooldown
                    }
                } else {
                    20.0 // Default attack power
                }
            } else {
                return;
            };

        // Apply damage to target
        if let Some(target) = self.entities.iter_mut().find(|e| e.id == target_id) {
            if let Some(health) = &mut target.health {
                let damage =
                    attack_power - target.combat_stats.as_ref().map_or(0.0, |cs| cs.defense);
                let final_damage = damage.max(5.0); // Minimum damage
                health.current -= final_damage;
                health.current = health.current.max(0.0);

                if health.current <= 0.0 {
                    target.ai_state = AIState::Dead;
                    self.kills += 1;
                }
            }
        }

        // Update player attack cooldown
        if let Some(player) = self.entities.iter_mut().find(|e| e.id == self.player_id) {
            if let Some(combat_stats) = &mut player.combat_stats {
                combat_stats.last_attack_time = self.game_time;
            }
        }
    }

    fn feed_on_target(&mut self, target_id: u32) {
        let blood_gained =
            if let Some(target) = self.entities.iter_mut().find(|e| e.id == target_id) {
                if let Some(health) = &mut target.health {
                    let blood_amount = health.current * 0.6;
                    health.current = 0.0; // Feeding is lethal
                    target.ai_state = AIState::Dead;
                    self.feeding_count += 1;
                    blood_amount
                } else {
                    0.0
                }
            } else {
                0.0
            };

        // Add blood to player
        if let Some(player) = self.entities.iter_mut().find(|e| e.id == self.player_id) {
            if let Some(blood_meter) = &mut player.blood_meter {
                blood_meter.current = (blood_meter.current + blood_gained).min(blood_meter.maximum);
            }

            // Heal player
            if let Some(health) = &mut player.health {
                health.current = (health.current + blood_gained * 0.3).min(health.maximum);
            }

            // Improve abilities slightly
            if let Some(abilities) = &mut player.abilities {
                abilities.strength += 0.01;
                abilities.speed += 0.005;
                abilities.blood_sense += 0.02;
            }
        }
    }

    fn attempt_interaction(&mut self) {
        let player_pos = if let Some(player) = self.entities.iter().find(|e| e.id == self.player_id)
        {
            player.position
        } else {
            return;
        };

        // Find nearby clan leaders
        let interact_range = 70.0;

        for entity in &self.entities {
            if let EntityType::ClanLeader(clan_name) = &entity.entity_type {
                let distance = ((player_pos.x - entity.position.x).powi(2)
                    + (player_pos.y - entity.position.y).powi(2))
                .sqrt();

                if distance <= interact_range {
                    // Simple clan interaction - increase trust slightly
                    if let Some(clan) = self.clans.get_mut(clan_name) {
                        clan.trust_towards_player += 0.1;
                        clan.trust_towards_player = clan.trust_towards_player.min(1.0);

                        // Check if clan should become allied
                        if clan.trust_towards_player > 0.7 && !clan.is_allied {
                            clan.is_allied = true;
                        }
                    }
                    break;
                }
            }
        }
    }

    fn check_objectives(&mut self) {
        // Check survival objective
        if self.time.day_count >= 7 {
            self.complete_objective("Survive your first week");
        }

        // Check abilities objective
        if let Some(player) = self.entities.iter().find(|e| e.id == self.player_id) {
            if let Some(abilities) = &player.abilities {
                if abilities.strength > 1.0 || abilities.speed > 1.0 {
                    self.complete_objective("Discover your vampire abilities");
                }
            }
        }

        // Check feeding objective
        if self.feeding_count >= 5 {
            self.complete_objective("Feed on blood sources");
        }

        // Check shelter objective (being alive during day)
        if self.time.day_count >= 1 && self.time.is_day {
            if let Some(player) = self.entities.iter().find(|e| e.id == self.player_id) {
                if let Some(health) = &player.health {
                    if health.current > 20.0 {
                        // Survived day with reasonable health
                        self.complete_objective("Find shelter from sunlight");
                    }
                }
            }
        }

        // Check clan objectives
        let allied_count = self.clans.values().filter(|clan| clan.is_allied).count();
        if allied_count >= 1 {
            self.complete_objective("Establish contact with clan leaders");
        }
    }

    fn complete_objective(&mut self, objective: &str) {
        if let Some(pos) = self
            .phase_objectives
            .iter()
            .position(|obj| obj == objective)
        {
            let completed = self.phase_objectives.remove(pos);
            self.completed_objectives.push(completed);
        }
    }
}
