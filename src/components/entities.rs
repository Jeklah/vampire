//! Core entity components for the vampire RPG
//!
//! This module contains basic entity components like position, velocity, health, etc.

use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

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
}

impl Health {
    pub fn new(maximum: f32) -> Self {
        Self {
            current: maximum,
            maximum,
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

/// Main game entity containing all components
#[derive(Debug, Clone)]
pub struct GameEntity {
    pub id: u32,
    pub position: Position,
    pub velocity: Velocity,
    pub health: Option<Health>,
    pub blood_meter: Option<super::vampire::BloodMeter>,
    pub abilities: Option<super::vampire::VampireAbilities>,
    pub combat_stats: Option<super::combat::CombatStats>,
    pub entity_type: super::game_data::EntityType,
    pub color: Color,
    pub ai_target: Option<u32>,
    pub ai_state: super::combat::AIState,
    pub facing_direction: f32,
}

/// Render component for visual representation
#[derive(Debug, Clone)]
pub struct Render {
    pub color: Color,
    pub scale: f32,
    pub rotation: f32,
    pub visible: bool,
}

impl Default for Render {
    fn default() -> Self {
        Self {
            color: WHITE,
            scale: 1.0,
            rotation: 0.0,
            visible: true,
        }
    }
}
