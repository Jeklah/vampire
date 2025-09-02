//! Exploration System for Persistent Ground and Fog of War Management
//!
//! This module provides a system for managing explored areas, ground tiles, and fog of war
//! that persists across entity cleanup cycles. Ground tiles are treated as permanent
//! world features rather than temporary entities.

use crate::components::environment::{TileTextureData, TileType};
use crate::components::*;
use crate::systems::world::{WorldSystem, FOG_COLOR, FOG_TILE_SIZE, GROUND_LEVEL, TILE_SIZE};
use macroquad::prelude::*;
use std::collections::HashSet;

/// Represents a persistent explored area that should never have fog of war
#[derive(Debug, Clone, PartialEq)]
pub struct ExploredRegion {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub exploration_time: f32,
}

impl ExploredRegion {
    pub fn new(x: f32, y: f32, width: f32, height: f32, time: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
            exploration_time: time,
        }
    }

    pub fn contains_point(&self, px: f32, py: f32) -> bool {
        px >= self.x && px <= self.x + self.width && py >= self.y && py <= self.y + self.height
    }

    pub fn overlaps_with(&self, other_x: f32, other_y: f32, other_w: f32, other_h: f32) -> bool {
        !(self.x + self.width <= other_x
            || self.x >= other_x + other_w
            || self.y + self.height <= other_y
            || self.y >= other_y + other_h)
    }
}

/// Persistent ground tile that survives entity cleanup
#[derive(Debug, Clone)]
pub struct PersistentGroundTile {
    pub tile: GroundTile,
    pub created_time: f32,
    pub last_accessed: f32,
}

impl PersistentGroundTile {
    pub fn new(tile: GroundTile, time: f32) -> Self {
        Self {
            tile,
            created_time: time,
            last_accessed: time,
        }
    }

    pub fn access(&mut self, time: f32) {
        self.last_accessed = time;
    }
}

/// Exploration system managing persistent ground and fog of war
pub struct ExplorationSystem {
    /// Persistent ground tiles that survive cleanup
    persistent_ground: Vec<PersistentGroundTile>,
    /// Explored regions for fog of war management
    explored_regions: Vec<ExploredRegion>,
    /// Grid-based exploration tracking for performance
    explored_grid: HashSet<(i32, i32)>,
    /// Cached fog areas for performance
    fog_cache: Vec<(f32, f32, f32, f32, f32)>,
    /// Fog cache validity
    fog_cache_valid: bool,
    /// Last camera position for cache invalidation
    last_cache_camera_x: f32,
    last_cache_camera_y: f32,
    /// Performance settings
    max_ground_tiles: usize,
    max_explored_regions: usize,
    grid_cell_size: f32,
}

impl ExplorationSystem {
    /// Create a new exploration system
    pub fn new() -> Self {
        Self {
            persistent_ground: Vec::new(),
            explored_regions: Vec::new(),
            explored_grid: HashSet::new(),
            fog_cache: Vec::new(),
            fog_cache_valid: false,
            last_cache_camera_x: 0.0,
            last_cache_camera_y: 0.0,
            max_ground_tiles: 500,
            max_explored_regions: 100,
            grid_cell_size: TILE_SIZE,
        }
    }

    /// Add a ground tile to persistent storage
    pub fn add_ground_tile(&mut self, tile: GroundTile, current_time: f32) {
        // Check if we already have a tile at this location
        if !self.has_ground_tile_at(tile.x, tile.y) {
            // Add to persistent storage
            self.persistent_ground
                .push(PersistentGroundTile::new(tile.clone(), current_time));

            // Mark grid cell as explored
            let grid_x = (tile.x / self.grid_cell_size).floor() as i32;
            let grid_y = (tile.y / self.grid_cell_size).floor() as i32;
            self.explored_grid.insert((grid_x, grid_y));

            // Create explored region
            self.add_explored_region(tile.x, tile.y, TILE_SIZE, TILE_SIZE, current_time);

            // Invalidate fog cache
            self.invalidate_fog_cache();

            // Clean up if we have too many tiles
            if self.persistent_ground.len() > self.max_ground_tiles {
                self.cleanup_old_ground_tiles(current_time);
            }
        }
    }

    /// Add multiple ground tiles efficiently
    pub fn add_ground_tiles(&mut self, tiles: &[GroundTile], current_time: f32) {
        for tile in tiles {
            self.add_ground_tile(tile.clone(), current_time);
        }
    }

    /// Check if there's a ground tile at the specified location
    pub fn has_ground_tile_at(&self, x: f32, y: f32) -> bool {
        self.persistent_ground.iter().any(|persistent_tile| {
            let tile = &persistent_tile.tile;
            let dx = (tile.x - x).abs();
            let dy = (tile.y - y).abs();
            dx < TILE_SIZE && dy < TILE_SIZE
        })
    }

