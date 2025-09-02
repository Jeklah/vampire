//! Real-time Ground Generation System
//!
//! This module provides a spatial-aware ground generation system that creates ground tiles
//! in real-time as the player explores new areas. Uses the spatial grid system for efficient
//! area detection and idiomatic Rust patterns for performance and maintainability.

use crate::components::environment::{GroundTile, TileType};
use crate::systems::spatial_grid::SpatialGrid;
use crate::systems::world::{WorldSystem, GROUND_LEVEL, TILE_SIZE};
use macroquad::prelude::*;
use std::collections::{HashMap, HashSet};

/// Request for ground generation in a specific area
#[derive(Debug, Clone, PartialEq)]
pub struct GroundGenerationRequest {
    pub center_x: f32,
    pub center_y: f32,
    pub radius: f32,
    pub priority: GenerationPriority,
    pub request_time: f32,
}

impl GroundGenerationRequest {
    pub fn new(
        center_x: f32,
        center_y: f32,
        radius: f32,
        priority: GenerationPriority,
        time: f32,
    ) -> Self {
        Self {
            center_x,
            center_y,
            radius,
            priority,
            request_time: time,
        }
    }

    /// Calculate the bounds of this generation request
    pub fn bounds(&self) -> (f32, f32, f32, f32) {
        (
            self.center_x - self.radius,
            self.center_y - self.radius,
            self.center_x + self.radius,
            self.center_y + self.radius,
        )
    }

    /// Check if this request overlaps with a given area
    pub fn overlaps_with(&self, x: f32, y: f32, width: f32, height: f32) -> bool {
        let (min_x, min_y, max_x, max_y) = self.bounds();
        !(max_x <= x || min_x >= x + width || max_y <= y || min_y >= y + height)
    }
}

/// Priority levels for ground generation requests
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GenerationPriority {
    Background = 0, // Low priority, for distant areas
    Normal = 1,     // Normal priority, for nearby exploration
    Immediate = 2,  // High priority, for areas player is entering
    Critical = 3,   // Critical priority, for areas player is standing on
}

/// Result of a ground generation operation
#[derive(Debug, Clone)]
pub struct GenerationResult {
    pub tiles_generated: Vec<GroundTile>,
    pub areas_covered: Vec<(f32, f32, f32, f32)>, // x, y, width, height
    pub generation_time: f32,
}

impl GenerationResult {
    pub fn empty(time: f32) -> Self {
        Self {
            tiles_generated: Vec::new(),
            areas_covered: Vec::new(),
            generation_time: time,
        }
    }

    pub fn new(tiles: Vec<GroundTile>, areas: Vec<(f32, f32, f32, f32)>, time: f32) -> Self {
        Self {
            tiles_generated: tiles,
            areas_covered: areas,
            generation_time: time,
        }
    }
}

/// Spatial-aware ground generation system
pub struct GroundGenerationSystem {
    /// Spatial grid for efficient area queries
    spatial_grid: SpatialGrid,
    /// Pending generation requests
    generation_queue: Vec<GroundGenerationRequest>,
    /// Areas currently being generated (to avoid duplicates)
    active_generations: HashSet<(i32, i32)>, // Grid cell coordinates
    /// Generation history to avoid regenerating same areas
    generated_areas: HashMap<(i32, i32), f32>, // Grid cell -> generation time
    /// Generated tiles cache for efficient lookup
    tile_cache: HashMap<(i32, i32), GroundTile>, // Grid position -> tile
    /// Performance settings
    max_tiles_per_frame: usize,
    max_generation_distance: f32,
    grid_cell_size: f32,
    /// Statistics
    stats: GenerationStats,
}

/// Statistics for monitoring ground generation performance
#[derive(Debug, Clone, Default)]
pub struct GenerationStats {
    pub total_tiles_generated: usize,
    pub total_requests_processed: usize,
    pub active_requests: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub average_generation_time: f32,
    pub tiles_per_second: f32,
}

impl GroundGenerationSystem {
    /// Create a new ground generation system
    pub fn new() -> Self {
        Self {
            spatial_grid: SpatialGrid::new(TILE_SIZE),
            generation_queue: Vec::new(),
            active_generations: HashSet::new(),
            generated_areas: HashMap::new(),
            tile_cache: HashMap::new(),
            max_tiles_per_frame: 20,
            max_generation_distance: 800.0,
            grid_cell_size: TILE_SIZE,
            stats: GenerationStats::default(),
        }
    }

