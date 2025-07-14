//! Main game state and loop management
//!
//! This module contains the core game state, initialization, and main update loop.
//! It coordinates all systems and manages the overall game flow.

use crate::components::{
    AIBehavior, AIState, BloodMeter, Clan, CombatStats, Dialogue, Health, Inventory, Player,
    Position, Render, SunlightVulnerability, VampireAbilities, Velocity, AI,
};
use crate::resources::{
    CameraResource, ClanRegistry, ClanType as ResourcesClanType, GamePhase as GamePhaseResource,
    InputResource, Phase, TerritoryManager, TimeResource,
};
use crate::systems::*;
use hecs::{Entity, World};
use macroquad::prelude::*;
use std::collections::HashMap;

/// Main game state containing all game data
pub struct GameState {
    pub world: World,
    pub time: TimeResource,
    pub game_phase: GamePhaseResource,
    pub clan_registry: ClanRegistry,
    pub territory_manager: TerritoryManager,
    pub input: InputResource,
    pub camera: CameraResource,
    pub player_entity: Option<Entity>,
    pub paused: bool,
    pub current_menu: Option<GameMenu>,
}

#[derive(Debug, Clone)]
pub enum GameMenu {
    MainMenu,
    PauseMenu,
    ClanManagement,
    Inventory,
    PhaseTransition,
}

impl GameState {
    /// Initialize a new game state
    pub fn new() -> Self {
        let mut world = World::new();
        let mut game_state = Self {
            world,
            time: TimeResource::default(),
            game_phase: GamePhaseResource::default(),
            clan_registry: ClanRegistry::default(),
            territory_manager: TerritoryManager::default(),
            input: InputResource::default(),
            camera: CameraResource::default(),
            player_entity: None,
            paused: false,
            current_menu: None,
        };

        game_state.initialize_world();
        game_state
    }

    /// Initialize the game world with starting entities
    fn initialize_world(&mut self) {
        // Create the player entity
        let player_entity = self.world.spawn((
            Position::new(400.0, 300.0),
            Velocity::zero(),
            Health::new(100.0),
            BloodMeter::new(100.0),
            VampireAbilities::default(),
            CombatStats::default(),
            Inventory::new(20),
            Render {
                color: RED,
                scale: 1.5,
                ..Default::default()
            },
            Player::default(),
            SunlightVulnerability::default(),
        ));

        self.player_entity = Some(player_entity);

        // Set camera to follow player
        self.camera.set_target((400.0, 300.0));

        // Create some initial NPCs and enemies
        self.spawn_initial_npcs();
        self.spawn_initial_enemies();
    }

    /// Spawn initial NPCs for the world
    fn spawn_initial_npcs(&mut self) {
        // Spawn clan leaders
        self.spawn_clan_leader("Grimjaw", ResourcesClanType::BoneEaters, 200.0, 200.0);
        self.spawn_clan_leader("Shadowmere", ResourcesClanType::FlameHaters, 600.0, 400.0);
        self.spawn_clan_leader("Silentfang", ResourcesClanType::NightBloods, 800.0, 200.0);

        // Spawn some clan members
        for i in 0..5 {
            self.spawn_clan_member(
                150.0 + i as f32 * 30.0,
                180.0,
                ResourcesClanType::BoneEaters,
                AIBehavior::Neutral,
            );
        }

        for i in 0..4 {
            self.spawn_clan_member(
                550.0 + i as f32 * 40.0,
                380.0,
                ResourcesClanType::FlameHaters,
                AIBehavior::Neutral,
            );
        }

        for i in 0..3 {
            self.spawn_clan_member(
                750.0 + i as f32 * 50.0,
                180.0,
                ResourcesClanType::NightBloods,
                AIBehavior::Neutral,
            );
        }
    }