    /// Get all ground tiles as regular GroundTile structs
    pub fn get_ground_tiles(&self) -> Vec<GroundTile> {
        self.persistent_ground
            .iter()
            .map(|p| p.tile.clone())
            .collect()
    }

    /// Add an explored region
    fn add_explored_region(&mut self, x: f32, y: f32, width: f32, height: f32, time: f32) {
        // Check if this area is already covered by existing regions
        let new_region = ExploredRegion::new(x, y, width, height, time);

        let already_covered = self
            .explored_regions
            .iter()
            .any(|region| region.overlaps_with(x, y, width, height));

        if !already_covered {
            self.explored_regions.push(new_region);

            // Clean up if we have too many regions
            if self.explored_regions.len() > self.max_explored_regions {
                self.cleanup_old_explored_regions(time);
            }
        }
    }

    /// Mark an area as explored (usually called when player enters an area)
    pub fn mark_area_explored(&mut self, center_x: f32, center_y: f32, radius: f32, time: f32) {
        let size = radius * 2.0;
        let x = center_x - radius;
        let y = center_y - radius;

        self.add_explored_region(x, y, size, size, time);
        self.invalidate_fog_cache();
    }

    /// Check if an area has been explored
    pub fn is_area_explored(&self, x: f32, y: f32, width: f32, height: f32) -> bool {
        // Quick grid check first
        let grid_x = (x / self.grid_cell_size).floor() as i32;
        let grid_y = (y / self.grid_cell_size).floor() as i32;

        if self.explored_grid.contains(&(grid_x, grid_y)) {
            return true;
        }

        // Check explored regions
        self.explored_regions.iter().any(|region| {
            region.overlaps_with(x, y, width, height)
        }) ||
        // Check persistent ground tiles
        self.persistent_ground.iter().any(|persistent_tile| {
            let tile = &persistent_tile.tile;
            !(tile.x + TILE_SIZE <= x
                || tile.x >= x + width
                || tile.y + TILE_SIZE <= y
                || tile.y >= y + height)
        })
    }

    /// Generate fog areas that don't overlap with explored regions
    pub fn calculate_fog_areas(
        &mut self,
        camera_x: f32,
        camera_y: f32,
        screen_width: f32,
        screen_height: f32,
        zoom_level: f32,
        current_time: f32,
    ) -> &[(f32, f32, f32, f32, f32)] {
        // Check if cache needs updating
        let camera_moved = (camera_x - self.last_cache_camera_x).abs() > 200.0
            || (camera_y - self.last_cache_camera_y).abs() > 200.0;

        if !self.fog_cache_valid || camera_moved {
            self.rebuild_fog_cache(
                camera_x,
                camera_y,
                screen_width,
                screen_height,
                zoom_level,
                current_time,
            );
        }

        &self.fog_cache
    }

    /// Rebuild the fog cache
    fn rebuild_fog_cache(
        &mut self,
        camera_x: f32,
        camera_y: f32,
        screen_width: f32,
        screen_height: f32,
        zoom_level: f32,
        current_time: f32,
    ) {
        self.fog_cache.clear();

        // Calculate visible area with buffer
        let expansion = 256.0;
        let world_left = camera_x - (screen_width / (2.0 * zoom_level)) - expansion;
        let world_right = camera_x + (screen_width / (2.0 * zoom_level)) + expansion;
        let world_top = camera_y - (screen_height / (2.0 * zoom_level)) - expansion;
        let world_bottom = camera_y + (screen_height / (2.0 * zoom_level)) + expansion;

        // Ensure we don't go above the horizon line
        let world_top = world_top.max(GROUND_LEVEL);

        // Generate fog grid
        let fog_cols = ((world_right - world_left) / FOG_TILE_SIZE).ceil() as i32;
        let fog_rows = ((world_bottom - world_top) / FOG_TILE_SIZE).ceil() as i32;

        for col in 0..fog_cols {
            for row in 0..fog_rows {
                let fog_x = world_left + col as f32 * FOG_TILE_SIZE;
                let fog_y = world_top + row as f32 * FOG_TILE_SIZE;

                // Only add fog if area hasn't been explored
                if fog_y >= GROUND_LEVEL
                    && !self.is_area_explored(fog_x, fog_y, FOG_TILE_SIZE, FOG_TILE_SIZE)
                {
                    let alpha = self.calculate_fog_alpha(fog_x, fog_y, current_time);
                    self.fog_cache
                        .push((fog_x, fog_y, FOG_TILE_SIZE, FOG_TILE_SIZE, alpha));
                }
            }
        }

        self.fog_cache_valid = true;
        self.last_cache_camera_x = camera_x;
        self.last_cache_camera_y = camera_y;
    }

