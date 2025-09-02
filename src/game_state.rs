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

    // Horizon movement tracking (preserves current visuals)
    pub last_player_y: f32,
    pub is_moving_toward_horizon: bool,
    pub ground_update_timer: f32,
    pub accumulated_horizon_movement: f32,
    pub fog_cache_invalidated: bool,

    // Virtual horizon walking system
    pub virtual_horizon_distance: f32,
    pub world_scroll_offset: f32,
    pub player_attempting_horizon_movement: bool,

    // Performance optimization - cached player reference
    pub player_entity_index: Option<usize>,

    // Ground generation tracking for all directions
    pub last_player_x: f32,
    pub movement_threshold: f32,

    // Debug message log (ring buffer for performance)
    pub debug_messages: Vec<String>,
    pub debug_message_index: usize,
    pub max_debug_messages: usize,

    // UI state
    pub paused: bool,
    pub show_clan_menu: bool,
    pub show_legend: bool,
    pub show_quick_start: bool,
    pub show_debug_messages: bool,

    // Performance optimization fields
    pub ai_system: crate::systems::ai::AISystem,
    pub entity_pool: crate::systems::entity_pool::EntityPool,
    pub spatial_grid: crate::systems::spatial_grid::SpatialGrid,
    pub max_entities: usize,
    pub entity_cleanup_distance: f32,
    pub spawning_system: crate::systems::spawning::SpawningSystem,
    pub exploration_system: crate::systems::exploration::ExplorationSystem,
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
            show_debug_messages: true,
            game_time: 0.0,
            kills: 0,
            feeding_count: 0,
            stars: Vec::new(),
            moon: Moon::new(),
            blood_particles: Vec::new(),
            ground_tiles: Vec::new(),
            last_player_y: crate::systems::world::GROUND_LEVEL,
            is_moving_toward_horizon: false,
            ground_update_timer: 0.0,
            accumulated_horizon_movement: 0.0,
            fog_cache_invalidated: false,
            virtual_horizon_distance: 0.0,
            world_scroll_offset: 0.0,
            player_attempting_horizon_movement: false,
            player_entity_index: None,
            last_player_x: 400.0,     // Player spawn X position
            movement_threshold: 32.0, // Generate ground when player moves 32 pixels
            debug_messages: Vec::with_capacity(50),
            debug_message_index: 0,
            max_debug_messages: 50,
            // Performance optimization fields
            ai_system: crate::systems::ai::AISystem::new(),
            entity_pool: crate::systems::entity_pool::EntityPool::with_defaults(),
            spatial_grid: crate::systems::spatial_grid::SpatialGrid::new(100.0),
            max_entities: 100,
            entity_cleanup_distance: 800.0,
            spawning_system: crate::systems::spawning::SpawningSystem::new(),
            exploration_system: crate::systems::exploration::ExplorationSystem::new(),
        };

        // Initialize the world using the world system
        state.player_id = WorldSystem::initialize_world(
            &mut state.entities,
            &mut state.clans,
            &mut state.stars,
            &mut state.moon,
            &mut state.ground_tiles,
            &mut state.next_entity_id,
            &mut state.debug_messages,
        );

        // Initialize exploration system with initial ground tiles
        for tile in &state.ground_tiles {
            state.exploration_system.add_ground_tile(tile.clone(), 0.0);
        }

        // Mark initial area around player as explored
        if let Some(player) = state.entities.iter().find(|e| e.id == state.player_id) {
            state.exploration_system.mark_area_explored(
                player.position.x,
                player.position.y,
                200.0, // Initial exploration radius
                0.0,   // Initial time
            );
        }

        state
    }

    /// Main update loop that coordinates all systems
    /// Main update function called every frame
    pub fn update(&mut self, input_handler: &InputHandler, delta_time: f32) {
        // Handle UI input first
        self.handle_ui_input(input_handler);

        // Skip game updates if paused or showing menus
        if self.paused || self.show_clan_menu || self.show_legend || self.show_quick_start {
            return;
        }

        // Update game time
        self.game_time += delta_time;

        // Note: Cleanup is now handled by main loop using macroquad's frame timing
        // This improves performance by aligning with vsync and frame presentation

        // System updates in order of dependency
        self.update_time_system(delta_time);
        self.update_environment(delta_time);
        self.update_player_system(input_handler, delta_time);
        self.update_ai_system(delta_time);
        self.update_shelter_system(delta_time);
        self.update_blood_system(delta_time);
        self.update_objectives_system();
        self.update_horizon_movement_detection();
        self.update_horizon_ground_effect();
        self.ensure_ground_near_player();
        self.update_camera();
        self.update_phase_progression();
        self.update_spawning_system();
        self.update_exploration_system();

        // Debug information every 5 seconds using macroquad timing
        if (self.game_time * 0.2).floor() != ((self.game_time - delta_time) * 0.2).floor() {
            self.add_performance_debug_info();
        }
    }

    /// Efficient batch cleanup using macroquad's vector operations for optimal performance
    pub fn batch_cleanup_with_macroquad(&mut self) {
        use macroquad::prelude::vec2;

        let player_pos = self.get_cached_player_data().map(|(pos, _)| pos);
        if player_pos.is_none() {
            return;
        }

        let player_position = player_pos.unwrap();
        let player_vec = vec2(player_position.x, player_position.y);
        let cleanup_distance_sq = self.entity_cleanup_distance * self.entity_cleanup_distance;

        let _initial_count = self.entities.len();
        let mut removed_count = 0;

        // Use macroquad's efficient vector operations for batch distance checking
        let mut entities_to_keep = Vec::with_capacity(self.entities.len());

        for entity in self.entities.drain(..) {
            let should_keep = if entity.id == self.player_id {
                true // Never remove player
            } else {
                let entity_vec = vec2(entity.position.x, entity.position.y);
                let distance_sq = player_vec.distance_squared(entity_vec);
                let is_alive = !matches!(entity.ai_state, AIState::Dead)
                    && !entity.health.as_ref().map_or(false, |h| h.current <= 0.0);

                distance_sq < cleanup_distance_sq && is_alive
            };

            if should_keep {
                entities_to_keep.push(entity);
            } else {
                // Return to pool for reuse
                if entity.id != self.player_id {
                    self.entity_pool.release(entity);
                    removed_count += 1;
                }
            }
        }

        self.entities = entities_to_keep;

        if removed_count > 0 {
            self.add_debug_message(format!(
                "Batch cleanup: removed {} entities using macroquad vectors",
                removed_count
            ));
        }
    }

    /// Clean up entities that are too far from the player using macroquad vector operations
    /// Note: Ground tiles are persistent and handled by the exploration system
    fn cleanup_distant_entities(&mut self) {
        use macroquad::prelude::vec2;

        let player_pos = self.get_cached_player_data().map(|(pos, _)| pos);

        if let Some(player_position) = player_pos {
            let initial_count = self.entities.len();
            let player_vec = vec2(player_position.x, player_position.y);
            let cleanup_distance_sq = self.entity_cleanup_distance * self.entity_cleanup_distance;

            // Use batch processing with Vec2 for efficient distance calculations
            self.entities.retain(|entity| {
                if entity.id == self.player_id {
                    return true; // Never remove player
                }

                let entity_vec = vec2(entity.position.x, entity.position.y);
                let distance_sq = player_vec.distance_squared(entity_vec);

                distance_sq < cleanup_distance_sq
            });

            let removed_count = initial_count - self.entities.len();
            if removed_count > 0 {
                self.add_debug_message(format!("Cleaned up {} distant entities", removed_count));
            }
        }
    }

    /// Clean up dead entities and return them to the pool using efficient batch processing
    /// Note: Ground tiles are persistent and handled by the exploration system
    fn cleanup_dead_entities(&mut self) {
        let initial_count = self.entities.len();
        let mut dead_entities = Vec::new();

        // Collect dead entities first to avoid borrowing issues
        let mut i = 0;
        while i < self.entities.len() {
            let entity = &self.entities[i];

            let is_dead = matches!(entity.ai_state, AIState::Dead)
                || entity.health.as_ref().map_or(false, |h| h.current <= 0.0);

            if is_dead && entity.id != self.player_id {
                dead_entities.push(self.entities.remove(i));
            } else {
                i += 1;
            }
        }

        // Return dead entities to pool for reuse
        for dead_entity in dead_entities {
            self.entity_pool.release(dead_entity);
        }

        let removed_count = initial_count - self.entities.len();
        if removed_count > 0 {
            self.add_debug_message(format!("Cleaned up {} dead entities", removed_count));
        }
    }

    /// Add performance debug information using macroquad timing
    fn add_performance_debug_info(&mut self) {
        use macroquad::prelude::get_fps;

        let ai_stats = self.ai_system.get_ai_statistics();
        let spawn_stats = self.spawning_system.get_stats().clone();
        let exploration_stats = self.exploration_system.get_stats();
        let current_fps = get_fps();

        self.add_debug_message(format!(
            "FPS: {} | Entities: {}/{} | Pool: {} | AI Cache: {} | Grid: {} cells",
            current_fps,
            self.entities.len(),
            self.max_entities,
            self.entity_pool.current_size(),
            ai_stats.cached_entities,
            ai_stats.spatial_grid_cells
        ));

        // Count entities by type for spawn monitoring
        let hostile_count = self
            .entities
            .iter()
            .filter(|e| {
                matches!(
                    e.entity_type,
                    crate::components::EntityType::HostileInfected
                ) && !matches!(e.ai_state, crate::components::AIState::Dead)
            })
            .count();
        let animal_count = self
            .entities
            .iter()
            .filter(|e| {
                matches!(e.entity_type, crate::components::EntityType::Animal)
                    && !matches!(e.ai_state, crate::components::AIState::Dead)
            })
            .count();
        let clan_count = self
            .entities
            .iter()
            .filter(|e| {
                matches!(e.entity_type, crate::components::EntityType::ClanMember(_))
                    && !matches!(e.ai_state, crate::components::AIState::Dead)
            })
            .count();

        self.add_debug_message(format!(
            "Spawned: {} | H:{} A:{} C:{} | Fails:{} | Ground:{} | Explored:{}",
            spawn_stats.total_spawned,
            hostile_count,
            animal_count,
            clan_count,
            spawn_stats.failed_spawns,
            exploration_stats.persistent_ground_tiles,
            exploration_stats.explored_regions
        ));

        if ai_stats.performance_mode {
            self.add_debug_message("Performance mode: ENABLED".to_string());
        }

        // Memory usage info
        let memory_mb = ai_stats.memory_usage as f32 / 1024.0 / 1024.0;
        self.add_debug_message(format!(
            "Memory: {:.2} MB | AI Range: {:.0}",
            memory_mb, ai_stats.ai_range
        ));
    }

    /// Toggle performance mode for all systems
    pub fn set_performance_mode(&mut self, enabled: bool) {
        self.ai_system.set_performance_mode(enabled);
        self.spawning_system.set_performance_mode(enabled);
        self.exploration_system.set_performance_mode(enabled);

        if enabled {
            self.max_entities = 75;
            self.entity_cleanup_distance = 600.0;
            self.add_debug_message("Performance mode enabled - reduced limits".to_string());
        } else {
            self.max_entities = 100;
            self.entity_cleanup_distance = 800.0;
            self.add_debug_message("Performance mode disabled - normal limits".to_string());
        }
    }

    /// Check if we're at entity limit and should avoid spawning more
    pub fn at_entity_limit(&self) -> bool {
        self.entities.len() >= self.max_entities
    }

    /// Force immediate cleanup using macroquad's efficient operations
    pub fn force_cleanup(&mut self) {
        use macroquad::prelude::get_time;

        let start_time = get_time();

        // Use the more efficient batch cleanup with macroquad vectors
        // Note: Ground tiles are now handled by the exploration system and are persistent
        self.batch_cleanup_with_macroquad();

        // Trigger immediate respawning to refill the world
        self.spawning_system.reset_timers(self.game_time);
        self.update_spawning_system();

        let cleanup_time = get_time() - start_time;

        self.add_debug_message(format!(
            "Force cleanup executed in {:.2}ms + respawning triggered",
            cleanup_time * 1000.0
        ));
    }

    /// Manually trigger spawning of a specific entity type (for debugging)
    pub fn debug_spawn_entities(&mut self, entity_type: &str, count: usize) {
        for _ in 0..count {
            if self.entities.len() >= self.max_entities {
                break;
            }

            if let Some(player_pos) = self.get_cached_player_data().map(|(pos, _)| pos) {
                // Spawn near player but not too close
                let angle = rand::gen_range(0.0, 2.0 * std::f32::consts::PI);
                let distance = rand::gen_range(150.0, 300.0);
                let spawn_x = player_pos.x + angle.cos() * distance;
                let spawn_y = player_pos.y + angle.sin() * distance;

                match entity_type {
                    "hostile" => {
                        crate::systems::world::WorldSystem::spawn_hostile_infected(
                            &mut self.entities,
                            &mut self.next_entity_id,
                            spawn_x,
                            spawn_y,
                        );
                        self.add_debug_message("Debug spawned hostile entity".to_string());
                    }
                    "animal" => {
                        crate::systems::world::WorldSystem::spawn_animal(
                            &mut self.entities,
                            &mut self.next_entity_id,
                            spawn_x,
                            spawn_y,
                        );
                        self.add_debug_message("Debug spawned animal".to_string());
                    }
                    "clan_member" => {
                        let clan_names = ["Bone-Eaters", "Flame-Haters", "Night-Bloods"];
                        let clan_name = clan_names[rand::gen_range(0, clan_names.len())];
                        let color = match clan_name {
                            "Bone-Eaters" => macroquad::prelude::LIGHTGRAY,
                            "Flame-Haters" => macroquad::prelude::VIOLET,
                            "Night-Bloods" => macroquad::prelude::BLUE,
                            _ => macroquad::prelude::WHITE,
                        };
                        crate::systems::world::WorldSystem::spawn_clan_member(
                            &mut self.entities,
                            &mut self.next_entity_id,
                            clan_name,
                            spawn_x,
                            spawn_y,
                            color,
                        );
                        self.add_debug_message(format!("Debug spawned {} clan member", clan_name));
                    }
                    _ => {
                        self.add_debug_message(format!("Unknown entity type: {}", entity_type));
                    }
                }
            }
        }
    }

    /// Spawn entity using the pool system for better performance
    pub fn spawn_pooled_entity(&mut self, entity_type: EntityType, x: f32, y: f32) -> Option<u32> {
        if self.at_entity_limit() {
            return None;
        }

        let entity =
            self.entity_pool
                .spawn_entity(entity_type, Position { x, y }, &mut self.next_entity_id);

        let entity_id = entity.id;
        self.entities.push(entity);
        Some(entity_id)
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

        if input_handler.is_key_just_pressed(KeyCode::M) {
            self.show_debug_messages = !self.show_debug_messages;
            let status = if self.show_debug_messages {
                "enabled"
            } else {
                "disabled"
            };

            self.add_debug_message(format!("Debug messages {}", status));
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

        // Handle shelter interaction
        if input_handler.is_key_just_pressed(KeyCode::F) {
            if let Some(message) = ShelterSystem::handle_player_shelter_interaction(
                &mut self.entities,
                self.player_id,
                self.game_time,
            ) {
                self.add_debug_message(format!("Shelter: {}", message));
            }
        }

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
        self.ai_system
            .update_all_ai(&mut self.entities, self.player_id, delta_time);
    }

    /// Update the spawning system to maintain entity populations
    fn update_spawning_system(&mut self) {
        self.spawning_system.update(
            &mut self.entities,
            &mut self.next_entity_id,
            self.player_id,
            self.game_time,
            self.max_entities,
        );
    }

    /// Update the exploration system for persistent ground and fog management
    fn update_exploration_system(&mut self) {
        if let Some((player_pos, _)) = self.get_cached_player_data() {
            // Update exploration based on player position
            self.exploration_system
                .update_exploration(player_pos.x, player_pos.y, self.game_time);

            // Ensure ground exists near player
            self.exploration_system.ensure_ground_near_player(
                player_pos.x,
                player_pos.y,
                self.game_time,
            );

            // Sync exploration system ground tiles with legacy ground_tiles for compatibility
            let exploration_ground = self.exploration_system.get_ground_tiles();

            // Only sync if there are significant differences
            if (exploration_ground.len() as i32 - self.ground_tiles.len() as i32).abs() > 10 {
                self.ground_tiles = exploration_ground;
            }
        }
    }

    /// Update shelter system
    fn update_shelter_system(&mut self, delta_time: f32) {
        ShelterSystem::update_shelters(
            &mut self.entities,
            self.game_time,
            self.time.get_sunlight_intensity(),
            delta_time,
        );
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

    /// Update camera to follow player with horizon walking support
    fn update_camera(&mut self) {
        if let Some((player_pos, _)) = self.get_cached_player_data() {
            self.camera_x = player_pos.x;
            self.camera_y = player_pos.y;
        }
    }

    /// Get cached player position and data for performance (returns copy to avoid borrow issues)
    fn get_cached_player_data(&mut self) -> Option<(Position, f32)> {
        // Update cache if invalid
        if let Some(index) = self.player_entity_index {
            if index >= self.entities.len() || self.entities[index].id != self.player_id {
                self.player_entity_index = None;
            }
        }

        // Find player if not cached
        if self.player_entity_index.is_none() {
            for (index, entity) in self.entities.iter().enumerate() {
                if entity.id == self.player_id {
                    self.player_entity_index = Some(index);
                    break;
                }
            }
        }

        // Return cached player data (copy to avoid borrowing issues)
        self.player_entity_index
            .and_then(|index| self.entities.get(index))
            .map(|player| (player.position, player.position.y))
    }

    /// Check for and handle phase progression
    fn update_phase_progression(&mut self) {
        // Use Rust 1.88+ collect_into for better performance
        let mut allied_clan_count = 0u32;
        for clan in self.clans.values() {
            if clan.is_allied {
                allied_clan_count += 1;
            }
        }

        if ObjectivesSystem::can_advance_phase(
            &self.phase,
            &self.completed_objectives,
            self.time.day_count(),
            allied_clan_count as usize,
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
        EntityFinder::by_id(&self.entities, self.player_id)
            .map(|player| BloodSystem::check_blood_status(player))
            .unwrap_or(BloodStatus::None)
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
        EntityFinder::by_id(&self.entities, self.player_id)
            .and_then(|player| player.health.as_ref())
            .map_or(true, |health| health.current <= 0.0)
    }

    /// Get survival statistics
    pub fn get_survival_stats(&self) -> SurvivalScore {
        BloodSystem::calculate_survival_score(self.feeding_count, self.time.day_count(), self.kills)
    }

    /// Get nearby shelter information for UI display
    pub fn get_nearby_shelters(&self) -> Vec<ShelterInfo> {
        ShelterSystem::get_nearby_shelter_info(&self.entities, self.player_id, 200.0)
    }

    /// Check if player is currently in shelter
    pub fn is_player_in_shelter(&self) -> bool {
        EntityFinder::by_id(&self.entities, self.player_id)
            .and_then(|player| player.shelter_occupancy.as_ref())
            .map_or(false, |occupancy| occupancy.is_in_shelter())
    }

    /// Get current shelter protection level for player
    pub fn get_player_shelter_protection(&self) -> f32 {
        let sunlight_damage = self.time.get_sunlight_intensity() * 100.0;
        let protected_damage = ShelterSystem::calculate_shelter_protection(
            &self.entities,
            self.player_id,
            sunlight_damage,
        );
        1.0 - (protected_damage / sunlight_damage.max(1.0))
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
        // Use ring buffer for performance - no allocations or removals
        if self.debug_messages.len() < self.max_debug_messages {
            self.debug_messages.push(message);
        } else {
            // Overwrite oldest message
            self.debug_messages[self.debug_message_index] = message;
            self.debug_message_index = (self.debug_message_index + 1) % self.max_debug_messages;
        }
    }

    /// Reset game to initial state
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Clear fog cache invalidation flag (called by renderer)
    pub fn clear_fog_cache_invalidation(&mut self) {
        self.fog_cache_invalidated = false;
    }

    /// Detect horizon movement with accumulated tracking
    fn update_horizon_movement_detection(&mut self) {
        use crate::systems::world::{HORIZON_LINE, HORIZON_MOVEMENT_THRESHOLD};

        if let Some((_player_pos, current_y)) = self.get_cached_player_data() {
            let y_movement = self.last_player_y - current_y;

            // Detect when player is trying to move beyond horizon
            let player_at_horizon = (current_y - HORIZON_LINE).abs() < 1.0;
            let trying_to_move_up = y_movement > 0.0;

            self.player_attempting_horizon_movement = player_at_horizon && trying_to_move_up;

            // When player attempts to move beyond horizon, trigger world scrolling
            if self.player_attempting_horizon_movement {
                self.virtual_horizon_distance += y_movement;
                self.world_scroll_offset += y_movement;
                self.accumulated_horizon_movement += y_movement;

                self.add_debug_message(format!(
                    "Horizon walking: virtual distance {:.1}, scroll {:.1}",
                    self.virtual_horizon_distance, self.world_scroll_offset
                ));
            } else if current_y > HORIZON_LINE + 10.0 {
                // Reset when player moves significantly away from horizon
                if self.virtual_horizon_distance > 0.0 {
                    self.add_debug_message(format!(
                        "Reset virtual distance: {:.1} -> 0",
                        self.virtual_horizon_distance
                    ));
                    self.virtual_horizon_distance = 0.0;
                    self.world_scroll_offset = 0.0;
                    self.accumulated_horizon_movement = 0.0;
                }
            }

            // Update horizon movement flag
            let was_moving_toward_horizon = self.is_moving_toward_horizon;
            self.is_moving_toward_horizon = self.player_attempting_horizon_movement
                || self.accumulated_horizon_movement > HORIZON_MOVEMENT_THRESHOLD;

            // Log when horizon movement starts/stops for debugging
            if self.is_moving_toward_horizon && !was_moving_toward_horizon {
                self.add_debug_message(format!(
                    "Started horizon walking (virtual: {:.1})",
                    self.virtual_horizon_distance
                ));
            } else if !self.is_moving_toward_horizon && was_moving_toward_horizon {
                self.add_debug_message(format!(
                    "Stopped horizon walking (virtual: {:.1})",
                    self.virtual_horizon_distance
                ));
            }

            self.last_player_y = current_y;
        }
    }

    /// Apply horizon ground scrolling effect during horizon movement
    fn update_horizon_ground_effect(&mut self) {
        use crate::systems::world::{WorldSystem, GROUND_SHIFT_DISTANCE};

        if self.is_moving_toward_horizon {
            // Less frequent updates for better performance
            self.ground_update_timer += 1.0 / 60.0; // Assume 60 FPS

            // Update every 0.1 seconds instead of 0.033 for better performance
            if self.ground_update_timer >= 0.1 {
                if let Some((player_pos, _)) = self.get_cached_player_data() {
                    // Larger movement distance per update since we update less frequently
                    let movement_distance = if self.world_scroll_offset > 0.0 {
                        GROUND_SHIFT_DISTANCE * 0.3 // Larger scroll for less frequent updates
                    } else {
                        GROUND_SHIFT_DISTANCE * 0.15 // Larger normal effect
                    };

                    // Only update if there are not too many ground tiles already
                    if self.ground_tiles.len() < 200 {
                        WorldSystem::shift_ground_for_horizon_movement(
                            &mut self.ground_tiles,
                            player_pos.x,
                            player_pos.y,
                            movement_distance,
                            &mut self.debug_messages,
                        );
                    }

                    // Shift entities less frequently
                    if self.world_scroll_offset > 0.0 && self.ground_update_timer >= 0.2 {
                        self.shift_world_entities(movement_distance);
                    }

                    // Generate content less frequently
                    if self.virtual_horizon_distance > 0.0
                        && (self.virtual_horizon_distance as u32) % 50 == 0
                    {
                        self.generate_horizon_content();
                    }

                    // Signal fog cache update less frequently
                    self.fog_cache_invalidated = true;
                    self.world_scroll_offset = 0.0;
                }

                self.ground_update_timer = 0.0;
            }
        } else {
            self.ground_update_timer = 0.0;
        }
    }

    /// Ensure ground tiles exist near the player position - now handled by exploration system
    fn ensure_ground_near_player(&mut self) {
        // This is now handled by the exploration system in update_exploration_system()
        // Keeping this method for compatibility, but functionality moved to exploration system
    }

    /// Additional directional spawning for movement responsiveness
    fn spawn_directional_ground_for_player(&mut self) {
        if let Some(_) = self.get_cached_player_data() {
            // Skip directional ground spawning for now - method is private
        }
    }

    /// Shift all world entities downward to simulate world scrolling during horizon walking
    fn shift_world_entities(&mut self, shift_distance: f32) {
        let player_id = self.player_id;

        for entity in &mut self.entities {
            // Don't shift the player - they stay in place
            if entity.id != player_id {
                entity.position.y += shift_distance;
            }
        }

        self.add_debug_message(format!(
            "Shifted {} entities by {:.1} units",
            self.entities.len() - 1,
            shift_distance
        ));
    }

    /// Generate new content (enemies, shelters, etc.) during horizon walking
    fn generate_horizon_content(&mut self) {
        use crate::systems::world::WorldSystem;

        // Only generate content when virtual distance reaches certain thresholds
        let generation_threshold = 200.0; // Generate content every 200 units for less frequency
        let content_tier = (self.virtual_horizon_distance / generation_threshold) as u32;

        // Generate content based on how far the player has "walked", but limit entity count
        if content_tier > 0
            && ((content_tier as f32) * 200.0 - self.virtual_horizon_distance).abs() < 20.0
            && self.entities.len() < 50
        // Limit total entities for performance
        {
            // Generate new enemies less frequently
            if rand::gen_range(0.0, 1.0) < 0.4 {
                let spawn_x = self.camera_x + rand::gen_range(-400.0, 400.0);
                let spawn_y = crate::systems::world::HORIZON_LINE + rand::gen_range(0.0, 200.0);

                WorldSystem::spawn_hostile_infected(
                    &mut self.entities,
                    &mut self.next_entity_id,
                    spawn_x,
                    spawn_y,
                );
            }

            // Generate new animals less frequently
            if rand::gen_range(0.0, 1.0) < 0.3 {
                let spawn_x = self.camera_x + rand::gen_range(-300.0, 300.0);
                let spawn_y = crate::systems::world::HORIZON_LINE + rand::gen_range(0.0, 150.0);

                WorldSystem::spawn_animal(
                    &mut self.entities,
                    &mut self.next_entity_id,
                    spawn_x,
                    spawn_y,
                );
            }

            // Generate shelters very rarely
            if rand::gen_range(0.0, 1.0) < 0.1 {
                use crate::components::{ShelterCondition, ShelterType};
                use crate::systems::ShelterSystem;

                let spawn_x = self.camera_x + rand::gen_range(-200.0, 200.0);
                let spawn_y = crate::systems::world::HORIZON_LINE + rand::gen_range(10.0, 100.0);

                let shelter_types = [ShelterType::Cave, ShelterType::Ruins];
                let shelter_type = shelter_types[rand::gen_range(0, shelter_types.len())].clone();

                ShelterSystem::spawn_shelter(
                    &mut self.entities,
                    &mut self.next_entity_id,
                    shelter_type,
                    spawn_x,
                    spawn_y,
                    Some(ShelterCondition::Good),
                    Some("Generated Shelter".to_string()),
                );
            }
        }
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

    #[test]
    fn test_debug_message_toggle_state() {
        let mut game_state = GameState::new();

        // Initially debug messages should be enabled
        assert!(game_state.show_debug_messages);

        // Manually toggle the state to test the field
        game_state.show_debug_messages = false;
        assert!(!game_state.show_debug_messages);

        game_state.show_debug_messages = true;
        assert!(game_state.show_debug_messages);
    }

    #[test]
    fn test_message_toggle_integration() {
        let mut game_state = GameState::new();

        // Initially debug messages should be enabled
        assert!(game_state.show_debug_messages);
        let initial_message_count = game_state.debug_messages.len();

        // Simulate M key press by calling handle_ui_input with a mock that returns true for M
        // Since we can't easily mock the input handler, we'll test the toggle logic directly
        let initial_state = game_state.show_debug_messages;

        // Test the toggle logic by simulating what happens when M is pressed
        game_state.show_debug_messages = !game_state.show_debug_messages;
        let status = if game_state.show_debug_messages {
            "enabled"
        } else {
            "disabled"
        };
        game_state.add_debug_message(format!("Debug messages {}", status));

        // Verify the state changed and message was added
        assert_ne!(game_state.show_debug_messages, initial_state);
        assert_eq!(game_state.debug_messages.len(), initial_message_count + 1);
        assert!(game_state
            .debug_messages
            .last()
            .unwrap()
            .contains("Debug messages disabled"));

        // Toggle again
        let current_state = game_state.show_debug_messages;
        game_state.show_debug_messages = !game_state.show_debug_messages;
        let status = if game_state.show_debug_messages {
            "enabled"
        } else {
            "disabled"
        };
        game_state.add_debug_message(format!("Debug messages {}", status));

        // Verify it toggled back and added another message
        assert_ne!(game_state.show_debug_messages, current_state);
        assert_eq!(game_state.debug_messages.len(), initial_message_count + 2);
        assert!(game_state
            .debug_messages
            .last()
            .unwrap()
            .contains("Debug messages enabled"));
    }
}
