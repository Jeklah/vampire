//! Infinite World System for vertical streaming world generation
//!
//! This module manages dynamic world generation, entity spawning, and cleanup
//! for an infinite vertical forward-scrolling vampire RPG experience.
//! Player walks from bottom to top (forward into the distance).

use crate::components::{
    entities::GameEntity,
    environment::{GroundTile, TileType},
    game_data::{Clan, EntityType},
};
use crate::systems::WorldSystem;
use macroquad::prelude::*;
use std::collections::HashMap;

/// Size of each world chunk in game units (vertical)
pub const CHUNK_SIZE: f32 = 480.0;

/// Distance ahead of player to generate chunks (upward)
pub const GENERATION_DISTANCE: f32 = 1440.0; // 3 chunks ahead

/// Distance behind player to despawn chunks (downward)
pub const CLEANUP_DISTANCE: f32 = 960.0; // 2 chunks behind

/// Ground area height from bottom of screen
pub const GROUND_HEIGHT: f32 = 200.0;

/// Sky area height from top of screen
pub const SKY_HEIGHT: f32 = 150.0;

/// World chunk containing entities and terrain for a specific Y range
#[derive(Debug, Clone)]
pub struct WorldChunk {
    pub start_y: f32,
    pub end_y: f32,
    pub entities: Vec<u32>, // Entity IDs in this chunk
    pub ground_tiles: Vec<GroundTile>,
    pub generated: bool,
}

impl WorldChunk {
    pub fn new(start_y: f32) -> Self {
        Self {
            start_y,
            end_y: start_y - CHUNK_SIZE, // Moving up means decreasing Y
            entities: Vec::new(),
            ground_tiles: Vec::new(),
            generated: false,
        }
    }

    pub fn contains_y(&self, y: f32) -> bool {
        y <= self.start_y && y > self.end_y
    }
}

/// Manages infinite world generation and streaming for vertical movement
pub struct InfiniteWorldSystem {
    chunks: HashMap<i32, WorldChunk>,
    last_player_y: f32,
    next_entity_id: u32,
    last_shelter_spawn_y: f32,
    last_enemy_spawn_y: f32,
    ground_y_base: f32, // Base ground level that moves with camera
}

impl InfiniteWorldSystem {
    pub fn new(next_entity_id: u32) -> Self {
        Self {
            chunks: HashMap::new(),
            last_player_y: 600.0, // Start at bottom of screen
            next_entity_id,
            last_shelter_spawn_y: 800.0, // Start with higher Y to allow early spawning
            last_enemy_spawn_y: 700.0,
            ground_y_base: 720.0 - GROUND_HEIGHT, // Bottom of screen minus ground height
        }
    }

    /// Update the infinite world based on player position
    pub fn update(
        &mut self,
        player_y: f32,
        entities: &mut Vec<GameEntity>,
        clans: &HashMap<String, Clan>,
    ) {
        // Update ground base position
        self.ground_y_base = screen_height() - GROUND_HEIGHT;

        // Generate chunks ahead of player (upward - lower Y values)
        self.generate_chunks_ahead(player_y, entities, clans);

        // Clean up chunks behind player (downward - higher Y values)
        self.cleanup_chunks_behind(player_y, entities);

        self.last_player_y = player_y;
    }

    /// Generate chunks ahead of the player (upward direction)
    fn generate_chunks_ahead(
        &mut self,
        player_y: f32,
        entities: &mut Vec<GameEntity>,
        clans: &HashMap<String, Clan>,
    ) {
        let generation_end = player_y - GENERATION_DISTANCE; // Generate upward (lower Y)
        let start_chunk_id = ((player_y / CHUNK_SIZE).ceil() as i32).max(0);
        let end_chunk_id = (generation_end / CHUNK_SIZE).floor() as i32;

        for chunk_id in (end_chunk_id..=start_chunk_id).rev() {
            if !self.chunks.contains_key(&chunk_id) {
                let mut chunk = WorldChunk::new(chunk_id as f32 * CHUNK_SIZE);
                self.generate_chunk_content(&mut chunk, entities, clans);
                self.chunks.insert(chunk_id, chunk);
            }
        }
    }