    /// Calculate fog alpha based on distance to explored areas
    fn calculate_fog_alpha(&self, fog_x: f32, fog_y: f32, _current_time: f32) -> f32 {
        let mut min_distance = f32::MAX;
        let search_radius = 200.0;

        // Check distance to explored regions
        for region in &self.explored_regions {
            let dx = (region.x + region.width * 0.5) - (fog_x + FOG_TILE_SIZE * 0.5);
            let dy = (region.y + region.height * 0.5) - (fog_y + FOG_TILE_SIZE * 0.5);

            if dx.abs() <= search_radius && dy.abs() <= search_radius {
                let distance = (dx * dx + dy * dy).sqrt();
                min_distance = min_distance.min(distance);
            }
        }

        // Check distance to persistent ground tiles
        for persistent_tile in &self.persistent_ground {
            let tile = &persistent_tile.tile;
            let dx = tile.x - fog_x;
            let dy = tile.y - fog_y;

            if dx.abs() <= search_radius && dy.abs() <= search_radius {
                let distance = (dx * dx + dy * dy).sqrt();
                min_distance = min_distance.min(distance);
            }
        }

        // Calculate alpha with smooth transition
        let fade_distance = 100.0;
        if min_distance >= fade_distance {
            FOG_COLOR[3]
        } else if min_distance <= FOG_TILE_SIZE {
            0.2 // Minimum alpha near explored areas
        } else {
            let fade_factor = (min_distance - FOG_TILE_SIZE) / (fade_distance - FOG_TILE_SIZE);
            0.2 + (FOG_COLOR[3] - 0.2) * fade_factor
        }
    }

    /// Update exploration based on player position
    pub fn update_exploration(&mut self, player_x: f32, player_y: f32, current_time: f32) {
        // Mark area around player as explored
        let exploration_radius = 128.0;
        self.mark_area_explored(player_x, player_y, exploration_radius, current_time);

        // Update last accessed time for nearby ground tiles
        for persistent_tile in &mut self.persistent_ground {
            let tile = &persistent_tile.tile;
            let dx = tile.x - player_x;
            let dy = tile.y - player_y;
            let distance_sq = dx * dx + dy * dy;

            if distance_sq < (exploration_radius * exploration_radius) {
                persistent_tile.access(current_time);
            }
        }
    }

    /// Generate ground tiles around player if needed
    pub fn ensure_ground_near_player(&mut self, player_x: f32, player_y: f32, current_time: f32) {
        let generation_radius = 400.0;
        let tile_spacing = TILE_SIZE;

        // Calculate area to check
        let min_x = player_x - generation_radius;
        let max_x = player_x + generation_radius;
        let min_y = (player_y - generation_radius).max(GROUND_LEVEL);
        let max_y = player_y + generation_radius;

        let mut tiles_to_add = Vec::new();

        // Generate tiles in a grid pattern
        let mut x = min_x;
        while x < max_x {
            let mut y = min_y;
            while y < max_y {
                // Check if we need a tile here
                if !self.has_ground_tile_at(x, y) {
                    // Only add if it's in a reasonable area and we're not at max capacity
                    if self.persistent_ground.len() < self.max_ground_tiles {
                        let tile_type = if rand::gen_range(0.0, 1.0) < 0.7 {
                            TileType::Grass
                        } else if rand::gen_range(0.0, 1.0) < 0.5 {
                            TileType::Dirt
                        } else {
                            TileType::Stone
                        };

                        tiles_to_add.push(GroundTile {
                            x: WorldSystem::snap_to_grid(x),
                            y: WorldSystem::snap_to_grid(y),
                            tile_type,
                            texture_data: TileTextureData {
                                grass_patches: Vec::new(),
                                dirt_spots: Vec::new(),
                                stone_blocks: Vec::new(),
                            },
                        });
                    }
                }
                y += tile_spacing;
            }
            x += tile_spacing;
        }

        // Add all generated tiles
        for tile in tiles_to_add {
            self.add_ground_tile(tile, current_time);
        }
    }

    /// Invalidate fog cache (should be called when ground changes)
    pub fn invalidate_fog_cache(&mut self) {
        self.fog_cache_valid = false;
    }