    /// Request ground generation around a specific point
    pub fn request_generation(
        &mut self,
        center_x: f32,
        center_y: f32,
        radius: f32,
        priority: GenerationPriority,
        current_time: f32,
    ) {
        // Only generate below ground level
        if center_y < GROUND_LEVEL {
            return;
        }

        // Don't generate too far from reasonable areas
        if radius > self.max_generation_distance {
            return;
        }

        let request =
            GroundGenerationRequest::new(center_x, center_y, radius, priority, current_time);

        // Check if this area is already being generated or recently generated
        if !self.should_generate_area(&request, current_time) {
            return;
        }

        // Add to queue, maintaining priority order
        self.generation_queue.push(request);
        self.generation_queue.sort_by(|a, b| {
            b.priority.cmp(&a.priority).then_with(|| {
                a.request_time
                    .partial_cmp(&b.request_time)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        });

        self.stats.active_requests = self.generation_queue.len();
    }

    /// Process ground generation requests (call this each frame)
    pub fn update(&mut self, current_time: f32) -> GenerationResult {
        let start_time = get_time();
        let mut tiles_generated = Vec::new();
        let mut areas_covered = Vec::new();
        let mut processed_requests = 0;

        // Process high-priority requests first
        let mut tiles_this_frame = 0;
        let mut requests_to_remove = Vec::new();

        // Clone the requests to avoid borrowing issues
        let requests_to_process: Vec<_> = self.generation_queue.iter().cloned().collect();

        for (index, request) in requests_to_process.iter().enumerate() {
            if tiles_this_frame >= self.max_tiles_per_frame {
                break; // Don't exceed per-frame limit
            }

            // Skip expired requests
            if current_time - request.request_time > 10.0 {
                requests_to_remove.push(index);
                continue;
            }

            if let Some(result) = self.process_generation_request(request, current_time) {
                tiles_generated.extend(result.tiles_generated);
                areas_covered.extend(result.areas_covered);
                tiles_this_frame += tiles_generated.len();
                processed_requests += 1;
                requests_to_remove.push(index);
            }
        }

        // Remove processed requests (in reverse order to maintain indices)
        for &index in requests_to_remove.iter().rev() {
            if index < self.generation_queue.len() {
                self.generation_queue.remove(index);
            }
        }

        // Update statistics
        self.stats.total_requests_processed += processed_requests;
        self.stats.active_requests = self.generation_queue.len();
        self.stats.total_tiles_generated += tiles_generated.len();

        let generation_time = (get_time() - start_time) as f32;
        self.stats.average_generation_time =
            (self.stats.average_generation_time * 0.9) + (generation_time * 0.1);

        if generation_time > 0.0 {
            self.stats.tiles_per_second = tiles_generated.len() as f32 / generation_time;
        }

        GenerationResult::new(tiles_generated, areas_covered, current_time)
    }

    /// Check if we should generate in a given area
    fn should_generate_area(&self, request: &GroundGenerationRequest, current_time: f32) -> bool {
        let grid_x = (request.center_x / self.grid_cell_size).floor() as i32;
        let grid_y = (request.center_y / self.grid_cell_size).floor() as i32;

        // Don't generate if already active
        if self.active_generations.contains(&(grid_x, grid_y)) {
            return false;
        }

        // Don't regenerate recently generated areas unless it's critical
        if let Some(&generation_time) = self.generated_areas.get(&(grid_x, grid_y)) {
            let time_since_generation = current_time - generation_time;
            match request.priority {
                GenerationPriority::Critical => time_since_generation > 1.0,
                GenerationPriority::Immediate => time_since_generation > 5.0,
                _ => time_since_generation > 30.0,
            }
        } else {
            true
        }
    }

    /// Process a single generation request
    fn process_generation_request(
        &mut self,
        request: &GroundGenerationRequest,
        current_time: f32,
    ) -> Option<GenerationResult> {
        let grid_x = (request.center_x / self.grid_cell_size).floor() as i32;
        let grid_y = (request.center_y / self.grid_cell_size).floor() as i32;

        // Mark as active to prevent duplicates
        self.active_generations.insert((grid_x, grid_y));

        let result = self.generate_tiles_in_area(request, current_time);

        // Mark as completed
        self.active_generations.remove(&(grid_x, grid_y));
        self.generated_areas.insert((grid_x, grid_y), current_time);

        Some(result)
    }

    /// Generate tiles within a specific area
    fn generate_tiles_in_area(
        &mut self,
        request: &GroundGenerationRequest,
        current_time: f32,
    ) -> GenerationResult {
        let mut tiles = Vec::new();
        let mut areas = Vec::new();

        // Calculate generation area
        let (min_x, min_y, max_x, max_y) = request.bounds();

        // Ensure we're at or below ground level
        let min_y = min_y.max(GROUND_LEVEL);

        if min_y >= max_y {
            return GenerationResult::empty(current_time);
        }

        // Generate tiles in a grid pattern
        let tile_size = TILE_SIZE;
        let cols = ((max_x - min_x) / tile_size).ceil() as i32;
        let rows = ((max_y - min_y) / tile_size).ceil() as i32;

        for col in 0..cols {
            for row in 0..rows {
                let tile_x = WorldSystem::snap_to_grid(min_x + col as f32 * tile_size);
                let tile_y = WorldSystem::snap_to_grid(min_y + row as f32 * tile_size);

                // Check if we already have a tile at this position
                let grid_pos = (
                    (tile_x / self.grid_cell_size).floor() as i32,
                    (tile_y / self.grid_cell_size).floor() as i32,
                );

                if !self.tile_cache.contains_key(&grid_pos) {
                    // Generate new tile
                    let tile = self.create_tile(tile_x, tile_y, request.priority);
                    self.tile_cache.insert(grid_pos, tile.clone());
                    tiles.push(tile);
                    self.stats.cache_misses += 1;
                } else {
                    self.stats.cache_hits += 1;
                }
            }
        }

        // Record area coverage
        if !tiles.is_empty() {
            areas.push((min_x, min_y, max_x - min_x, max_y - min_y));
        }

        GenerationResult::new(tiles, areas, current_time)
    }

    /// Create a single ground tile
    fn create_tile(&self, x: f32, y: f32, priority: GenerationPriority) -> GroundTile {
        // Choose tile type based on priority and randomness
        let tile_type = match priority {
            GenerationPriority::Critical | GenerationPriority::Immediate => {
                // Higher quality tiles for immediate areas
                if rand::gen_range(0.0, 1.0) < 0.6 {
                    TileType::Grass
                } else if rand::gen_range(0.0, 1.0) < 0.7 {
                    TileType::Dirt
                } else {
                    TileType::Stone
                }
            }
            _ => {
                // Standard distribution for background generation
                if rand::gen_range(0.0, 1.0) < 0.5 {
                    TileType::Grass
                } else if rand::gen_range(0.0, 1.0) < 0.3 {
                    TileType::DeadGrass
                } else {
                    TileType::Dirt
                }
            }
        };

        GroundTile::new(x, y, tile_type)
    }

    /// Get a tile at a specific position (for caching efficiency)
    pub fn get_tile_at(&self, x: f32, y: f32) -> Option<&GroundTile> {
        let grid_pos = (
            (x / self.grid_cell_size).floor() as i32,
            (y / self.grid_cell_size).floor() as i32,
        );
        self.tile_cache.get(&grid_pos)
    }

    /// Check if there's a tile at the given position
    pub fn has_tile_at(&self, x: f32, y: f32) -> bool {
        self.get_tile_at(x, y).is_some()
    }

    /// Get all tiles within a specific area
    pub fn get_tiles_in_area(
        &self,
        min_x: f32,
        min_y: f32,
        max_x: f32,
        max_y: f32,
    ) -> Vec<GroundTile> {
        let mut tiles = Vec::new();

        let start_col = (min_x / self.grid_cell_size).floor() as i32;
        let end_col = (max_x / self.grid_cell_size).ceil() as i32;
        let start_row = (min_y / self.grid_cell_size).floor() as i32;
        let end_row = (max_y / self.grid_cell_size).ceil() as i32;

        for col in start_col..=end_col {
            for row in start_row..=end_row {
                if let Some(tile) = self.tile_cache.get(&(col, row)) {
                    tiles.push(tile.clone());
                }
            }
        }

        tiles
    }

    /// Request immediate ground generation for player's current position
    pub fn request_immediate_ground(&mut self, player_x: f32, player_y: f32, current_time: f32) {
        self.request_generation(
            player_x,
            player_y,
            TILE_SIZE * 2.0, // Small radius for immediate generation
            GenerationPriority::Critical,
            current_time,
        );
    }

    /// Request exploration-based ground generation
    pub fn request_exploration_ground(
        &mut self,
        center_x: f32,
        center_y: f32,
        radius: f32,
        current_time: f32,
    ) {
        self.request_generation(
            center_x,
            center_y,
            radius,
            GenerationPriority::Immediate,
            current_time,
        );
    }

    /// Request background ground generation for performance
    pub fn request_background_ground(
        &mut self,
        center_x: f32,
        center_y: f32,
        radius: f32,
        current_time: f32,
    ) {
        self.request_generation(
            center_x,
            center_y,
            radius,
            GenerationPriority::Background,
            current_time,
        );
    }

    /// Clean up old generation data to prevent memory growth
    pub fn cleanup_old_data(&mut self, current_time: f32, max_age: f32) {
        // Clean up old generated areas
        self.generated_areas
            .retain(|_, &mut generation_time| current_time - generation_time < max_age);

        // Clean up old cached tiles (keep recently accessed ones)
        // This is simplified - in a full implementation, we'd track access times
        if self.tile_cache.len() > 1000 {
            let tiles_to_remove = self.tile_cache.len() - 800;
            let keys_to_remove: Vec<_> = self
                .tile_cache
                .keys()
                .take(tiles_to_remove)
                .cloned()
                .collect();
            for key in keys_to_remove {
                self.tile_cache.remove(&key);
            }
        }

        // Clean up expired requests
        self.generation_queue
            .retain(|request| current_time - request.request_time < 10.0);
    }

    /// Get current system statistics
    pub fn get_stats(&self) -> &GenerationStats {
        &self.stats
    }

    /// Set performance mode
    pub fn set_performance_mode(&mut self, enabled: bool) {
        if enabled {
            self.max_tiles_per_frame = 10;
            self.max_generation_distance = 400.0;
        } else {
            self.max_tiles_per_frame = 20;
            self.max_generation_distance = 800.0;
        }
    }

    /// Clear all cached data (for level resets)
    pub fn clear(&mut self) {
        self.generation_queue.clear();
        self.active_generations.clear();
        self.generated_areas.clear();
        self.tile_cache.clear();
        self.stats = GenerationStats::default();
    }
}

impl Default for GroundGenerationSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ground_generation_system_creation() {
        let system = GroundGenerationSystem::new();
        assert_eq!(system.generation_queue.len(), 0);
        assert_eq!(system.tile_cache.len(), 0);
    }

    #[test]
    fn test_generation_request_creation() {
        let request =
            GroundGenerationRequest::new(100.0, 100.0, 50.0, GenerationPriority::Normal, 0.0);
        assert_eq!(request.center_x, 100.0);
        assert_eq!(request.center_y, 100.0);
        assert_eq!(request.radius, 50.0);
        assert_eq!(request.priority, GenerationPriority::Normal);
    }

    #[test]
    fn test_generation_request_bounds() {
        let request =
            GroundGenerationRequest::new(100.0, 100.0, 50.0, GenerationPriority::Normal, 0.0);
        let (min_x, min_y, max_x, max_y) = request.bounds();
        assert_eq!(min_x, 50.0);
        assert_eq!(min_y, 50.0);
        assert_eq!(max_x, 150.0);
        assert_eq!(max_y, 150.0);
    }

    #[test]
    fn test_request_priority_ordering() {
        let mut system = GroundGenerationSystem::new();

        system.request_generation(0.0, 700.0, 64.0, GenerationPriority::Background, 0.0);
        system.request_generation(0.0, 700.0, 64.0, GenerationPriority::Critical, 1.0);
        system.request_generation(0.0, 700.0, 64.0, GenerationPriority::Normal, 0.5);

        // Should be sorted by priority (highest first)
        assert_eq!(
            system.generation_queue[0].priority,
            GenerationPriority::Critical
        );
        assert_eq!(
            system.generation_queue[1].priority,
            GenerationPriority::Normal
        );
        assert_eq!(
            system.generation_queue[2].priority,
            GenerationPriority::Background
        );
    }

    #[test]
    fn test_tile_cache_functionality() {
        let mut system = GroundGenerationSystem::new();

        // Test cache functionality without calling update() to avoid macroquad issues
        // Directly test the has_tile_at and get_tile_at methods
        assert!(!system.has_tile_at(0.0, 700.0));

        // Create and cache a tile manually
        let tile = system.create_tile(0.0, 700.0, GenerationPriority::Immediate);
        let grid_pos = (
            (0.0 / system.grid_cell_size).floor() as i32,
            (700.0 / system.grid_cell_size).floor() as i32,
        );
        system.tile_cache.insert(grid_pos, tile);

        // Now should find the tile
        assert!(system.has_tile_at(0.0, 700.0));
        assert!(system.get_tile_at(0.0, 700.0).is_some());
    }

    #[test]
    fn test_performance_mode() {
        let mut system = GroundGenerationSystem::new();
        let normal_max = system.max_tiles_per_frame;

        system.set_performance_mode(true);
        assert!(system.max_tiles_per_frame < normal_max);

        system.set_performance_mode(false);
        assert_eq!(system.max_tiles_per_frame, normal_max);
    }

    #[test]
    fn test_area_overlap_detection() {
        let request =
            GroundGenerationRequest::new(100.0, 100.0, 50.0, GenerationPriority::Normal, 0.0);

        // Should overlap
        assert!(request.overlaps_with(75.0, 75.0, 50.0, 50.0));

        // Should not overlap
        assert!(!request.overlaps_with(200.0, 200.0, 50.0, 50.0));
    }
}
