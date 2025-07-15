//! Game State Module
//!
//! This module contains the core GameState that coordinates all game systems.
//! The GameState is now a lean coordinator that delegates specific responsibilities
//! to focused systems, following the Single Responsibility Principle.

use crate::components::*;
use crate::systems::*;
use crate::InputHandler;
use macroquad::prelude::*;
use std::collections::HashMap;

/// Core game state that coordinates all systems and manages game data
pub struct GameState {
    // Entity management
    pub entities: Vec<GameEntity>,
    pub next_entity_id: u32,
    pub player_id: u32,

    // Game systems
    pub time: TimeSystem,
    pub phase: GamePhase,

    // Game data
    pub clans: HashMap<String, Clan>,
    pub camera_x: f32,
    pub camera_y: f32,
    pub phase_objectives: Vec<String>,
    pub completed_objectives: Vec<String>,
    pub game_time: f32,
    pub kills: u32,
    pub feeding_count: u32,

    // Environment
    pub stars: Vec<Star>,
    pub moon: Moon,
    pub blood_particles: Vec<BloodParticle>,
    pub ground_tiles: Vec<GroundTile>,

    // Debug message log
    pub debug_messages: Vec<String>,

    // UI state
    pub paused: bool,
    pub show_clan_menu: bool,
    pub show_legend: bool,
    pub show_quick_start: bool,
}