    /// Spawn initial hostile enemies
    fn spawn_initial_enemies(&mut self) {
        // Spawn some savage infected humans
        for i in 0..8 {
            let x = rand::gen_range(100.0, 1000.0);
            let y = rand::gen_range(100.0, 600.0);

            self.world.spawn((
                Position::new(x, y),
                Velocity::zero(),
                Health::new(50.0),
                CombatStats {
                    attack_power: 15.0,
                    defense: 2.0,
                    accuracy: 0.7,
                    dodge_chance: 0.15,
                    critical_chance: 0.1,
                },
                AI {
                    behavior: AIBehavior::Hostile,
                    state: AIState::Patrolling,
                    aggression: 0.8,
                    awareness_radius: 120.0,
                    ..Default::default()
                },
                Render {
                    color: Color::new(0.3, 0.0, 0.0, 1.0),
                    scale: 1.0,
                    ..Default::default()
                },
            ));
        }

        // Spawn some animals (potential blood sources)
        for i in 0..12 {
            let x = rand::gen_range(50.0, 1200.0);
            let y = rand::gen_range(50.0, 700.0);

            self.world.spawn((
                Position::new(x, y),
                Velocity::zero(),
                Health::new(25.0),
                AI {
                    behavior: AIBehavior::Fearful,
                    state: AIState::Idle,
                    aggression: 0.1,
                    awareness_radius: 80.0,
                    ..Default::default()
                },
                Render {
                    color: BROWN,
                    scale: 0.8,
                    ..Default::default()
                },
            ));
        }
    }

    /// Spawn a clan leader
    fn spawn_clan_leader(&mut self, name: &str, clan_type: ResourcesClanType, x: f32, y: f32) {
        let color = match clan_type {
            ResourcesClanType::BoneEaters => BEIGE,
            ResourcesClanType::FlameHaters => PURPLE,
            ResourcesClanType::NightBloods => DARKBLUE,
            ResourcesClanType::PlayerClan => GREEN,
        };

        self.world.spawn((
            Position::new(x, y),
            Velocity::zero(),
            Health::new(120.0),
            CombatStats {
                attack_power: 25.0,
                defense: 8.0,
                accuracy: 0.85,
                dodge_chance: 0.2,
                critical_chance: 0.15,
            },
            AI {
                behavior: AIBehavior::Neutral,
                state: AIState::Idle,
                aggression: 0.6,
                awareness_radius: 150.0,
                ..Default::default()
            },
            Clan::new(name.to_string(), clan_type),
            Dialogue::default(),
            Render {
                color,
                scale: 1.3,
                ..Default::default()
            },
        ));
    }

    /// Spawn a clan member
    fn spawn_clan_member(
        &mut self,
        x: f32,
        y: f32,
        clan_type: ResourcesClanType,
        behavior: AIBehavior,
    ) {
        let color = match clan_type {
            ResourcesClanType::BoneEaters => LIGHTGRAY,
            ResourcesClanType::FlameHaters => VIOLET,
            ResourcesClanType::NightBloods => BLUE,
            ResourcesClanType::PlayerClan => LIME,
        };

        self.world.spawn((
            Position::new(x, y),
            Velocity::zero(),
            Health::new(75.0),
            CombatStats {
                attack_power: 18.0,
                defense: 5.0,
                accuracy: 0.75,
                dodge_chance: 0.12,
                critical_chance: 0.08,
            },
            AI {
                behavior,
                state: AIState::Idle,
                aggression: 0.5,
                awareness_radius: 100.0,
                ..Default::default()
            },
            Render {
                color,
                scale: 1.0,
                ..Default::default()
            },
        ));
    }

    /// Main update loop for the game
    pub fn update(&mut self, delta_time: f32) {
        // Update input
        self.input.update();

        // Handle menu toggles
        if self.input.is_key_just_pressed(KeyCode::Escape) {
            self.toggle_pause_menu();
        }

        if self.input.is_key_just_pressed(KeyCode::Tab) {
            self.toggle_clan_menu();
        }

        if self.input.is_key_just_pressed(KeyCode::I) {
            self.toggle_inventory_menu();
        }

        // Don't update game systems if paused or in menu
        if self.paused || self.current_menu.is_some() {
            return;
        }

        // Update time system
        TimeSystem::update(&mut self.time, delta_time);
        TimeSystem::update_sunlight_exposure(&mut self.world, &self.time);

        // Handle player input and movement
        self.handle_player_input(delta_time);

        // Update all systems
        MovementSystem::update(&mut self.world, delta_time);
        AISystem::update(&mut self.world, delta_time);
        CombatSystem::update(&mut self.world, delta_time);
        BloodSystem::update(&mut self.world, &self.time, delta_time);
        PhaseSystem::update(&mut self.world, &mut self.game_phase, &self.clan_registry);

        // Update camera to follow player
        if let Some(player_entity) = self.player_entity {
            if let Ok(player_pos) = self.world.get::<Position>(player_entity) {
                self.camera.set_target((player_pos.x, player_pos.y));
            }
        }
        self.camera.update(delta_time);

        // Clean up dead entities
        self.cleanup_dead_entities();

        // Check for phase transitions
        if self.game_phase.can_advance {
            self.current_menu = Some(GameMenu::PhaseTransition);
        }
    }

