//! Environment components for atmospheric effects
//!
//! This module contains components for environmental elements like stars, moon,
//! ground tiles, and particle effects.

use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

/// Star component for background atmosphere
#[derive(Debug, Clone)]
pub struct Star {
    pub x: f32,
    pub y: f32,
    pub brightness: f32,
    pub twinkle_speed: f32,
    pub twinkle_offset: f32,
}

impl Star {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            brightness: rand::gen_range(0.3, 1.0),
            twinkle_speed: rand::gen_range(0.5, 2.0),
            twinkle_offset: rand::gen_range(0.0, 6.28),
        }
    }

    pub fn update(&mut self, time: f32) {
        // Create twinkling effect
        let twinkle = ((time * self.twinkle_speed + self.twinkle_offset).sin() + 1.0) * 0.5;
        self.brightness = 0.3 + twinkle * 0.7;
    }
}

/// Moon component for atmospheric night sky
#[derive(Debug, Clone)]
pub struct Moon {
    pub x: f32,
    pub y: f32,
    pub phase: f32, // 0.0 to 1.0 for waxing/waning
    pub glow_intensity: f32,
}

impl Moon {
    pub fn new() -> Self {
        Self {
            x: 1400.0, // Fixed position in world
            y: 100.0,
            phase: 0.8, // Nearly full moon
            glow_intensity: 0.9,
        }
    }

    pub fn update(&mut self, time: f32) {
        // Subtle glow pulsing
        self.glow_intensity = 0.7 + ((time * 0.3).sin() + 1.0) * 0.1;
    }
}

impl Default for Moon {
    fn default() -> Self {
        Self::new()
    }
}

/// Blood particle effect component
#[derive(Debug, Clone)]
pub struct BloodParticle {
    pub x: f32,
    pub y: f32,
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub life: f32,
    pub max_life: f32,
    pub size: f32,
}

impl BloodParticle {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            velocity_x: rand::gen_range(-30.0, 30.0),
            velocity_y: rand::gen_range(-50.0, -10.0),
            life: 100.0,
            max_life: 100.0,
            size: rand::gen_range(1.0, 3.0),
        }
    }

    pub fn update(&mut self, delta_time: f32) -> bool {
        println!(
            "BloodParticle update: life before = {}, delta_time = {}",
            self.life, delta_time
        );
        self.x += self.velocity_x * delta_time;
        self.y += self.velocity_y * delta_time;
        self.velocity_y += 98.0 * delta_time; // Gravity
        self.life -= delta_time * 0.8; // Fade over time (slower for longer effect)
        println!(
            "BloodParticle update: life after = {} (will keep? {})",
            self.life,
            self.life > 0.0
        );
        self.life > 0.0
    }

    pub fn draw(&self, camera_offset_x: f32, camera_offset_y: f32) {
        let zoom_level = 1.5;
        let screen_x = self.x * zoom_level + camera_offset_x;
        let screen_y = self.y * zoom_level + camera_offset_y;
        let alpha = self.life / self.max_life;

        // Make particles large and bright red for debugging visibility
        draw_circle(
            screen_x,
            screen_y,
            14.0,                           // Large size for visibility
            Color::new(1.0, 0.0, 0.0, 1.0), // Bright red, fully opaque
        );
    }
}

/// Ground tile types for terrain variety
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TileType {
    Grass,
    Dirt,
    Stone,
    DeadGrass,
}

/// Ground tile component for terrain system
#[derive(Debug, Clone)]
pub struct GroundTile {
    pub x: f32,
    pub y: f32,
    pub tile_type: TileType,
}

impl GroundTile {
    pub fn new(x: f32, y: f32, tile_type: TileType) -> Self {
        Self { x, y, tile_type }
    }
}