impl GameState {
    /// Create a new game state with all systems initialized
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
            phase_objectives: ObjectivesSystem::get_initial_objectives(
                &GamePhase::SurvivalAndDiscovery,
            ),
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
            debug_messages: Vec::new(),
        };

        // Initialize the world using the world system
        state.player_id = WorldSystem::initialize_world(
            &mut state.entities,
            &mut state.clans,
            &mut state.stars,
            &mut state.moon,
            &mut state.ground_tiles,
            &mut state.next_entity_id,
        );

        state
    }

    /// Main update loop that coordinates all systems
    pub fn update(&mut self, input_handler: &InputHandler, delta_time: f32) {
        // Handle UI input first
        self.handle_ui_input(input_handler);

        // Skip game updates if paused or showing menus
        if self.paused || self.show_clan_menu || self.show_legend || self.show_quick_start {
            return;
        }

        // Update game time
        self.game_time += delta_time;

        // Debug: print all entities' type, position, and health at start of each frame
        println!("--- Entities at start of frame ---");
        for entity in &self.entities {
            let health = entity.health.as_ref().map(|h| h.current).unwrap_or(-1.0);
            println!(
                "Entity: {:?}, Pos: ({:.1}, {:.1}), Health: {}",
                entity.entity_type, entity.position.x, entity.position.y, health
            );
        }
        println!("----------------------------------");

        // System updates in order of dependency
        self.update_time_system(delta_time);
        self.update_environment(delta_time);
        self.update_player_system(input_handler, delta_time);
        self.update_ai_system(delta_time);
        self.update_blood_system(delta_time);
        self.update_objectives_system();
        self.update_camera();
        self.update_phase_progression();
    }

    /// Handle UI-related input (menus, pause, etc.)
    fn handle_ui_input(&mut self, input_handler: &InputHandler) {
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
    }

    /// Update the time system
    fn update_time_system(&mut self, delta_time: f32) {
        self.time.update(delta_time);
    }

    /// Update environmental elements
    fn update_environment(&mut self, delta_time: f32) {
        // Update stars
        for star in &mut self.stars {
            star.update(self.game_time);
        }

        // Update moon
        self.moon.update(self.game_time);

        // Update blood particles
        BloodSystem::update_blood_particles(&mut self.blood_particles, delta_time);
    }

    /// Update player-related systems
    fn update_player_system(&mut self, input_handler: &InputHandler, delta_time: f32) {
        // Handle player input and actions
        PlayerSystem::handle_input(
            &mut self.entities,
            input_handler,
            self.player_id,
            self.game_time,
        );

        // Update player movement
        PlayerSystem::update_movement(
            &mut self.entities,
            input_handler,
            self.player_id,
            self.time.is_day(),
            delta_time,
        );

        // Handle feeding attempts and update feeding counter
        if input_handler.is_key_just_pressed(KeyCode::R) {
            let mut debug_messages = Vec::new();
            if let Some(feed_pos) = PlayerSystem::attempt_feeding(
                &mut self.entities,
                self.player_id,
                &mut debug_messages,
            ) {
                self.feeding_count += 1;
                debug_messages.push(format!(
                    "FEEDING SUCCESS! Creating blood particles at ({}, {})",
                    feed_pos.x, feed_pos.y
                ));

                // Create blood particle effects at the fed-upon entity's position
                BloodSystem::create_blood_particles(
                    &mut self.blood_particles,
                    feed_pos.x,
                    feed_pos.y,
                    8,
                    &mut debug_messages,
                );
            } else {
                debug_messages.push("FEEDING FAILED - no target position returned".to_string());
            }

            // Add all debug messages after the feeding attempt
            for message in debug_messages {
                self.add_debug_message(message);
            }
        }

        // Handle attack attempts and update kill counter
        if input_handler.is_key_just_pressed(KeyCode::Space) {
            if let Some(target_pos) =
                PlayerSystem::attempt_attack(&mut self.entities, self.player_id, self.game_time)
            {
                self.kills += 1;

                // Create blood particle effects at the attacked entity's position
                let mut attack_debug_messages = Vec::new();
                BloodSystem::create_blood_particles(
                    &mut self.blood_particles,
                    target_pos.x,
                    target_pos.y,
                    12, // More particles for combat
                    &mut attack_debug_messages,
                );
                for message in attack_debug_messages {
                    self.add_debug_message(message);
                }
            }
        }

        // Handle clan interactions
        if input_handler.is_key_just_pressed(KeyCode::E) {
            if let Some(clan_name) =
                PlayerSystem::attempt_interaction(&mut self.entities, self.player_id)
            {
                self.interact_with_clan(&clan_name);
            }
        }
    }

    /// Update AI system for all NPCs
    fn update_ai_system(&mut self, delta_time: f32) {
        AISystem::update_all_ai(&mut self.entities, self.player_id, delta_time);
    }

    /// Update blood system and related mechanics
    fn update_blood_system(&mut self, delta_time: f32) {
        BloodSystem::update_blood_system(
            &mut self.entities,
            self.time.is_day(),
            self.time.get_sunlight_intensity(),
            delta_time,
        );
    }

    /// Update objectives and check for completions
    fn update_objectives_system(&mut self) {
        ObjectivesSystem::check_objectives(
            &self.entities,
            self.player_id,
            &self.time,
            &self.clans,
            self.kills,
            self.feeding_count,
            &mut self.phase_objectives,
            &mut self.completed_objectives,
        );
    }

    /// Update camera to follow player
    fn update_camera(&mut self) {
        if let Some(player) = self.entities.iter().find(|e| e.id == self.player_id) {
            self.camera_x = player.position.x;
            self.camera_y = player.position.y;
        }
    }

    /// Check for and handle phase progression
    fn update_phase_progression(&mut self) {
        let allied_clans = self.clans.values().filter(|clan| clan.is_allied).count();

        if ObjectivesSystem::can_advance_phase(
            &self.phase,
            &self.completed_objectives,
            self.time.day_count(),
            allied_clans,
        ) {
            if let Some(next_phase) = ObjectivesSystem::get_next_phase(&self.phase) {
                self.advance_to_phase(next_phase);
            }
        }
    }

    /// Advance to the next game phase
    fn advance_to_phase(&mut self, new_phase: GamePhase) {
        self.phase = new_phase.clone();

        // Add new objectives for the new phase
        let mut new_objectives = ObjectivesSystem::get_initial_objectives(&new_phase);
        self.phase_objectives.append(&mut new_objectives);
    }

    /// Handle clan interaction logic
    fn interact_with_clan(&mut self, clan_name: &str) {
        if let Some(clan) = self.clans.get_mut(clan_name) {
            clan.trust_towards_player += 0.1;
            clan.trust_towards_player = clan.trust_towards_player.min(1.0);

            // Check if clan should become allied
            if clan.trust_towards_player > 0.7 && !clan.is_allied {
                clan.is_allied = true;
            }
        }
    }

    /// Get player status for UI display
    pub fn get_player_status(&self) -> Option<PlayerStatus> {
        PlayerSystem::get_player_status(&self.entities, self.player_id)
    }

    /// Get current blood status for the player
    pub fn get_player_blood_status(&self) -> BloodStatus {
        if let Some(player) = self.entities.iter().find(|e| e.id == self.player_id) {
            BloodSystem::check_blood_status(player)
        } else {
            BloodStatus::None
        }
    }

    /// Get current objectives progress
    pub fn get_objectives_progress(&self) -> ObjectiveProgress {
        ObjectivesSystem::get_progress_summary(
            &self.completed_objectives,
            &self.phase_objectives,
            &self.phase,
        )
    }

    /// Spawn a new entity using the world system
    pub fn spawn_entity(&mut self, entity_type: EntityType, x: f32, y: f32) -> Option<u32> {
        match entity_type {
            EntityType::HostileInfected => Some(WorldSystem::spawn_hostile_infected(
                &mut self.entities,
                &mut self.next_entity_id,
                x,
                y,
            )),
            EntityType::Animal => Some(WorldSystem::spawn_animal(
                &mut self.entities,
                &mut self.next_entity_id,
                x,
                y,
            )),
            EntityType::ClanMember(clan_name) => {
                let color = match clan_name.as_str() {
                    "Bone-Eaters" => LIGHTGRAY,
                    "Flame-Haters" => VIOLET,
                    "Night-Bloods" => BLUE,
                    _ => WHITE,
                };
                Some(WorldSystem::spawn_clan_member(
                    &mut self.entities,
                    &mut self.next_entity_id,
                    &clan_name,
                    x,
                    y,
                    color,
                ))
            }
            _ => None, // Other entity types not supported for dynamic spawning
        }
    }

    /// Check if the game is over (player dead)
    pub fn is_game_over(&self) -> bool {
        if let Some(player) = self.entities.iter().find(|e| e.id == self.player_id) {
            if let Some(health) = &player.health {
                health.current <= 0.0
            } else {
                true
            }
        } else {
            true
        }
    }

    /// Get survival statistics
    pub fn get_survival_stats(&self) -> SurvivalScore {
        BloodSystem::calculate_survival_score(self.feeding_count, self.time.day_count(), self.kills)
    }

    /// Find the position of the target entity that would be attacked
    fn find_attack_target_position(&self) -> Option<Position> {
        let player_pos = self
            .entities
            .iter()
            .find(|e| e.id == self.player_id)?
            .position;

        let attack_range = 60.0;

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
                            return Some(entity.position);
                        }
                    }
                }
            }
        }

        None
    }

    /// Add a debug message to the log
    pub fn add_debug_message(&mut self, message: String) {
        self.debug_messages.push(message);
        // Keep only the last 20 messages
        if self.debug_messages.len() > 20 {
            self.debug_messages.remove(0);
        }
    }

    /// Reset game to initial state
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_state_creation() {
        let game_state = GameState::new();
        assert!(!game_state.entities.is_empty());
        assert_eq!(game_state.player_id, 0);
        assert!(!game_state.phase_objectives.is_empty());
    }

    #[test]
    fn test_clan_interaction() {
        let mut game_state = GameState::new();
        let initial_trust = game_state
            .clans
            .get("Bone-Eaters")
            .unwrap()
            .trust_towards_player;

        game_state.interact_with_clan("Bone-Eaters");

        let new_trust = game_state
            .clans
            .get("Bone-Eaters")
            .unwrap()
            .trust_towards_player;
        assert!(new_trust > initial_trust);
    }

    #[test]
    fn test_is_game_over() {
        let mut game_state = GameState::new();
        assert!(!game_state.is_game_over());

        // Kill the player
        if let Some(player) = game_state
            .entities
            .iter_mut()
            .find(|e| e.id == game_state.player_id)
        {
            if let Some(health) = &mut player.health {
                health.current = 0.0;
            }
        }

        assert!(game_state.is_game_over());
    }
}
