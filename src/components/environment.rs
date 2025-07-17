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
            velocity_x: rand::gen_range(-60.0, 60.0),
            velocity_y: rand::gen_range(-100.0, -20.0),
            life: 100.0,
            max_life: 100.0,
            size: rand::gen_range(1.0, 3.0),
        }
    }

    pub fn update(&mut self, delta_time: f32) -> bool {
        self.x += self.velocity_x * delta_time;
        self.y += self.velocity_y * delta_time;
        self.velocity_y += 98.0 * delta_time; // Gravity
        self.life -= delta_time * 0.8; // Fade over time (slower for longer effect)
        self.life > 0.0
    }

    pub fn draw(&self, camera_offset_x: f32, camera_offset_y: f32) {
        let zoom_level = 1.5;
        let screen_x = self.x * zoom_level + camera_offset_x;
        let screen_y = self.y * zoom_level + camera_offset_y;
        let _alpha = self.life / self.max_life;

        // Make particles large and bright red for debugging visibility
        draw_circle(
            screen_x,
            screen_y,
            7.0,                            // Half the original size
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
    pub texture_data: TileTextureData,
}

/// Pre-generated texture data for a ground tile to avoid per-frame random generation
#[derive(Debug, Clone)]
pub struct TileTextureData {
    pub grass_patches: Vec<(f32, f32, f32, f32)>, // x, y, width, height offsets
    pub dirt_spots: Vec<(f32, f32, f32)>,         // x, y, radius offsets
    pub stone_blocks: Vec<(f32, f32, f32, f32)>,  // x, y, width, height offsets
}

impl GroundTile {
    pub fn new(x: f32, y: f32, tile_type: TileType) -> Self {
        let texture_data = Self::generate_texture_data(&tile_type);
        Self {
            x,
            y,
            tile_type,
            texture_data,
        }
    }

    /// Pre-generate texture data for this tile type to avoid per-frame random generation
    fn generate_texture_data(tile_type: &TileType) -> TileTextureData {
        let mut grass_patches = Vec::new();
        let mut dirt_spots = Vec::new();
        let mut stone_blocks = Vec::new();

        match tile_type {
            TileType::Grass => {
                // Pre-generate grass patch positions
                for i in 0..8 {
                    for j in 0..4 {
                        let x_offset = (i as f32 * 8.0) + rand::gen_range(-4.0, 4.0);
                        let y_offset = (j as f32 * 16.0) + rand::gen_range(-4.0, 4.0);
                        grass_patches.push((x_offset, y_offset, 4.0, 8.0));
                    }
                }
            }
            TileType::DeadGrass => {
                // Pre-generate dead grass patch positions
                for i in 0..6 {
                    for j in 0..3 {
                        let x_offset = (i as f32 * 10.0) + rand::gen_range(-4.0, 4.0);
                        let y_offset = (j as f32 * 20.0) + rand::gen_range(-4.0, 4.0);
                        grass_patches.push((x_offset, y_offset, 4.0, 8.0));
                    }
                }
            }
            TileType::Dirt => {
                // Pre-generate dirt spot positions
                for _i in 0..12 {
                    let x_offset = rand::gen_range(0.0, 64.0);
                    let y_offset = rand::gen_range(0.0, 64.0);
                    let radius = rand::gen_range(2.0, 4.0);
                    dirt_spots.push((x_offset, y_offset, radius));
                }
            }
            TileType::Stone => {
                // Pre-generate stone block positions
                for i in 0..4 {
                    for j in 0..4 {
                        let x_offset = i as f32 * 16.0;
                        let y_offset = j as f32 * 16.0;
                        stone_blocks.push((x_offset, y_offset, 12.0, 12.0));
                    }
                }
            }
        }

        TileTextureData {
            grass_patches,
            dirt_spots,
            stone_blocks,
        }
    }
}

impl TileTextureData {
    /// Create empty texture data
    pub fn new() -> Self {
        Self {
            grass_patches: Vec::new(),
            dirt_spots: Vec::new(),
            stone_blocks: Vec::new(),
        }
    }
}

impl Default for TileTextureData {
    fn default() -> Self {
        Self::new()
    }
}