    /// Handle player input for movement and actions
    fn handle_player_input(&mut self, delta_time: f32) {
        if let Some(player_entity) = self.player_entity {
            let mut movement = Vec2::ZERO;
            let base_speed = 200.0;

            // Movement input
            if self.input.is_key_down(KeyCode::W) {
                movement.y -= 1.0;
            }
            if self.input.is_key_down(KeyCode::S) {
                movement.y += 1.0;
            }
            if self.input.is_key_down(KeyCode::A) {
                movement.x -= 1.0;
            }
            if self.input.is_key_down(KeyCode::D) {
                movement.x += 1.0;
            }

            // Normalize diagonal movement
            if movement.length() > 0.0 {
                movement = movement.normalize();
            }

            // Apply speed modifications based on vampire abilities and sunlight
            let mut speed_modifier = 1.0;

            if let Ok(abilities) = self.world.get::<VampireAbilities>(player_entity) {
                speed_modifier *= abilities.speed;
            }

            if let Ok(vulnerability) = self.world.get::<SunlightVulnerability>(player_entity) {
                if vulnerability.in_sunlight {
                    speed_modifier *= 1.0 - vulnerability.movement_penalty;
                }
            }

            // Update player velocity
            if let Ok(mut velocity) = self.world.get_mut::<Velocity>(player_entity) {
                velocity.x = movement.x * base_speed * speed_modifier;
                velocity.y = movement.y * base_speed * speed_modifier;
            }

            // Action inputs
            if self.input.is_key_just_pressed(KeyCode::Space) {
                self.handle_player_action();
            }

            if self.input.is_key_just_pressed(KeyCode::E) {
                self.handle_player_interact();
            }

            if self.input.is_key_just_pressed(KeyCode::R) {
                self.handle_player_feed();
            }
        }
    }

    /// Handle player action (attack, ability use)
    fn handle_player_action(&mut self) {
        if let Some(player_entity) = self.player_entity {
            // Find nearby enemies to attack
            if let Ok(player_pos) = self.world.get::<Position>(player_entity) {
                let attack_range = 60.0;
                let mut targets = Vec::new();

                for (entity, (position, ai)) in self.world.query::<(&Position, &AI)>().iter() {
                    if matches!(ai.behavior, AIBehavior::Hostile) {
                        let distance = player_pos.distance_to(position);
                        if distance <= attack_range {
                            targets.push(entity);
                        }
                    }
                }

                // Attack the closest target
                if let Some(&target) = targets.first() {
                    self.attack_entity(player_entity, target);
                }
            }
        }
    }

    /// Handle player interaction with NPCs
    fn handle_player_interact(&mut self) {
        if let Some(player_entity) = self.player_entity {
            if let Ok(player_pos) = self.world.get::<Position>(player_entity) {
                let interact_range = 70.0;

                // Find nearby NPCs with dialogue
                for (entity, (position, dialogue)) in
                    self.world.query::<(&Position, &Dialogue)>().iter()
                {
                    let distance = player_pos.distance_to(position);
                    if distance <= interact_range {
                        // Start dialogue or clan interaction
                        self.start_dialogue(entity);
                        break;
                    }
                }
            }
        }
    }

