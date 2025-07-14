//! Combat-related components
//!
//! This module contains components for combat mechanics, AI states, and battle statistics.

use serde::{Deserialize, Serialize};

/// Combat statistics component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatStats {
    pub attack_power: f32,
    pub defense: f32,
    pub last_attack_time: f32,
    pub attack_cooldown: f32,
}

impl CombatStats {
    pub fn new(attack_power: f32, defense: f32) -> Self {
        Self {
            attack_power,
            defense,
            last_attack_time: 0.0,
            attack_cooldown: 1.0,
        }
    }

    pub fn can_attack(&self, current_time: f32) -> bool {
        current_time - self.last_attack_time >= self.attack_cooldown
    }
}

impl Default for CombatStats {
    fn default() -> Self {
        Self {
            attack_power: 10.0,
            defense: 5.0,
            last_attack_time: 0.0,
            attack_cooldown: 1.0,
        }
    }
}

/// AI state for entity behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AIState {
    Idle,
    Hostile,
    Fleeing,
    Dead,
}

impl Default for AIState {
    fn default() -> Self {
        Self::Idle
    }
}

/// AI behavior types for different entity personalities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AIBehavior {
    Hostile,
    Neutral,
    Friendly,
    Fearful,
    Loyal,
}

impl Default for AIBehavior {
    fn default() -> Self {
        Self::Neutral
    }
}
