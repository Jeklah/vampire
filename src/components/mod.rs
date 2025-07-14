//! Core game components for the vampire RPG
//!
//! This module defines all the components used in the Entity Component System (ECS).
//! Components represent data that can be attached to entities.

use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Position component for entities in 2D space
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl Position {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn distance_to(&self, other: &Position) -> f32 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

/// Velocity component for moving entities
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

impl Velocity {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

/// Health component for entities that can take damage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Health {
    pub current: f32,
    pub maximum: f32,
    pub regeneration: f32,
}

impl Health {
    pub fn new(maximum: f32) -> Self {
        Self {
            current: maximum,
            maximum,
            regeneration: 0.0,
        }
    }

    pub fn is_alive(&self) -> bool {
        self.current > 0.0
    }

    pub fn take_damage(&mut self, damage: f32) {
        self.current = (self.current - damage).max(0.0);
    }

    pub fn heal(&mut self, amount: f32) {
        self.current = (self.current + amount).min(self.maximum);
    }

    pub fn health_percentage(&self) -> f32 {
        self.current / self.maximum
    }
}

/// Blood meter component - core vampire resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BloodMeter {
    pub current: f32,
    pub maximum: f32,
    pub drain_rate: f32,
    pub feeding_efficiency: f32,
}

impl BloodMeter {
    pub fn new(maximum: f32) -> Self {
        Self {
            current: maximum * 0.5, // Start at half blood
            maximum,
            drain_rate: 1.0,
            feeding_efficiency: 1.0,
        }
    }

    pub fn consume(&mut self, amount: f32) -> bool {
        if self.current >= amount {
            self.current -= amount;
            true
        } else {
            false
        }
    }

    pub fn add_blood(&mut self, amount: f32) {
        self.current = (self.current + amount * self.feeding_efficiency).min(self.maximum);
    }

    pub fn is_starving(&self) -> bool {
        self.current < self.maximum * 0.2
    }

    pub fn blood_percentage(&self) -> f32 {
        self.current / self.maximum
    }
}

/// Vampire abilities component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VampireAbilities {
    pub strength: f32,
    pub speed: f32,
    pub blood_sense: f32,
    pub shadow_movement: f32,
    pub mist_form: bool,
    pub wings: bool,
    pub shadow_command: f32,
}

impl Default for VampireAbilities {
    fn default() -> Self {
        Self {
            strength: 1.0,
            speed: 1.0,
            blood_sense: 0.0,
            shadow_movement: 0.0,
            mist_form: false,
            wings: false,
            shadow_command: 0.0,
        }
    }
}

/// Clan affiliation and properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clan {
    pub name: String,
    pub clan_type: crate::resources::ClanType,
    pub leader: Option<String>,
    pub members: Vec<String>,
    pub territory: Vec<Position>,
    pub trust: f32,
    pub fear: f32,
    pub strength: f32,
    pub special_traits: Vec<String>,
}

impl Clan {
    pub fn new(name: String, clan_type: crate::resources::ClanType) -> Self {
        Self {
            name,
            clan_type,
            leader: None,
            members: Vec::new(),
            territory: Vec::new(),
            trust: 0.0,
            fear: 0.0,
            strength: 1.0,
            special_traits: Vec::new(),
        }
    }

    pub fn loyalty_score(&self) -> f32 {
        (self.trust - self.fear).max(-1.0).min(1.0)
    }

    pub fn will_obey(&self) -> bool {
        self.loyalty_score() > 0.0 || self.fear > 0.7
    }
}

/// Territory control component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Territory {
    pub name: String,
    pub position: Position,
    pub radius: f32,
    pub controlled_by: Option<String>,
    pub resources: TerritoryResources,
    pub defense_level: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerritoryResources {
    pub blood_sources: f32,
    pub shelter_quality: f32,
    pub strategic_value: f32,
}

/// Combat statistics component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatStats {
    pub attack_power: f32,
    pub defense: f32,
    pub accuracy: f32,
    pub dodge_chance: f32,
    pub critical_chance: f32,
}

impl Default for CombatStats {
    fn default() -> Self {
        Self {
            attack_power: 10.0,
            defense: 5.0,
            accuracy: 0.8,
            dodge_chance: 0.1,
            critical_chance: 0.05,
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

/// Render component for visual representation
#[derive(Debug, Clone)]
pub struct Render {
    pub texture: Option<Texture2D>,
    pub color: Color,
    pub scale: f32,
    pub rotation: f32,
    pub visible: bool,
}

impl Default for Render {
    fn default() -> Self {
        Self {
            texture: None,
            color: WHITE,
            scale: 1.0,
            rotation: 0.0,
            visible: true,
        }
    }
}

/// Player component - marks the player entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub name: String,
    pub phase: crate::resources::Phase,
    pub hibernation_cycles: u32,
    pub reputation: f32,
    pub title: String,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            name: "The First Vampire".to_string(),
            phase: crate::resources::Phase::SurvivalAndDiscovery,
            hibernation_cycles: 0,
            reputation: 0.0,
            title: "The Sole Survivor".to_string(),
        }
    }
}

/// AI component for NPCs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AI {
    pub behavior: AIBehavior,
    pub target: Option<Position>,
    pub state: AIState,
    pub aggression: f32,
    pub awareness_radius: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AIBehavior {
    Hostile,
    Neutral,
    Friendly,
    Fearful,
    Loyal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AIState {
    Idle,
    Patrolling,
    Hunting,
    Fleeing,
    Following,
    Attacking,
}

impl Default for AI {
    fn default() -> Self {
        Self {
            behavior: AIBehavior::Neutral,
            target: None,
            state: AIState::Idle,
            aggression: 0.5,
            awareness_radius: 100.0,
        }
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

/// Sunlight vulnerability component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SunlightVulnerability {
    pub damage_rate: f32,
    pub movement_penalty: f32,
    pub ability_penalty: f32,
    pub in_sunlight: bool,
}

impl Default for SunlightVulnerability {
    fn default() -> Self {
        Self {
            damage_rate: 5.0,
            movement_penalty: 0.5,
            ability_penalty: 0.3,
            in_sunlight: false,
        }
    }
}