    /// Handle player feeding action
    fn handle_player_feed(&mut self) {
        if let Some(player_entity) = self.player_entity {
            if let Ok(player_pos) = self.world.get::<Position>(player_entity) {
                let feed_range = 50.0;

                // Find nearby entities to feed on
                for (entity, (position, health)) in
                    self.world.query::<(&Position, &Health)>().iter()
                {
                    if entity == player_entity {
                        continue;
                    }

                    let distance = player_pos.distance_to(position);
                    if distance <= feed_range && health.current > 0.0 {
                        // Attempt to feed
                        if BloodSystem::feed_on_target(&mut self.world, player_entity, entity) {
                            println!("Fed on target!");
                        }
                        break;
                    }
                }
            }
        }
    }

    /// Attack an entity
    fn attack_entity(&mut self, attacker: Entity, target: Entity) {
        // This would trigger combat resolution
        // For now, just apply some damage
        if let Ok(mut health) = self.world.get_mut::<Health>(target) {
            health.take_damage(20.0);
        }
    }

    /// Start dialogue with an NPC
    fn start_dialogue(&mut self, npc_entity: Entity) {
        // Check if it's a clan leader
        if let Ok(clan) = self.world.get::<Clan>(npc_entity) {
            // Open clan management menu or start negotiation
            println!("Starting dialogue with {}", clan.name);
            self.current_menu = Some(GameMenu::ClanManagement);
        }
    }

    /// Toggle pause menu
    fn toggle_pause_menu(&mut self) {
        match self.current_menu {
            Some(GameMenu::PauseMenu) => {
                self.current_menu = None;
                self.paused = false;
            }
            None => {
                self.current_menu = Some(GameMenu::PauseMenu);
                self.paused = true;
            }
            _ => {
                self.current_menu = Some(GameMenu::PauseMenu);
                self.paused = true;
            }
        }
    }

    /// Toggle clan management menu
    fn toggle_clan_menu(&mut self) {
        match self.current_menu {
            Some(GameMenu::ClanManagement) => {
                self.current_menu = None;
            }
            _ => {
                self.current_menu = Some(GameMenu::ClanManagement);
            }
        }
    }

    /// Toggle inventory menu
    fn toggle_inventory_menu(&mut self) {
        match self.current_menu {
            Some(GameMenu::Inventory) => {
                self.current_menu = None;
            }
            _ => {
                self.current_menu = Some(GameMenu::Inventory);
            }
        }
    }

    /// Clean up dead entities
    fn cleanup_dead_entities(&mut self) {
        let mut entities_to_remove = Vec::new();

        for (entity, health) in self.world.query::<&Health>().iter() {
            if !health.is_alive() {
                entities_to_remove.push(entity);
            }
        }

        for entity in entities_to_remove {
            // Don't remove the player
            if Some(entity) != self.player_entity {
                let _ = self.world.despawn(entity);
            }
        }
    }

    /// Render the game
    pub fn render(&self) {
        clear_background(BLACK);

        // Draw game world
        RenderSystem::draw(&self.world, &self.camera);

        // Draw UI
        RenderSystem::draw_ui(&self.world, &self.time, &self.clan_registry);

        // Draw menus
        if let Some(ref menu) = self.current_menu {
            self.draw_menu(menu);
        }

        // Draw debug info
        if cfg!(debug_assertions) {
            self.draw_debug_info();
        }
    }

    /// Draw the current menu
    fn draw_menu(&self, menu: &GameMenu) {
        match menu {
            GameMenu::MainMenu => self.draw_main_menu(),
            GameMenu::PauseMenu => self.draw_pause_menu(),
            GameMenu::ClanManagement => self.draw_clan_menu(),
            GameMenu::Inventory => self.draw_inventory_menu(),
            GameMenu::PhaseTransition => self.draw_phase_transition_menu(),
        }
    }

    /// Draw main menu
    fn draw_main_menu(&self) {
        let screen_center = (screen_width() / 2.0, screen_height() / 2.0);

        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            Color::new(0.0, 0.0, 0.0, 0.8),
        );

        draw_text(
            "VAMPIRE RPG",
            screen_center.0 - 100.0,
            screen_center.1 - 100.0,
            48.0,
            RED,
        );

