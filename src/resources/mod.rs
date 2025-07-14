//! Game-wide resources for the vampire RPG
//!
//! This module defines resources that are accessible globally throughout the game.
//! Resources represent singleton data that doesn't belong to specific entities.

use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Global time resource managing day/night cycles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeResource {
    pub current_time: f32, // 0.0 to 24.0 hours
    pub day_length: f32,   // Real-time seconds for a full day
    pub time_scale: f32,   // Speed multiplier for time
    pub day_count: u32,    // Total days passed
    pub is_day: bool,      // Quick check for daylight
    pub sunrise_hour: f32, // When day starts (default: 6.0)
    pub sunset_hour: f32,  // When night starts (default: 18.0)
}

impl Default for TimeResource {
    fn default() -> Self {
        Self {
            current_time: 0.0,
            day_length: 120.0, // 2 minutes per day
            time_scale: 1.0,
            day_count: 0,
            is_day: false,
            sunrise_hour: 6.0,
            sunset_hour: 18.0,
        }
    }
}

impl TimeResource {
    pub fn new(day_length: f32) -> Self {
        Self {
            day_length,
            ..Default::default()
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        self.current_time += (delta_time / self.day_length) * 24.0 * self.time_scale;

        if self.current_time >= 24.0 {
            self.current_time -= 24.0;
            self.day_count += 1;
        }

        self.is_day =
            self.current_time >= self.sunrise_hour && self.current_time < self.sunset_hour;
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

        // Peak sunlight at noon (12.0)
        let noon_distance = (self.current_time - 12.0).abs();
        let max_distance = 6.0; // From sunrise/sunset to noon

        if noon_distance > max_distance {
            0.0
        } else {
            1.0 - (noon_distance / max_distance)
        }
    }
}

/// Game phase management resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamePhase {
    pub current_phase: Phase,
    pub phase_progress: f32, // 0.0 to 1.0
    pub phase_objectives: Vec<String>,
    pub completed_objectives: Vec<String>,
    pub can_advance: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Phase {
    SurvivalAndDiscovery,
    ClanEncounters,
    EmpireBuilding,
    WorldReaction,
}

impl Default for GamePhase {
    fn default() -> Self {
        Self {
            current_phase: Phase::SurvivalAndDiscovery,
            phase_progress: 0.0,
            phase_objectives: vec![
                "Survive your first week".to_string(),
                "Discover your vampire abilities".to_string(),
                "Find shelter from sunlight".to_string(),
                "Feed on blood sources".to_string(),
            ],
            completed_objectives: Vec::new(),
            can_advance: false,
        }
    }
}

impl GamePhase {
    pub fn complete_objective(&mut self, objective: &str) {
        if let Some(pos) = self
            .phase_objectives
            .iter()
            .position(|obj| obj == objective)
        {
            let completed = self.phase_objectives.remove(pos);
            self.completed_objectives.push(completed);
            self.update_progress();
        }
    }

    pub fn update_progress(&mut self) {
        let total = self.phase_objectives.len() + self.completed_objectives.len();
        if total > 0 {
            self.phase_progress = self.completed_objectives.len() as f32 / total as f32;
            self.can_advance = self.phase_objectives.is_empty();
        }
    }