    /// Generate content for a specific chunk
    fn generate_chunk_content(
        &mut self,
        chunk: &mut WorldChunk,
        entities: &mut Vec<GameEntity>,
        _clans: &HashMap<String, Clan>,
    ) {
        if chunk.generated {
            return;
        }

        // Generate ground tiles that span the full screen width
        self.generate_ground_tiles(chunk);

        // Spawn entities in this chunk
        self.spawn_chunk_entities(chunk, entities);

        chunk.generated = true;
    }

    /// Generate ground tiles that span the full screen width
    fn generate_ground_tiles(&self, chunk: &WorldChunk) {
        let tile_size = 64.0;
        let screen_width = screen_width();

        // Calculate how many tiles we need horizontally to cover screen width
        let tiles_across = (screen_width / tile_size).ceil() as i32 + 2; // +2 for buffer

        // Generate ground tiles for the full screen width
        for tile_x in 0..tiles_across {
            let x = tile_x as f32 * tile_size;

            // Generate tiles within this chunk's Y range
            let start_tile_y = (chunk.end_y / tile_size).floor() as i32;
            let end_tile_y = (chunk.start_y / tile_size).ceil() as i32;

            for tile_y in start_tile_y..end_tile_y {
                let y = tile_y as f32 * tile_size;

                // Only generate ground tiles in the bottom portion of the screen area
                if y >= self.ground_y_base {
                    let tile_type = self.determine_tile_type(x, y);
                    chunk.ground_tiles.push(GroundTile::new(x, y, tile_type));
                }
            }
        }
    }

    /// Determine tile type based on position and some variation
    fn determine_tile_type(&self, x: f32, y: f32) -> TileType {
        // Use position-based seeded randomization for consistent terrain
        let seed = ((y / 64.0) as i32 + (x / 64.0) as i32).abs() as u64;

        // Simple terrain variation based on distance traveled
        match seed % 10 {
            0..=5 => TileType::Grass,
            6..=7 => TileType::Dirt,
            8 => TileType::DeadGrass,
            _ => TileType::Stone,
        }
    }

    /// Spawn entities within a chunk
    fn spawn_chunk_entities(&mut self, chunk: &mut WorldChunk, entities: &mut Vec<GameEntity>) {
        let chunk_center_y = chunk.start_y - (CHUNK_SIZE / 2.0);

        // Spawn shelters periodically (check Y distance)
        if self.last_shelter_spawn_y - chunk_center_y >= 300.0 {
            if let Some(shelter_id) = self.spawn_shelter(chunk_center_y, entities) {
                chunk.entities.push(shelter_id);
                self.last_shelter_spawn_y = chunk_center_y;
            }
        }

        // Spawn enemies more frequently
        if self.last_enemy_spawn_y - chunk_center_y >= 150.0 {
            let enemy_count = rand::gen_range(1, 4);
            for _ in 0..enemy_count {
                if let Some(enemy_id) = self.spawn_enemy(chunk, entities) {
                    chunk.entities.push(enemy_id);
                }
            }
            self.last_enemy_spawn_y = chunk_center_y;
        }

        // Spawn animals (blood sources)
        let animal_count = rand::gen_range(1, 3);
        for _ in 0..animal_count {
            if let Some(animal_id) = self.spawn_animal(chunk, entities) {
                chunk.entities.push(animal_id);
            }
        }
    }

    /// Spawn a shelter at a specific Y position
    fn spawn_shelter(&mut self, y: f32, entities: &mut Vec<GameEntity>) -> Option<u32> {
        let x = rand::gen_range(100.0, screen_width() - 100.0);

        let entity_id = self.next_entity_id;
        self.next_entity_id += 1;

        if let Some(shelter) =
            WorldSystem::spawn_entity_at_position(entity_id, EntityType::Shelter, x, y)
        {
            entities.push(shelter);
            Some(entity_id)
        } else {
            None
        }
    }

