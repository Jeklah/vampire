//! Main entry point for the Vampire RPG
//!
//! This file initializes Macroquad and runs the main game loop.

use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Core game components
#[derive(Debug, Clone, Copy)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone, Copy)]
struct Velocity {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone)]
struct Health {
    current: f32,
    maximum: f32,
}

#[derive(Debug, Clone)]
struct BloodMeter {
    current: f32,
    maximum: f32,
    drain_rate: f32,
}

#[derive(Debug, Clone)]
struct VampireAbilities {
    strength: f32,
    speed: f32,
    blood_sense: f32,
    shadow_movement: f32,
}

#[derive(Debug, Clone)]
struct CombatStats {
    attack_power: f32,
    defense: f32,
    last_attack_time: f32,
    attack_cooldown: f32,
}

impl CombatStats {
    fn new(attack_power: f32, defense: f32) -> Self {
        Self {
            attack_power,
            defense,
            last_attack_time: 0.0,
            attack_cooldown: 1.0,
        }
    }

    fn can_attack(&self, current_time: f32) -> bool {
        current_time - self.last_attack_time >= self.attack_cooldown
    }
}

#[derive(Debug, Clone)]
struct GameEntity {
    id: u32,
    position: Position,
    velocity: Velocity,
    health: Option<Health>,
    blood_meter: Option<BloodMeter>,
    abilities: Option<VampireAbilities>,
    combat_stats: Option<CombatStats>,
    entity_type: EntityType,
    color: Color,
    ai_target: Option<u32>,
    ai_state: AIState,
}

#[derive(Debug, Clone)]
enum AIState {
    Idle,
    Hostile,
    Fleeing,
    Dead,
}

#[derive(Debug, Clone)]
enum EntityType {
    Player,
    ClanLeader(String),
    ClanMember(String),
    HostileInfected,
    Animal,
}

// Game phase system
#[derive(Debug, Clone, Serialize, Deserialize)]
enum GamePhase {
    SurvivalAndDiscovery,
    ClanEncounters,
    EmpireBuilding,
    WorldReaction,
}

// Time system
#[derive(Debug, Clone)]
struct TimeSystem {
    current_time: f32, // 0.0 to 24.0 hours
    day_length: f32,   // Real-time seconds for a full day
    day_count: u32,
    is_day: bool,
}

impl TimeSystem {
    fn new() -> Self {
        Self {
            current_time: 20.0, // Start at night
            day_length: 120.0,  // 2 minutes per day
            day_count: 0,
            is_day: false,
        }
    }

    fn update(&mut self, delta_time: f32) {
        self.current_time += (delta_time / self.day_length) * 24.0;

        if self.current_time >= 24.0 {
            self.current_time -= 24.0;
            self.day_count += 1;
        }

        self.is_day = self.current_time >= 6.0 && self.current_time < 18.0;
    }

    fn get_time_string(&self) -> String {
        let hours = self.current_time as u32;
        let minutes = ((self.current_time - hours as f32) * 60.0) as u32;
        format!("{:02}:{:02}", hours, minutes)
    }

