//! Vampire-specific components
//!
//! This module contains components related to vampire abilities and blood mechanics.

use serde::{Deserialize, Serialize};

/// Blood meter component - core vampire resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BloodMeter {
    pub current: f32,
    pub maximum: f32,
    pub drain_rate: f32,
}

impl BloodMeter {
    pub fn new(maximum: f32) -> Self {
        Self {
            current: maximum * 0.5, // Start at half blood
            maximum,
            drain_rate: 1.0,
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
        self.current = (self.current + amount).min(self.maximum);
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
}

impl Default for VampireAbilities {
    fn default() -> Self {
        Self {
            strength: 1.0,
            speed: 1.0,
            blood_sense: 0.0,
            shadow_movement: 0.0,
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
