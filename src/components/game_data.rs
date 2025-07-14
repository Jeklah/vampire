//! Game data components for clans, phases, and entity types
//!
//! This module contains components for game progression, clan management,
//! and entity classification.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Game phases representing different stages of the vampire story
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GamePhase {
    SurvivalAndDiscovery,
    ClanEncounters,
    EmpireBuilding,
    WorldReaction,
}

impl Default for GamePhase {
    fn default() -> Self {
        Self::SurvivalAndDiscovery
    }
}

/// Entity types for different character categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityType {
    Player,
    ClanLeader(String),
    ClanMember(String),
    HostileInfected,
    Animal,
}

/// Clan component for faction management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clan {
    pub name: String,
    pub leader_name: String,
    pub member_count: u32,
    pub trust_towards_player: f32,
    pub fear_of_player: f32,
    pub strength: f32,
    pub is_allied: bool,
    pub is_defeated: bool,
}

impl Clan {
    pub fn new(name: &str, leader_name: &str, member_count: u32) -> Self {
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

    pub fn loyalty_score(&self) -> f32 {
        (self.trust_towards_player - self.fear_of_player).clamp(-1.0, 1.0)
    }

    pub fn will_obey(&self) -> bool {
        self.loyalty_score() > 0.0 || self.fear_of_player > 0.7
    }
}

/// Player component - marks the player entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub name: String,
    pub phase: GamePhase,
    pub hibernation_cycles: u32,
    pub reputation: f32,
    pub title: String,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            name: "The First Vampire".to_string(),
            phase: GamePhase::SurvivalAndDiscovery,
            hibernation_cycles: 0,
            reputation: 0.0,
            title: "The Sole Survivor".to_string(),
        }
    }
}

/// Inventory component for items and resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inventory {
    pub items: HashMap<String, u32>,
    pub capacity: u32,
}

impl Inventory {
    pub fn new(capacity: u32) -> Self {
        Self {
            items: HashMap::new(),
            capacity,
        }
    }

    pub fn add_item(&mut self, item: String, quantity: u32) -> bool {
        let current_count = self.items.values().sum::<u32>();
        if current_count + quantity <= self.capacity {
            *self.items.entry(item).or_insert(0) += quantity;
            true
        } else {
            false
        }
    }

    pub fn remove_item(&mut self, item: &str, quantity: u32) -> bool {
        if let Some(count) = self.items.get_mut(item) {
            if *count >= quantity {
                *count -= quantity;
                if *count == 0 {
                    self.items.remove(item);
                }
                return true;
            }
        }
        false
    }

    pub fn has_item(&self, item: &str, quantity: u32) -> bool {
        self.items
            .get(item)
            .map_or(false, |&count| count >= quantity)
    }
}

/// Dialogue component for interactive NPCs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dialogue {
    pub current_node: String,
    pub available_topics: Vec<String>,
    pub relationship_modifiers: HashMap<String, f32>,
}

impl Default for Dialogue {
    fn default() -> Self {
        Self {
            current_node: "greeting".to_string(),
            available_topics: Vec::new(),
            relationship_modifiers: HashMap::new(),
        }
    }
}