    fn get_sunlight_intensity(&self) -> f32 {
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

// Clan system
#[derive(Debug, Clone)]
struct Clan {
    name: String,
    leader_name: String,
    member_count: u32,
    trust_towards_player: f32,
    fear_of_player: f32,
    strength: f32,
    is_allied: bool,
    is_defeated: bool,
}

impl Clan {
    fn new(name: &str, leader_name: &str, member_count: u32) -> Self {
        Self {
            name: name.to_string(),
            leader_name: leader_name.to_string(),
            member_count,
            trust_towards_player: 0.0,
            fear_of_player: 0.0,
            strength: 1.0,
            is_allied: false,
            is_defeated: false,
        }
    }

    fn loyalty_score(&self) -> f32 {
        (self.trust_towards_player - self.fear_of_player).clamp(-1.0, 1.0)
    }
}

// Main game state
struct GameState {
    entities: Vec<GameEntity>,
    next_entity_id: u32,
    player_id: u32,
    time: TimeSystem,
    phase: GamePhase,
    clans: HashMap<String, Clan>,
    camera_x: f32,
    camera_y: f32,
    phase_objectives: Vec<String>,
    completed_objectives: Vec<String>,
    paused: bool,
    show_clan_menu: bool,
    show_legend: bool,
    show_quick_start: bool,
    game_time: f32,
    kills: u32,
    feeding_count: u32,
}

impl GameState {
    fn new() -> Self {
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
        };

        state.initialize_world();
        state
    }

    fn initialize_world(&mut self) {
        // Create player
        let player = GameEntity {
            id: self.next_entity_id,
            position: Position { x: 400.0, y: 300.0 },
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

        // Create clan leaders
        self.spawn_clan_leader("Grimjaw", "Bone-Eaters", 200.0, 200.0, BEIGE);
        self.spawn_clan_leader("Shadowmere", "Flame-Haters", 600.0, 400.0, PURPLE);
        self.spawn_clan_leader("Silentfang", "Night-Bloods", 800.0, 200.0, DARKBLUE);

        // Create some hostile infected
        for _i in 0..8 {
            let x = rand::gen_range(100.0, 1000.0);
            let y = rand::gen_range(100.0, 600.0);
            self.spawn_hostile_infected(x, y);
        }

        // Create some animals (blood sources)
        for _i in 0..12 {
            let x = rand::gen_range(50.0, 1200.0);
            let y = rand::gen_range(50.0, 700.0);
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
        };

        self.entities.push(entity);
        self.next_entity_id += 1;
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
        };

        self.entities.push(entity);
        self.next_entity_id += 1;
    }

    fn update(&mut self, delta_time: f32) {
        // Handle input
        self.handle_input(delta_time);

        if self.paused || self.show_clan_menu || self.show_legend || self.show_quick_start {
            return;
        }

        // Update game time
        self.game_time += delta_time;
        self.time.update(delta_time);

        // Update player movement
        self.update_player_movement(delta_time);

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

    fn handle_input(&mut self, _delta_time: f32) {
        // Menu toggles
        if is_key_pressed(KeyCode::Escape) {
            self.paused = !self.paused;
        }

        if is_key_pressed(KeyCode::Tab) {
            self.show_clan_menu = !self.show_clan_menu;
        }

        if is_key_pressed(KeyCode::L) {
            self.show_legend = !self.show_legend;
        }

        if is_key_pressed(KeyCode::H) {
            self.show_quick_start = !self.show_quick_start;
        }

        // Close quick start guide on any movement
        if self.show_quick_start
            && (is_key_down(KeyCode::W)
                || is_key_down(KeyCode::A)
                || is_key_down(KeyCode::S)
                || is_key_down(KeyCode::D))
        {
            self.show_quick_start = false;
        }

        // Player actions
        if is_key_pressed(KeyCode::R) {
            self.attempt_feeding();
        }

        if is_key_pressed(KeyCode::E) {
            self.attempt_interaction();
        }

        if is_key_pressed(KeyCode::Space) {
            self.attempt_attack();
        }
    }

    fn update_player_movement(&mut self, delta_time: f32) {
        if let Some(player) = self.entities.iter_mut().find(|e| e.id == self.player_id) {
            let mut move_x = 0.0;
            let mut move_y = 0.0;

            if is_key_down(KeyCode::W) {
                move_y = -1.0;
            }
            if is_key_down(KeyCode::S) {
                move_y = 1.0;
            }
            if is_key_down(KeyCode::A) {
                move_x = -1.0;
            }
            if is_key_down(KeyCode::D) {
                move_x = 1.0;
            }

            // Normalize diagonal movement
            if move_x != 0.0 && move_y != 0.0 {
                move_x *= 0.707;
                move_y *= 0.707;
            }

            let speed = if let Some(abilities) = &player.abilities {
                200.0 * abilities.speed
            } else {
                200.0
            };

            // Apply sunlight penalty
            let speed_modifier = if self.time.is_day { 0.5 } else { 1.0 };

            player.velocity.x = move_x * speed * speed_modifier;
            player.velocity.y = move_y * speed * speed_modifier;

            // Update position
            player.position.x += player.velocity.x * delta_time;
            player.position.y += player.velocity.y * delta_time;

            // Keep in bounds
            player.position.x = player.position.x.clamp(0.0, 1600.0);
            player.position.y = player.position.y.clamp(0.0, 1200.0);
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
                    let speed = 80.0 * delta_time;
                    entity.velocity.x = (dx / distance) * speed;
                    entity.velocity.y = (dy / distance) * speed;

                    entity.position.x += entity.velocity.x;
                    entity.position.y += entity.velocity.y;

                    // Keep in bounds
                    entity.position.x = entity.position.x.clamp(0.0, 1600.0);
                    entity.position.y = entity.position.y.clamp(0.0, 1200.0);
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
            self.feed_on_target(target_id);
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

    fn render(&self) {
        clear_background(BLACK);

        // Calculate camera offset
        let camera_offset_x = screen_width() / 2.0 - self.camera_x;
        let camera_offset_y = screen_height() / 2.0 - self.camera_y;

        // Draw all entities
        for entity in &self.entities {
            let screen_x = entity.position.x + camera_offset_x;
            let screen_y = entity.position.y + camera_offset_y;

            // Only draw if on screen
            if screen_x > -20.0
                && screen_x < screen_width() + 20.0
                && screen_y > -20.0
                && screen_y < screen_height() + 20.0
            {
                let size = match entity.entity_type {
                    EntityType::Player => 20.0,
                    EntityType::ClanLeader(_) => 18.0,
                    EntityType::ClanMember(_) => 15.0,
                    EntityType::HostileInfected => 12.0,
                    EntityType::Animal => 10.0,
                };

                // Don't draw dead entities
                if let Some(health) = &entity.health {
                    if health.current <= 0.0 || matches!(entity.ai_state, AIState::Dead) {
                        continue;
                    }
                }

                // Draw entity with visual distinction
                match entity.entity_type {
                    EntityType::Player => {
                        // Player - diamond shape
                        draw_rectangle(
                            screen_x - size / 2.0,
                            screen_y - size / 2.0,
                            size,
                            size,
                            entity.color,
                        );
                        // Add border to make player stand out
                        draw_rectangle_lines(
                            screen_x - size / 2.0,
                            screen_y - size / 2.0,
                            size,
                            size,
                            2.0,
                            WHITE,
                        );
                    }
                    EntityType::ClanLeader(_) => {
                        // Clan leaders - larger rectangle with crown symbol
                        draw_rectangle(
                            screen_x - size / 2.0,
                            screen_y - size / 2.0,
                            size,
                            size,
                            entity.color,
                        );
                        // Draw crown (small triangle on top)
                        draw_triangle(
                            Vec2::new(screen_x, screen_y - size / 2.0 - 3.0),
                            Vec2::new(screen_x - 4.0, screen_y - size / 2.0 + 2.0),
                            Vec2::new(screen_x + 4.0, screen_y - size / 2.0 + 2.0),
                            GOLD,
                        );
                    }
                    EntityType::HostileInfected => {
                        // Hostile infected - jagged shape
                        draw_rectangle(
                            screen_x - size / 2.0,
                            screen_y - size / 2.0,
                            size,
                            size,
                            entity.color,
                        );
                        // Add X mark to show hostility
                        draw_line(
                            screen_x - size / 3.0,
                            screen_y - size / 3.0,
                            screen_x + size / 3.0,
                            screen_y + size / 3.0,
                            2.0,
                            RED,
                        );
                        draw_line(
                            screen_x + size / 3.0,
                            screen_y - size / 3.0,
                            screen_x - size / 3.0,
                            screen_y + size / 3.0,
                            2.0,
                            RED,
                        );
                    }
                    EntityType::Animal => {
                        // Animals - circle shape
                        draw_circle(screen_x, screen_y, size / 2.0, entity.color);
                    }
                    EntityType::ClanMember(_) => {
                        // Regular rectangle for clan members
                        draw_rectangle(
                            screen_x - size / 2.0,
                            screen_y - size / 2.0,
                            size,
                            size,
                            entity.color,
                        );
                    }
                }

                // Draw health bar if entity has health
                if let Some(health) = &entity.health {
                    let bar_width = size;
                    let bar_height = 4.0;
                    let bar_y = screen_y - size / 2.0 - 8.0;

                    // Background bar
                    draw_rectangle(
                        screen_x - bar_width / 2.0,
                        bar_y,
                        bar_width,
                        bar_height,
                        Color::new(0.3, 0.0, 0.0, 0.8),
                    );

                    // Health bar
                    let health_percentage = health.current / health.maximum;
                    let health_width = bar_width * health_percentage;
                    let health_color = if health_percentage > 0.6 {
                        GREEN
                    } else if health_percentage > 0.3 {
                        YELLOW
                    } else {
                        RED
                    };

                    draw_rectangle(
                        screen_x - bar_width / 2.0,
                        bar_y,
                        health_width,
                        bar_height,
                        health_color,
                    );
                }
            }
        }

        // Draw UI
        self.draw_ui();

        // Draw menus
        if self.paused {
            self.draw_pause_menu();
        }

        if self.show_clan_menu {
            self.draw_clan_menu();
        }

        if self.show_legend {
            self.draw_legend();
        }

        if self.show_quick_start {
            self.draw_quick_start_guide();
        }
    }

    fn draw_ui(&self) {
        // Time display
        let time_text = format!(
            "Time: {} - Day {}",
            self.time.get_time_string(),
            self.time.day_count
        );
        draw_text(&time_text, 20.0, 30.0, 24.0, WHITE);

        // Day/night indicator
        let day_text = if self.time.is_day { "DAY" } else { "NIGHT" };
        let day_color = if self.time.is_day { YELLOW } else { BLUE };
        draw_text(day_text, 20.0, 60.0, 24.0, day_color);

        // Player stats
        if let Some(player) = self.entities.iter().find(|e| e.id == self.player_id) {
            let mut y_offset = 100.0;

            // Health bar
            if let Some(health) = &player.health {
                draw_rectangle(20.0, y_offset, 200.0, 20.0, Color::new(0.3, 0.0, 0.0, 1.0));
                let health_width = 200.0 * (health.current / health.maximum);
                draw_rectangle(20.0, y_offset, health_width, 20.0, RED);
                draw_text("Health", 20.0, y_offset - 5.0, 16.0, WHITE);
                y_offset += 30.0;
            }

            // Blood bar
            if let Some(blood) = &player.blood_meter {
                draw_rectangle(20.0, y_offset, 200.0, 20.0, Color::new(0.0, 0.0, 0.3, 1.0));
                let blood_width = 200.0 * (blood.current / blood.maximum);
                draw_rectangle(20.0, y_offset, blood_width, 20.0, BLUE);
                draw_text("Blood", 20.0, y_offset - 5.0, 16.0, WHITE);
                y_offset += 30.0;
            }

            // Phase info
            draw_text(
                &format!("Phase: {:?}", self.phase),
                20.0,
                y_offset,
                16.0,
                WHITE,
            );
            y_offset += 20.0;

            // Stats
            draw_text(
                &format!("Kills: {} | Feedings: {}", self.kills, self.feeding_count),
                20.0,
                y_offset,
                14.0,
                GRAY,
            );
            y_offset += 20.0;

            // Objectives
            draw_text("Objectives:", 20.0, y_offset, 18.0, YELLOW);
            y_offset += 25.0;

            for objective in &self.phase_objectives {
                draw_text(&format!("• {}", objective), 30.0, y_offset, 14.0, WHITE);
                y_offset += 18.0;
            }
        }

        // Controls
        let controls_y = screen_height() - 100.0;
        draw_text(
            "Controls: WASD=Move, R=Feed, E=Interact, Space=Attack, Tab=Clans, L=Legend, H=Help, Esc=Pause",
            20.0,
            controls_y,
            16.0,
            LIGHTGRAY,
        );
    }

    fn draw_pause_menu(&self) {
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            Color::new(0.0, 0.0, 0.0, 0.7),
        );

        let center_x = screen_width() / 2.0;
        let center_y = screen_height() / 2.0;

        draw_text("PAUSED", center_x - 50.0, center_y - 50.0, 36.0, WHITE);
        draw_text(
            "Press ESC to Resume",
            center_x - 80.0,
            center_y,
            20.0,
            WHITE,
        );
    }

    fn draw_clan_menu(&self) {
        draw_rectangle(
            50.0,
            50.0,
            screen_width() - 100.0,
            screen_height() - 100.0,
            Color::new(0.1, 0.1, 0.2, 0.9),
        );

        draw_text("CLAN RELATIONS", 70.0, 80.0, 24.0, WHITE);

        let mut y = 120.0;
        for clan in self.clans.values() {
            let status_color = if clan.is_allied { GREEN } else { RED };

            draw_text(&clan.name, 70.0, y, 20.0, WHITE);
            draw_text(
                &format!("Leader: {}", clan.leader_name),
                200.0,
                y,
                16.0,
                GRAY,
            );
            draw_text(
                &format!("Members: {}", clan.member_count),
                350.0,
                y,
                16.0,
                GRAY,
            );
            draw_text(
                &format!("Trust: {:.1}", clan.trust_towards_player),
                450.0,
                y,
                16.0,
                GRAY,
            );

            let status = if clan.is_allied { "Allied" } else { "Neutral" };
            draw_text(status, 550.0, y, 16.0, status_color);

            y += 25.0;
        }

        draw_text(
            "Press TAB to close",
            70.0,
            screen_height() - 80.0,
            16.0,
            LIGHTGRAY,
        );
    }

    fn draw_legend(&self) {
        // Semi-transparent background
        draw_rectangle(
            screen_width() - 320.0,
            50.0,
            270.0,
            400.0,
            Color::new(0.0, 0.0, 0.0, 0.8),
        );

        // Legend title
        draw_text("LEGEND", screen_width() - 310.0, 80.0, 24.0, WHITE);

        let mut y = 110.0;
        let legend_x = screen_width() - 310.0;
        let color_size = 15.0;
        let text_offset = 25.0;

        // Player - diamond with border
        draw_rectangle(legend_x, y, color_size, color_size, RED);
        draw_rectangle_lines(legend_x, y, color_size, color_size, 1.0, WHITE);
        draw_text(
            "Player (You) - Red square with border",
            legend_x + text_offset,
            y + 12.0,
            14.0,
            WHITE,
        );
        y += 25.0;

        // Clan Leaders with crown symbol
        draw_rectangle(legend_x, y, color_size, color_size, BEIGE);
        draw_triangle(
            Vec2::new(legend_x + color_size / 2.0, y - 2.0),
            Vec2::new(legend_x + color_size / 2.0 - 3.0, y + 3.0),
            Vec2::new(legend_x + color_size / 2.0 + 3.0, y + 3.0),
            GOLD,
        );
        draw_text(
            "Bone-Eaters Leader (Crown)",
            legend_x + text_offset,
            y + 12.0,
            14.0,
            WHITE,
        );
        y += 20.0;

        draw_rectangle(legend_x, y, color_size, color_size, PURPLE);
        draw_triangle(
            Vec2::new(legend_x + color_size / 2.0, y - 2.0),
            Vec2::new(legend_x + color_size / 2.0 - 3.0, y + 3.0),
            Vec2::new(legend_x + color_size / 2.0 + 3.0, y + 3.0),
            GOLD,
        );
        draw_text(
            "Flame-Haters Leader (Crown)",
            legend_x + text_offset,
            y + 12.0,
            14.0,
            WHITE,
        );
        y += 20.0;

        draw_rectangle(legend_x, y, color_size, color_size, DARKBLUE);
        draw_triangle(
            Vec2::new(legend_x + color_size / 2.0, y - 2.0),
            Vec2::new(legend_x + color_size / 2.0 - 3.0, y + 3.0),
            Vec2::new(legend_x + color_size / 2.0 + 3.0, y + 3.0),
            GOLD,
        );
        draw_text(
            "Night-Bloods Leader (Crown)",
            legend_x + text_offset,
            y + 12.0,
            14.0,
            WHITE,
        );
        y += 25.0;

        // Enemies with X marks
        draw_rectangle(
            legend_x,
            y,
            color_size,
            color_size,
            Color::new(0.5, 0.1, 0.1, 1.0),
        );
        // Draw X mark
        draw_line(
            legend_x + 3.0,
            y + 3.0,
            legend_x + color_size - 3.0,
            y + color_size - 3.0,
            1.0,
            RED,
        );
        draw_line(
            legend_x + color_size - 3.0,
            y + 3.0,
            legend_x + 3.0,
            y + color_size - 3.0,
            1.0,
            RED,
        );
        draw_text(
            "Hostile Infected (X mark)",
            legend_x + text_offset,
            y + 12.0,
            14.0,
            WHITE,
        );
        y += 20.0;

        // Animals - circles
        draw_circle(
            legend_x + color_size / 2.0,
            y + color_size / 2.0,
            color_size / 2.0,
            BROWN,
        );
        draw_text(
            "Animals - Circles (Blood Source)",
            legend_x + text_offset,
            y + 12.0,
            14.0,
            WHITE,
        );
        y += 30.0;

        // Game mechanics legend
        draw_text("GAME TIPS:", legend_x, y, 18.0, YELLOW);
        y += 25.0;

        draw_text("• Red entities are hostile", legend_x, y, 14.0, LIGHTGRAY);
        y += 18.0;

        draw_text("• Feed on animals for blood", legend_x, y, 14.0, LIGHTGRAY);
        y += 18.0;

        draw_text("• Interact with clan leaders", legend_x, y, 14.0, LIGHTGRAY);
        y += 18.0;

        draw_text("• Avoid sunlight during day", legend_x, y, 14.0, LIGHTGRAY);
        y += 18.0;

        draw_text("• Watch your blood meter", legend_x, y, 14.0, LIGHTGRAY);
        y += 18.0;

        draw_text(
            "• Health bars show above entities",
            legend_x,
            y,
            14.0,
            LIGHTGRAY,
        );
        y += 25.0;

        draw_text("Press L to close", legend_x, y, 16.0, YELLOW);
    }

    fn draw_quick_start_guide(&self) {
        // Full screen overlay
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            Color::new(0.0, 0.0, 0.0, 0.85),
        );

        let center_x = screen_width() / 2.0;
        let mut y = 80.0;

        // Title
        draw_text(
            "VAMPIRE RPG - QUICK START GUIDE",
            center_x - 200.0,
            y,
            32.0,
            RED,
        );
        y += 60.0;

        // Story intro
        draw_text(
            "You are the sole survivor of a viral outbreak that created vampires.",
            center_x - 250.0,
            y,
            18.0,
            WHITE,
        );
        y += 25.0;

        draw_text(
            "You must survive, adapt, and eventually rule the savage clans.",
            center_x - 220.0,
            y,
            18.0,
            WHITE,
        );
        y += 40.0;

        // Essential controls
        draw_text("ESSENTIAL CONTROLS:", center_x - 100.0, y, 20.0, YELLOW);
        y += 30.0;

        draw_text("WASD - Move around", center_x - 150.0, y, 16.0, LIGHTGRAY);
        y += 20.0;

        draw_text(
            "R - Feed on animals and enemies (restores blood & health)",
            center_x - 200.0,
            y,
            16.0,
            LIGHTGRAY,
        );
        y += 20.0;

        draw_text(
            "Space - Attack hostile infected (red squares with X)",
            center_x - 180.0,
            y,
            16.0,
            LIGHTGRAY,
        );
        y += 20.0;

        draw_text(
            "E - Interact with clan leaders (squares with gold crowns)",
            center_x - 200.0,
            y,
            16.0,
            LIGHTGRAY,
        );
        y += 30.0;

        // Survival tips
        draw_text("SURVIVAL TIPS:", center_x - 70.0, y, 20.0, YELLOW);
        y += 30.0;

        draw_text(
            "• Keep your BLOOD meter above 20% or you'll take damage",
            center_x - 200.0,
            y,
            16.0,
            LIGHTGRAY,
        );
        y += 20.0;

        draw_text(
            "• Avoid sunlight during DAY - it damages you significantly",
            center_x - 200.0,
            y,
            16.0,
            LIGHTGRAY,
        );
        y += 20.0;

        draw_text(
            "• Feed on brown circles (animals) for easy blood",
            center_x - 180.0,
            y,
            16.0,
            LIGHTGRAY,
        );
        y += 20.0;

        draw_text(
            "• Build trust with clan leaders by repeatedly pressing E near them",
            center_x - 220.0,
            y,
            16.0,
            LIGHTGRAY,
        );
        y += 20.0;

        draw_text(
            "• Your abilities improve each time you feed",
            center_x - 160.0,
            y,
            16.0,
            LIGHTGRAY,
        );
        y += 40.0;

        // Legend reference
        draw_text(
            "Press L for detailed LEGEND • Press Tab for CLAN RELATIONS",
            center_x - 200.0,
            y,
            16.0,
            YELLOW,
        );
        y += 40.0;

        // Close instructions
        draw_text(
            "Press H to toggle this guide • Start moving (WASD) to begin!",
            center_x - 200.0,
            y,
            18.0,
            GREEN,
        );
    }
}

/// Window configuration for the game
fn window_conf() -> Conf {
    Conf {
        window_title: "Vampire RPG - The First Immortal".to_owned(),
        window_width: 1280,
        window_height: 720,
        window_resizable: false,
        fullscreen: false,
        sample_count: 4,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // Initialize random seed
    rand::srand(macroquad::miniquad::date::now() as u64);

    // Create game state
    let mut game_state = GameState::new();
    let mut last_time = get_time();

    // Main game loop
    loop {
        // Calculate delta time
        let current_time = get_time();
        let delta_time = (current_time - last_time) as f32;
        last_time = current_time;

        // Cap delta time to prevent large jumps
        let delta_time = delta_time.min(1.0 / 30.0);

        // Update game state
        game_state.update(delta_time);

        // Render the game
        game_state.render();

        // Handle window close
        if is_key_pressed(KeyCode::Q) && is_key_down(KeyCode::LeftControl) {
            break;
        }

        // Present frame
        next_frame().await;
    }
}