        draw_text(
            "Press ENTER to Start",
            screen_center.0 - 100.0,
            screen_center.1,
            24.0,
            WHITE,
        );
    }

    /// Draw pause menu
    fn draw_pause_menu(&self) {
        let screen_center = (screen_width() / 2.0, screen_height() / 2.0);

        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            Color::new(0.0, 0.0, 0.0, 0.7),
        );

        draw_text(
            "PAUSED",
            screen_center.0 - 50.0,
            screen_center.1 - 50.0,
            36.0,
            WHITE,
        );

        draw_text(
            "Press ESC to Resume",
            screen_center.0 - 80.0,
            screen_center.1,
            20.0,
            WHITE,
        );
    }

    /// Draw clan management menu
    fn draw_clan_menu(&self) {
        draw_rectangle(
            50.0,
            50.0,
            screen_width() - 100.0,
            screen_height() - 100.0,
            Color::new(0.1, 0.1, 0.2, 0.9),
        );

        draw_text("CLAN MANAGEMENT", 70.0, 80.0, 24.0, WHITE);

        let mut y = 120.0;
        for clan in self.clan_registry.clans.values() {
            let status_color = if clan.is_allied { GREEN } else { RED };

            draw_text(&clan.name, 70.0, y, 20.0, WHITE);
            draw_text(
                &format!("Members: {}", clan.member_count),
                200.0,
                y,
                16.0,
                GRAY,
            );
            draw_text(
                &format!("Strength: {:.1}", clan.strength),
                300.0,
                y,
                16.0,
                GRAY,
            );

            let status = if clan.is_allied {
                "Allied"
            } else if clan.is_defeated {
                "Defeated"
            } else {
                "Neutral"
            };
            draw_text(status, 400.0, y, 16.0, status_color);

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

    /// Draw inventory menu
    fn draw_inventory_menu(&self) {
        draw_rectangle(
            100.0,
            100.0,
            screen_width() - 200.0,
            screen_height() - 200.0,
            Color::new(0.2, 0.1, 0.1, 0.9),
        );

        draw_text("INVENTORY", 120.0, 130.0, 24.0, WHITE);

        if let Some(player_entity) = self.player_entity {
            if let Ok(inventory) = self.world.get::<Inventory>(player_entity) {
                let mut y = 170.0;

                if inventory.items.is_empty() {
                    draw_text("No items", 120.0, y, 18.0, GRAY);
                } else {
                    for (item, quantity) in &inventory.items {
                        draw_text(&format!("{}: {}", item, quantity), 120.0, y, 18.0, WHITE);
                        y += 25.0;
                    }
                }
            }
        }

        draw_text(
            "Press I to close",
            120.0,
            screen_height() - 120.0,
            16.0,
            LIGHTGRAY,
        );
    }

    /// Draw phase transition menu
    fn draw_phase_transition_menu(&self) {
        let screen_center = (screen_width() / 2.0, screen_height() / 2.0);

        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            Color::new(0.2, 0.0, 0.0, 0.9),
        );

        draw_text(
            "PHASE COMPLETE",
            screen_center.0 - 120.0,
            screen_center.1 - 100.0,
            36.0,
            GOLD,
        );

        draw_text(
            &format!("Current Phase: {:?}", self.game_phase.current_phase),
            screen_center.0 - 150.0,
            screen_center.1 - 50.0,
            20.0,
            WHITE,
        );

        draw_text(
            "You may now enter hibernation to advance to the next phase",
            screen_center.0 - 200.0,
            screen_center.1,
            18.0,
            WHITE,
        );

        draw_text(
            "Press ENTER to Continue",
            screen_center.0 - 100.0,
            screen_center.1 + 50.0,
            20.0,
            YELLOW,
        );
    }

    /// Draw debug information
    fn draw_debug_info(&self) {
        let y_start = screen_height() - 100.0;

        draw_text("DEBUG INFO", 10.0, y_start, 16.0, YELLOW);

        let entity_count = self.world.len();
        draw_text(
            &format!("Entities: {}", entity_count),
            10.0,
            y_start + 20.0,
            14.0,
            WHITE,
        );

        draw_text(
            &format!(
                "Camera: ({:.1}, {:.1})",
                self.camera.position.0, self.camera.position.1
            ),
            10.0,
            y_start + 40.0,
            14.0,
            WHITE,
        );

        draw_text(
            &format!("FPS: {:.0}", get_fps()),
            10.0,
            y_start + 60.0,
            14.0,
            WHITE,
        );
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}