    /// Clean up old ground tiles to prevent memory growth
    fn cleanup_old_ground_tiles(&mut self, current_time: f32) {
        let max_age = 300.0; // 5 minutes
        let target_count = self.max_ground_tiles * 3 / 4;

        // Sort by last accessed time and keep the most recent ones
        self.persistent_ground
            .sort_by(|a, b| b.last_accessed.partial_cmp(&a.last_accessed).unwrap());

        // Remove old tiles beyond target count
        if self.persistent_ground.len() > target_count {
            self.persistent_ground.truncate(target_count);
        }

        // Remove very old tiles regardless of count
        self.persistent_ground
            .retain(|tile| current_time - tile.created_time < max_age);

        // Rebuild grid after cleanup
        self.rebuild_explored_grid();
        self.invalidate_fog_cache();
    }

    /// Clean up old explored regions
    fn cleanup_old_explored_regions(&mut self, current_time: f32) {
        let max_age = 600.0; // 10 minutes

        self.explored_regions
            .retain(|region| current_time - region.exploration_time < max_age);
    }

    /// Rebuild the explored grid from persistent ground tiles
    fn rebuild_explored_grid(&mut self) {
        self.explored_grid.clear();
        for persistent_tile in &self.persistent_ground {
            let tile = &persistent_tile.tile;
            let grid_x = (tile.x / self.grid_cell_size).floor() as i32;
            let grid_y = (tile.y / self.grid_cell_size).floor() as i32;
            self.explored_grid.insert((grid_x, grid_y));
        }
    }

    /// Get statistics for debugging
    pub fn get_stats(&self) -> ExplorationStats {
        ExplorationStats {
            persistent_ground_tiles: self.persistent_ground.len(),
            explored_regions: self.explored_regions.len(),
            explored_grid_cells: self.explored_grid.len(),
            fog_cache_areas: self.fog_cache.len(),
            fog_cache_valid: self.fog_cache_valid,
        }
    }

    /// Set performance mode
    pub fn set_performance_mode(&mut self, enabled: bool) {
        if enabled {
            self.max_ground_tiles = 300;
            self.max_explored_regions = 50;
        } else {
            self.max_ground_tiles = 500;
            self.max_explored_regions = 100;
        }
    }

    /// Clear all exploration data (for level resets)
    pub fn clear_all(&mut self) {
        self.persistent_ground.clear();
        self.explored_regions.clear();
        self.explored_grid.clear();
        self.invalidate_fog_cache();
    }
}

impl Default for ExplorationSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for monitoring exploration system performance
#[derive(Debug, Clone)]
pub struct ExplorationStats {
    pub persistent_ground_tiles: usize,
    pub explored_regions: usize,
    pub explored_grid_cells: usize,
    pub fog_cache_areas: usize,
    pub fog_cache_valid: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exploration_system_creation() {
        let system = ExplorationSystem::new();
        assert_eq!(system.persistent_ground.len(), 0);
        assert_eq!(system.explored_regions.len(), 0);
        assert!(!system.fog_cache_valid);
    }

    #[test]
    fn test_add_ground_tile() {
        let mut system = ExplorationSystem::new();
        let tile = GroundTile {
            x: 100.0,
            y: 100.0,
            tile_type: TileType::Grass,
            texture_data: TileTextureData {
                grass_patches: Vec::new(),
                dirt_spots: Vec::new(),
                stone_blocks: Vec::new(),
            },
        };

        system.add_ground_tile(tile, 0.0);
        assert_eq!(system.persistent_ground.len(), 1);
        assert!(system.has_ground_tile_at(100.0, 100.0));
        assert!(!system.fog_cache_valid);
    }

    #[test]
    fn test_explored_region_creation() {
        let region = ExploredRegion::new(0.0, 0.0, 100.0, 100.0, 0.0);
        assert!(region.contains_point(50.0, 50.0));
        assert!(!region.contains_point(150.0, 150.0));
    }

    #[test]
    fn test_area_exploration() {
        let mut system = ExplorationSystem::new();
        assert!(!system.is_area_explored(100.0, 100.0, 64.0, 64.0));

        system.mark_area_explored(132.0, 132.0, 50.0, 0.0);
        assert!(system.is_area_explored(100.0, 100.0, 64.0, 64.0));
    }

    #[test]
    fn test_performance_mode() {
        let mut system = ExplorationSystem::new();
        let normal_max = system.max_ground_tiles;

        system.set_performance_mode(true);
        assert!(system.max_ground_tiles < normal_max);

        system.set_performance_mode(false);
        assert_eq!(system.max_ground_tiles, normal_max);
    }

    #[test]
    fn test_ground_tile_generation() {
        let mut system = ExplorationSystem::new();
        let initial_count = system.persistent_ground.len();

        system.ensure_ground_near_player(0.0, 700.0, 0.0);
        assert!(system.persistent_ground.len() > initial_count);
    }
}