    pub fn advance_phase(&mut self) {
        if !self.can_advance {
            return;
        }

        self.current_phase = match self.current_phase {
            Phase::SurvivalAndDiscovery => {
                self.phase_objectives = vec![
                    "Locate the first clan".to_string(),
                    "Establish contact with clan leaders".to_string(),
                    "Gain control of at least 2 clans".to_string(),
                ];
                Phase::ClanEncounters
            }
            Phase::ClanEncounters => {
                self.phase_objectives = vec![
                    "Build a lair".to_string(),
                    "Establish blood economy".to_string(),
                    "Unite clans under your rule".to_string(),
                    "Control 3 territories".to_string(),
                ];
                Phase::EmpireBuilding
            }
            Phase::EmpireBuilding => {
                self.phase_objectives = vec![
                    "Deal with daylight hunters".to_string(),
                    "Uncover ancient vampire truths".to_string(),
                    "Choose your legacy path".to_string(),
                ];
                Phase::WorldReaction
            }
            Phase::WorldReaction => Phase::WorldReaction, // Final phase
        };

        self.phase_progress = 0.0;
        self.completed_objectives.clear();
        self.can_advance = false;
    }
}

/// Resource for managing all clans in the game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClanRegistry {
    pub clans: HashMap<String, ClanData>,
    pub player_clan: Option<String>,
    pub clan_relationships: HashMap<String, HashMap<String, f32>>, // clan -> clan -> relationship
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClanData {
    pub name: String,
    pub clan_type: ClanType,
    pub leader_name: String,
    pub member_count: u32,
    pub territory_count: u32,
    pub trust_towards_player: f32,
    pub fear_of_player: f32,
    pub strength: f32,
    pub special_traits: Vec<String>,
    pub is_defeated: bool,
    pub is_allied: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClanType {
    BoneEaters,
    FlameHaters,
    NightBloods,
    PlayerClan,
}

impl Default for ClanRegistry {
    fn default() -> Self {
        let mut registry = Self {
            clans: HashMap::new(),
            player_clan: None,
            clan_relationships: HashMap::new(),
        };

        // Initialize the three main clans
        registry.add_clan(ClanData {
            name: "Bone-Eaters".to_string(),
            clan_type: ClanType::BoneEaters,
            leader_name: "Grimjaw".to_string(),
            member_count: 15,
            territory_count: 2,
            trust_towards_player: 0.0,
            fear_of_player: 0.0,
            strength: 1.2,
            special_traits: vec!["Brute Force".to_string(), "Bone Weapons".to_string()],
            is_defeated: false,
            is_allied: false,
        });

        registry.add_clan(ClanData {
            name: "Flame-Haters".to_string(),
            clan_type: ClanType::FlameHaters,
            leader_name: "Shadowmere".to_string(),
            member_count: 12,
            territory_count: 1,
            trust_towards_player: 0.0,
            fear_of_player: 0.0,
            strength: 0.9,
            special_traits: vec!["Fire Resistance".to_string(), "Shadow Affinity".to_string()],
            is_defeated: false,
            is_allied: false,
        });

        registry.add_clan(ClanData {
            name: "Night-Bloods".to_string(),
            clan_type: ClanType::NightBloods,
            leader_name: "Silentfang".to_string(),
            member_count: 10,
            territory_count: 3,
            trust_towards_player: 0.0,
            fear_of_player: 0.0,
            strength: 1.1,
            special_traits: vec!["Stealth Mastery".to_string(), "Blood Tracking".to_string()],
            is_defeated: false,
            is_allied: false,
        });

        registry
    }
}

impl ClanRegistry {
    pub fn add_clan(&mut self, clan: ClanData) {
        self.clan_relationships
            .insert(clan.name.clone(), HashMap::new());
        self.clans.insert(clan.name.clone(), clan);
    }

    pub fn get_clan(&self, name: &str) -> Option<&ClanData> {
        self.clans.get(name)
    }

    pub fn get_clan_mut(&mut self, name: &str) -> Option<&mut ClanData> {
        self.clans.get_mut(name)
    }

    pub fn modify_trust(&mut self, clan_name: &str, amount: f32) {
        if let Some(clan) = self.clans.get_mut(clan_name) {
            clan.trust_towards_player = (clan.trust_towards_player + amount).clamp(-1.0, 1.0);
        }
    }

    pub fn modify_fear(&mut self, clan_name: &str, amount: f32) {
        if let Some(clan) = self.clans.get_mut(clan_name) {
            clan.fear_of_player = (clan.fear_of_player + amount).clamp(0.0, 1.0);
        }
    }

    pub fn get_loyalty_score(&self, clan_name: &str) -> f32 {
        if let Some(clan) = self.clans.get(clan_name) {
            (clan.trust_towards_player - clan.fear_of_player).clamp(-1.0, 1.0)
        } else {
            0.0
        }
    }

    pub fn allied_clans(&self) -> Vec<&ClanData> {
        self.clans.values().filter(|clan| clan.is_allied).collect()
    }

    pub fn hostile_clans(&self) -> Vec<&ClanData> {
        self.clans
            .values()
            .filter(|clan| !clan.is_allied && !clan.is_defeated)
            .collect()
    }
}

/// Territory management resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerritoryManager {
    pub territories: HashMap<String, TerritoryData>,
    pub controlled_territories: Vec<String>,
    pub contested_territories: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerritoryData {
    pub name: String,
    pub position: (f32, f32),
    pub radius: f32,
    pub controller: Option<String>,
    pub blood_sources: f32,
    pub shelter_quality: f32,
    pub strategic_value: f32,
    pub defense_level: f32,
}

impl Default for TerritoryManager {
    fn default() -> Self {
        let mut manager = Self {
            territories: HashMap::new(),
            controlled_territories: Vec::new(),
            contested_territories: Vec::new(),
        };

        // Initialize some starting territories
        manager.add_territory(TerritoryData {
            name: "Ancient Cave".to_string(),
            position: (100.0, 100.0),
            radius: 50.0,
            controller: None,
            blood_sources: 0.8,
            shelter_quality: 0.9,
            strategic_value: 0.7,
            defense_level: 0.6,
        });

        manager.add_territory(TerritoryData {
            name: "Bone Pit".to_string(),
            position: (300.0, 200.0),
            radius: 75.0,
            controller: Some("Bone-Eaters".to_string()),
            blood_sources: 0.6,
            shelter_quality: 0.4,
            strategic_value: 0.8,
            defense_level: 0.9,
        });

        manager.add_territory(TerritoryData {
            name: "Shadow Grove".to_string(),
            position: (500.0, 150.0),
            radius: 60.0,
            controller: Some("Night-Bloods".to_string()),
            blood_sources: 0.9,
            shelter_quality: 0.7,
            strategic_value: 0.9,
            defense_level: 0.5,
        });

        manager
    }
}

impl TerritoryManager {
    pub fn add_territory(&mut self, territory: TerritoryData) {
        self.territories.insert(territory.name.clone(), territory);
    }

    pub fn claim_territory(&mut self, territory_name: &str, controller: &str) {
        if let Some(territory) = self.territories.get_mut(territory_name) {
            territory.controller = Some(controller.to_string());

            if controller == "Player" {
                self.controlled_territories.push(territory_name.to_string());
            }
        }
    }

    pub fn get_player_territories(&self) -> Vec<&TerritoryData> {
        self.territories
            .values()
            .filter(|t| t.controller.as_deref() == Some("Player"))
            .collect()
    }

    pub fn get_total_blood_income(&self) -> f32 {
        self.get_player_territories()
            .iter()
            .map(|t| t.blood_sources)
            .sum()
    }
}

/// Input handling resource
#[derive(Debug, Default)]
pub struct InputResource {
    pub mouse_position: (f32, f32),
    pub mouse_clicked: bool,
    pub keys_pressed: Vec<KeyCode>,
    pub keys_just_pressed: Vec<KeyCode>,
    pub keys_just_released: Vec<KeyCode>,
}

impl InputResource {
    pub fn update(&mut self) {
        self.mouse_position = mouse_position();
        self.mouse_clicked = is_mouse_button_pressed(MouseButton::Left);

        // Clear previous frame's input
        self.keys_just_pressed.clear();
        self.keys_just_released.clear();

        // Update keyboard state
        let all_keys = [
            KeyCode::W,
            KeyCode::A,
            KeyCode::S,
            KeyCode::D,
            KeyCode::Space,
            KeyCode::Escape,
            KeyCode::Enter,
            KeyCode::Tab,
            KeyCode::LeftShift,
            KeyCode::E,
            KeyCode::Q,
            KeyCode::R,
            KeyCode::T,
            KeyCode::Y,
            KeyCode::U,
            KeyCode::I,
            KeyCode::O,
            KeyCode::P,
        ];

        for &key in &all_keys {
            if is_key_pressed(key) {
                self.keys_just_pressed.push(key);
            }
            if is_key_released(key) {
                self.keys_just_released.push(key);
            }
        }

        self.keys_pressed = all_keys
            .into_iter()
            .filter(|&key| is_key_down(key))
            .collect();
    }

    pub fn is_key_just_pressed(&self, key: KeyCode) -> bool {
        self.keys_just_pressed.contains(&key)
    }

    pub fn is_key_down(&self, key: KeyCode) -> bool {
        self.keys_pressed.contains(&key)
    }
}

/// Camera resource for rendering
#[derive(Debug, Clone)]
pub struct CameraResource {
    pub position: (f32, f32),
    pub zoom: f32,
    pub target: Option<(f32, f32)>,
    pub smooth_follow: bool,
    pub follow_speed: f32,
}

impl Default for CameraResource {
    fn default() -> Self {
        Self {
            position: (0.0, 0.0),
            zoom: 1.0,
            target: None,
            smooth_follow: true,
            follow_speed: 5.0,
        }
    }
}

impl CameraResource {
    pub fn update(&mut self, delta_time: f32) {
        if let Some(target) = self.target {
            if self.smooth_follow {
                let dx = target.0 - self.position.0;
                let dy = target.1 - self.position.1;

                self.position.0 += dx * self.follow_speed * delta_time;
                self.position.1 += dy * self.follow_speed * delta_time;
            } else {
                self.position = target;
            }
        }
    }

    pub fn set_target(&mut self, target: (f32, f32)) {
        self.target = Some(target);
    }

    pub fn screen_to_world(&self, screen_pos: (f32, f32)) -> (f32, f32) {
        let screen_center = (screen_width() / 2.0, screen_height() / 2.0);
        (
            (screen_pos.0 - screen_center.0) / self.zoom + self.position.0,
            (screen_pos.1 - screen_center.1) / self.zoom + self.position.1,
        )
    }
}