    /// Spawn an enemy within a chunk
    fn spawn_enemy(&mut self, chunk: &WorldChunk, entities: &mut Vec<GameEntity>) -> Option<u32> {
        let x = rand::gen_range(50.0, screen_width() - 50.0);
        let y = rand::gen_range(chunk.end_y, chunk.start_y);

        let entity_id = self.next_entity_id;
        self.next_entity_id += 1;

        if let Some(enemy) =
            WorldSystem::spawn_entity_at_position(entity_id, EntityType::HostileInfected, x, y)
        {
            entities.push(enemy);
            Some(entity_id)
        } else {
            None
        }
    }

    /// Spawn an animal within a chunk
    fn spawn_animal(&mut self, chunk: &WorldChunk, entities: &mut Vec<GameEntity>) -> Option<u32> {
        let x = rand::gen_range(50.0, screen_width() - 50.0);
        let y = rand::gen_range(chunk.end_y, chunk.start_y);

        let entity_id = self.next_entity_id;
        self.next_entity_id += 1;

        if let Some(animal) =
            WorldSystem::spawn_entity_at_position(entity_id, EntityType::Animal, x, y)
        {
            entities.push(animal);
            Some(entity_id)
        } else {
            None
        }
    }

    /// Clean up chunks that are too far behind the player (downward)
    fn cleanup_chunks_behind(&mut self, player_y: f32, entities: &mut Vec<GameEntity>) {
        let cleanup_threshold = player_y + CLEANUP_DISTANCE;
        let mut chunks_to_remove = Vec::new();

        for (&chunk_id, chunk) in &self.chunks {
            if chunk.start_y > cleanup_threshold {
                // Remove entities from this chunk
                for &entity_id in &chunk.entities {
                    entities.retain(|e| e.id != entity_id);
                }
                chunks_to_remove.push(chunk_id);
            }
        }

        // Remove the chunks
        for chunk_id in chunks_to_remove {
            self.chunks.remove(&chunk_id);
        }
    }

    /// Get all ground tiles that should be rendered based on camera position
    pub fn get_visible_ground_tiles(&self, camera_y: f32, screen_height: f32) -> Vec<&GroundTile> {
        let mut visible_tiles = Vec::new();

        let view_start = camera_y - screen_height / 2.0;
        let view_end = camera_y + screen_height / 2.0;

        for chunk in self.chunks.values() {
            if chunk.start_y >= view_start && chunk.end_y <= view_end {
                for tile in &chunk.ground_tiles {
                    if tile.y >= view_start - 64.0 && tile.y <= view_end + 64.0 {
                        visible_tiles.push(tile);
                    }
                }
            }
        }

        visible_tiles
    }

    /// Update the next entity ID counter
    pub fn update_next_entity_id(&mut self, next_id: u32) {
        self.next_entity_id = next_id;
    }

    /// Get the current next entity ID
    pub fn get_next_entity_id(&self) -> u32 {
        self.next_entity_id
    }

    /// Get current ground base Y position
    pub fn get_ground_y_base(&self) -> f32 {
        self.ground_y_base
    }
}

impl Default for InfiniteWorldSystem {
    fn default() -> Self {
        Self::new(1000) // Start with ID 1000 to avoid conflicts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_creation() {
        let chunk = WorldChunk::new(480.0);
        assert_eq!(chunk.start_y, 480.0);
        assert_eq!(chunk.end_y, 0.0);
        assert!(!chunk.generated);
    }

    #[test]
    fn test_chunk_contains_y() {
        let chunk = WorldChunk::new(480.0);
        assert!(chunk.contains_y(240.0));
        assert!(!chunk.contains_y(500.0));
        assert!(!chunk.contains_y(-50.0));
    }

    #[test]
    fn test_infinite_world_system_creation() {
        let system = InfiniteWorldSystem::new(1000);
        assert_eq!(system.get_next_entity_id(), 1000);
        assert_eq!(system.chunks.len(), 0);
    }
}
