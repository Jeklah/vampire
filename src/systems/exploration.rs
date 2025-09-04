//! Exploration System for Persistent Ground and Fog of War Management
//!
//! This module provides a system for managing explored areas, ground tiles, and fog of war
//! that persists across entity cleanup cycles. Ground tiles are treated as permanent
//! world features rather than temporary entities.

use crate::components::environment::{TileTextureData, TileType};
use crate::components::*;
use crate::systems::ground_generation::GroundGenerationSystem;
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
    /// Real-time ground generation system
    ground_generator: GroundGenerationSystem,
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
            ground_generator: GroundGenerationSystem::new(),
            max_ground_tiles: 1000,
            max_explored_regions: 100,
            grid_cell_size: FOG_TILE_SIZE,
        }
    }

    /// Add a ground tile to persistent storage
    pub fn add_ground_tile(&mut self, tile: GroundTile, current_time: f32) {
        // Check if we already have a tile at this location
        if !self.has_ground_tile_at(tile.x, tile.y) {
            // Add to persistent storage
            self.persistent_ground
                .push(PersistentGroundTile::new(tile.clone(), current_time));

            // Mark grid cell as explored - align with fog tile boundaries
            let grid_x = (tile.x / FOG_TILE_SIZE).floor() as i32;
            let grid_y = (tile.y / FOG_TILE_SIZE).floor() as i32;
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
        // Use grid-aligned detection to prevent gaps between tiles
        let grid_x = WorldSystem::snap_to_grid(x);
        let grid_y = WorldSystem::snap_to_grid(y);

        self.persistent_ground.iter().any(|persistent_tile| {
            let tile = &persistent_tile.tile;
            // Check if tiles are at the same grid position (exact match)
            (tile.x - grid_x).abs() < 0.1 && (tile.y - grid_y).abs() < 0.1
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

        // Mark grid cells as explored for faster lookup - align with fog tile boundaries
        let cells_x = (size / FOG_TILE_SIZE).ceil() as i32;
        let cells_y = (size / FOG_TILE_SIZE).ceil() as i32;

        for cell_x in 0..cells_x {
            for cell_y in 0..cells_y {
                let grid_x = ((x + cell_x as f32 * FOG_TILE_SIZE) / FOG_TILE_SIZE).floor() as i32;
                let grid_y = ((y + cell_y as f32 * FOG_TILE_SIZE) / FOG_TILE_SIZE).floor() as i32;
                self.explored_grid.insert((grid_x, grid_y));
            }
        }

        self.add_explored_region(x, y, size, size, time);
        self.invalidate_fog_cache();
    }

    /// Check if an area has been explored
    pub fn is_area_explored(&self, x: f32, y: f32, width: f32, height: f32) -> bool {
        // Check multiple grid cells for larger areas
        let cells_to_check_x = ((width / FOG_TILE_SIZE).ceil() as i32).max(1);
        let cells_to_check_y = ((height / FOG_TILE_SIZE).ceil() as i32).max(1);

        for cell_x in 0..cells_to_check_x {
            for cell_y in 0..cells_to_check_y {
                let grid_x = ((x + cell_x as f32 * FOG_TILE_SIZE) / FOG_TILE_SIZE).floor() as i32;
                let grid_y = ((y + cell_y as f32 * FOG_TILE_SIZE) / FOG_TILE_SIZE).floor() as i32;

                if self.explored_grid.contains(&(grid_x, grid_y)) {
                    return true;
                }
            }
        }

        // Check explored regions with more precise overlap detection
        for region in &self.explored_regions {
            // Check if any part of the test area overlaps with the explored region
            if !(x + width <= region.x
                || x >= region.x + region.width
                || y + height <= region.y
                || y >= region.y + region.height)
            {
                return true;
            }
        }

        // Check persistent ground tiles
        for persistent_tile in &self.persistent_ground {
            let tile = &persistent_tile.tile;
            if !(tile.x + TILE_SIZE <= x
                || tile.x >= x + width
                || tile.y + TILE_SIZE <= y
                || tile.y >= y + height)
            {
                return true;
            }
        }

        false
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
        // Check if this fog tile overlaps with any explored region
        for region in &self.explored_regions {
            // Check for overlap between fog tile and explored region
            if !(fog_x + FOG_TILE_SIZE <= region.x
                || fog_x >= region.x + region.width
                || fog_y + FOG_TILE_SIZE <= region.y
                || fog_y >= region.y + region.height)
            {
                return 0.0; // No fog in explored areas
            }
        }

        // Check if this fog tile overlaps with any ground tile
        for persistent_tile in &self.persistent_ground {
            let tile = &persistent_tile.tile;
            if !(fog_x + FOG_TILE_SIZE <= tile.x
                || fog_x >= tile.x + TILE_SIZE
                || fog_y + FOG_TILE_SIZE <= tile.y
                || fog_y >= tile.y + TILE_SIZE)
            {
                return 0.0; // No fog where ground exists
            }
        }

        // Find minimum distance to explored areas for fade effect
        let mut min_distance = f32::MAX;
        let search_radius = 250.0; // Increased for smoother transitions

        // Check distance to explored regions
        for region in &self.explored_regions {
            let region_center_x = region.x + region.width * 0.5;
            let region_center_y = region.y + region.height * 0.5;
            let fog_center_x = fog_x + FOG_TILE_SIZE * 0.5;
            let fog_center_y = fog_y + FOG_TILE_SIZE * 0.5;

            let dx = region_center_x - fog_center_x;
            let dy = region_center_y - fog_center_y;

            if dx.abs() <= search_radius && dy.abs() <= search_radius {
                let distance = (dx * dx + dy * dy).sqrt();
                min_distance = min_distance.min(distance);
            }
        }

        // Calculate alpha with smoother transition
        let fade_start_distance = 50.0; // Start fading closer to explored areas
        let fade_end_distance = 150.0; // Full fog at this distance

        if min_distance <= fade_start_distance {
            0.0 // No fog near explored areas
        } else if min_distance >= fade_end_distance {
            FOG_COLOR[3] // Full fog opacity
        } else {
            // Smooth fade between no fog and full fog
            let fade_factor =
                (min_distance - fade_start_distance) / (fade_end_distance - fade_start_distance);
            fade_factor * FOG_COLOR[3]
        }
    }

    /// Update exploration based on player position
    pub fn update_exploration(&mut self, player_x: f32, player_y: f32, current_time: f32) {
        // Check if player is entering unexplored areas
        let exploration_radius = 200.0;
        let upcoming_areas =
            self.get_upcoming_exploration_areas(player_x, player_y, exploration_radius);

        // Request ground generation for unexplored areas BEFORE marking them explored
        for (area_x, area_y) in upcoming_areas {
            if !self.is_area_explored(area_x, area_y, TILE_SIZE, TILE_SIZE) {
                // Request immediate ground generation for areas player is entering
                self.ground_generator.request_exploration_ground(
                    area_x,
                    area_y,
                    TILE_SIZE * 2.0,
                    current_time,
                );
            }
        }

        // Process ground generation requests
        let generation_result = self.ground_generator.update(current_time);

        // Add generated tiles to persistent storage
        for tile in generation_result.tiles_generated {
            self.add_ground_tile(tile, current_time);
        }

        // Now mark area around player as explored (fog will disappear)
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
        // Use a more responsive generation approach with larger visibility area
        let inner_radius = 512.0; // Close area - always filled (increased from 256)
        let outer_radius = 1024.0; // Extended area - filled as needed (increased from 512)

        // First pass: Ensure immediate area around player has complete coverage
        self.generate_ground_in_radius(player_x, player_y, inner_radius, current_time, true);

        // Second pass: Fill extended area with directional prediction
        self.generate_ground_in_radius(player_x, player_y, outer_radius, current_time, false);
    }

    /// Generate ground tiles within a specific radius
    fn generate_ground_in_radius(
        &mut self,
        center_x: f32,
        center_y: f32,
        radius: f32,
        current_time: f32,
        force_complete: bool,
    ) {
        // Calculate grid-aligned bounds
        let min_x = WorldSystem::snap_to_grid(center_x - radius);
        let max_x = WorldSystem::snap_to_grid(center_x + radius);
        let min_y = WorldSystem::snap_to_grid((center_y - radius).max(GROUND_LEVEL));
        let max_y = WorldSystem::snap_to_grid(center_y + radius);

        let mut tiles_to_add = Vec::new();
        let mut tiles_added_this_call = 0;
        let max_tiles_per_call = if force_complete { 400 } else { 128 };

        // Generate tiles in a precise grid pattern
        let mut x = min_x;
        while x <= max_x && tiles_added_this_call < max_tiles_per_call {
            let mut y = min_y;
            while y <= max_y && tiles_added_this_call < max_tiles_per_call {
                // Skip if we're at max capacity (unless force_complete for inner area)
                if !force_complete && self.persistent_ground.len() >= self.max_ground_tiles {
                    break;
                }

                // Check distance from center for circular generation
                let dx = x - center_x;
                let dy = y - center_y;
                let distance_sq = dx * dx + dy * dy;
                let radius_sq = radius * radius;

                if distance_sq <= radius_sq && !self.has_ground_tile_at(x, y) {
                    let tile_type = if rand::gen_range(0.0, 1.0) < 0.7 {
                        TileType::Grass
                    } else if rand::gen_range(0.0, 1.0) < 0.5 {
                        TileType::Dirt
                    } else {
                        TileType::Stone
                    };

                    tiles_to_add.push(GroundTile {
                        x,
                        y,
                        tile_type,
                        texture_data: TileTextureData {
                            grass_patches: Vec::new(),
                            dirt_spots: Vec::new(),
                            stone_blocks: Vec::new(),
                        },
                    });
                    tiles_added_this_call += 1;
                }
                y += TILE_SIZE;
            }
            x += TILE_SIZE;
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
            let grid_x = (tile.x / FOG_TILE_SIZE).floor() as i32;
            let grid_y = (tile.y / FOG_TILE_SIZE).floor() as i32;
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
            self.max_ground_tiles = 600;
            self.max_explored_regions = 50;
        } else {
            self.max_ground_tiles = 1000;
            self.max_explored_regions = 100;
        }
        self.ground_generator.set_performance_mode(enabled);
    }

    /// Clear all exploration data (for level resets)
    pub fn clear_all(&mut self) {
        self.persistent_ground.clear();
        self.explored_regions.clear();
        self.explored_grid.clear();
        self.ground_generator.clear();
        self.invalidate_fog_cache();
    }

    /// Get areas that the player is about to explore (for ground generation)
    fn get_upcoming_exploration_areas(
        &self,
        player_x: f32,
        player_y: f32,
        radius: f32,
    ) -> Vec<(f32, f32)> {
        let mut areas = Vec::new();
        let step = TILE_SIZE;

        // Create a grid of points around the player
        let min_x = player_x - radius;
        let max_x = player_x + radius;
        let min_y = (player_y - radius).max(GROUND_LEVEL);
        let max_y = player_y + radius;

        let mut x = min_x;
        while x <= max_x {
            let mut y = min_y;
            while y <= max_y {
                // Check if this point is within the exploration radius
                let dx = x - player_x;
                let dy = y - player_y;
                if (dx * dx + dy * dy) <= (radius * radius) {
                    areas.push((WorldSystem::snap_to_grid(x), WorldSystem::snap_to_grid(y)));
                }
                y += step;
            }
            x += step;
        }

        areas
    }

    /// Request immediate ground generation for player's current position
    pub fn request_immediate_ground(&mut self, player_x: f32, player_y: f32, current_time: f32) {
        self.ground_generator
            .request_immediate_ground(player_x, player_y, current_time);

        // Process the request immediately for critical areas
        let generation_result = self.ground_generator.update(current_time);
        for tile in generation_result.tiles_generated {
            self.add_ground_tile(tile, current_time);
        }
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
            x: 128.0, // Grid-aligned coordinate (64 * 2)
            y: 704.0, // Grid-aligned coordinate (64 * 11) and above ground level
            tile_type: TileType::Grass,
            texture_data: TileTextureData {
                grass_patches: Vec::new(),
                dirt_spots: Vec::new(),
                stone_blocks: Vec::new(),
            },
        };

        system.add_ground_tile(tile, 0.0);
        assert_eq!(system.persistent_ground.len(), 1);
        assert!(system.has_ground_tile_at(128.0, 704.0));
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

    #[test]
    fn test_ground_coverage_no_gaps() {
        let mut system = ExplorationSystem::new();

        // Generate ground around a test position
        let test_x = 512.0;
        let test_y = 700.0;
        system.ensure_ground_near_player(test_x, test_y, 0.0);

        // Check for gaps in a grid pattern around the player
        let check_radius = 192.0; // 3 tiles in each direction
        let tile_size = TILE_SIZE;

        let min_x = WorldSystem::snap_to_grid(test_x - check_radius);
        let max_x = WorldSystem::snap_to_grid(test_x + check_radius);
        let min_y = WorldSystem::snap_to_grid(test_y - check_radius);
        let max_y = WorldSystem::snap_to_grid(test_y + check_radius);

        let mut missing_tiles = Vec::new();

        // Check every grid position for tile coverage
        let mut x = min_x;
        while x <= max_x {
            let mut y = min_y;
            while y <= max_y {
                // Skip positions above ground level
                if y >= GROUND_LEVEL {
                    let dx = x - test_x;
                    let dy = y - test_y;
                    let distance = (dx * dx + dy * dy).sqrt();

                    // Within the inner radius should have complete coverage
                    if distance <= 256.0 && !system.has_ground_tile_at(x, y) {
                        missing_tiles.push((x, y));
                    }
                }
                y += tile_size;
            }
            x += tile_size;
        }

        // Assert no gaps exist in the inner coverage area
        if !missing_tiles.is_empty() {
            panic!(
                "Found {} missing tiles in coverage area: {:?}",
                missing_tiles.len(),
                missing_tiles.iter().take(5).collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn test_ground_tile_grid_alignment() {
        let mut system = ExplorationSystem::new();

        // Generate tiles at various positions
        system.ensure_ground_near_player(100.5, 650.3, 0.0);
        system.ensure_ground_near_player(200.7, 750.9, 0.0);

        // Verify all generated tiles are grid-aligned
        for persistent_tile in &system.persistent_ground {
            let tile = &persistent_tile.tile;

            // Check that coordinates are multiples of TILE_SIZE
            let x_aligned = (tile.x % TILE_SIZE).abs() < 0.1;
            let y_aligned = (tile.y % TILE_SIZE).abs() < 0.1;

            assert!(
                x_aligned && y_aligned,
                "Tile at ({}, {}) is not grid-aligned (TILE_SIZE = {})",
                tile.x,
                tile.y,
                TILE_SIZE
            );
        }
    }

    #[test]
    fn test_spatial_system_coordination() {
        let mut exploration_system = ExplorationSystem::new();

        // Test that fog tile boundaries align with exploration grid
        let test_x = 256.0; // 2 * FOG_TILE_SIZE
        let test_y = 768.0; // 6 * FOG_TILE_SIZE

        // Mark area as explored
        exploration_system.mark_area_explored(test_x, test_y, 200.0, 0.0);

        // Verify that fog tiles at this location are properly marked as explored
        assert!(exploration_system.is_area_explored(test_x, test_y, FOG_TILE_SIZE, FOG_TILE_SIZE));

        // Verify that ground tiles within this area would be considered explored
        let ground_tile_x = test_x + 32.0; // Within the explored area
        let ground_tile_y = test_y + 32.0;
        assert!(exploration_system.is_area_explored(
            ground_tile_x,
            ground_tile_y,
            TILE_SIZE,
            TILE_SIZE
        ));

        // Test fog alpha calculation coordination
        let fog_alpha_explored = exploration_system.calculate_fog_alpha(test_x, test_y, 0.0);
        let fog_alpha_unexplored =
            exploration_system.calculate_fog_alpha(test_x + 500.0, test_y, 0.0);

        assert!(
            fog_alpha_explored < 0.1,
            "Fog should be minimal in explored areas"
        );
        assert!(
            fog_alpha_unexplored > 0.5,
            "Fog should be significant in unexplored areas"
        );

        // Test that ground generation and exploration tracking coordinate properly
        let ground_tile = GroundTile {
            x: WorldSystem::snap_to_grid(test_x + 64.0),
            y: WorldSystem::snap_to_grid(test_y + 64.0),
            tile_type: TileType::Grass,
            texture_data: TileTextureData::default(),
        };

        exploration_system.add_ground_tile(ground_tile.clone(), 0.0);

        // Verify the ground tile area is marked as explored
        assert!(exploration_system.is_area_explored(
            ground_tile.x,
            ground_tile.y,
            TILE_SIZE,
            TILE_SIZE
        ));

        // Verify fog calculation accounts for the ground tile
        let fog_alpha_near_ground =
            exploration_system.calculate_fog_alpha(ground_tile.x, ground_tile.y, 0.0);
        assert!(
            fog_alpha_near_ground < 0.1,
            "Fog should be minimal near ground tiles"
        );
    }
}
