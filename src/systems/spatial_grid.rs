//! Spatial Grid System for Entity Culling and Optimization
//!
//! This module provides a spatial partitioning system to optimize entity queries
//! and reduce the computational complexity of distance-based operations.

use crate::components::*;
use std::collections::HashMap;

/// Grid-based spatial partitioning system
pub struct SpatialGrid {
    grid: HashMap<(i32, i32), Vec<u32>>,
    cell_size: f32,
    entity_positions: HashMap<u32, (i32, i32)>, // Track which cell each entity is in
}

impl SpatialGrid {
    /// Create a new spatial grid with the specified cell size
    pub fn new(cell_size: f32) -> Self {
        Self {
            grid: HashMap::new(),
            cell_size,
            entity_positions: HashMap::new(),
        }
    }

    /// Convert world position to grid cell coordinates
    #[inline]
    fn world_to_grid(&self, pos: Position) -> (i32, i32) {
        (
            (pos.x / self.cell_size).floor() as i32,
            (pos.y / self.cell_size).floor() as i32,
        )
    }

    /// Update the grid with current entity positions
    pub fn update(&mut self, entities: &[GameEntity]) {
        // Clear the grid
        self.grid.clear();

        // Re-populate the grid
        for entity in entities {
            let cell = self.world_to_grid(entity.position);

            // Update entity's cell position
            self.entity_positions.insert(entity.id, cell);

            // Add entity to the grid cell
            self.grid
                .entry(cell)
                .or_insert_with(Vec::new)
                .push(entity.id);
        }
    }

    /// Get entity IDs within a radius of the given position
    pub fn get_entities_in_radius(&self, center: Position, radius: f32) -> Vec<u32> {
        let center_cell = self.world_to_grid(center);
        let cell_radius = (radius / self.cell_size).ceil() as i32;

        let mut nearby_entities = Vec::new();

        // Check all cells within the radius
        for x in (center_cell.0 - cell_radius)..=(center_cell.0 + cell_radius) {
            for y in (center_cell.1 - cell_radius)..=(center_cell.1 + cell_radius) {
                if let Some(entity_ids) = self.grid.get(&(x, y)) {
                    nearby_entities.extend(entity_ids);
                }
            }
        }

        nearby_entities
    }

    /// Get entities in the same cell as the given position
    pub fn get_entities_in_cell(&self, pos: Position) -> Vec<u32> {
        let cell = self.world_to_grid(pos);
        self.grid.get(&cell).cloned().unwrap_or_default()
    }

    /// Get entities in adjacent cells (including the center cell)
    pub fn get_entities_in_adjacent_cells(&self, pos: Position) -> Vec<u32> {
        let center_cell = self.world_to_grid(pos);
        let mut nearby_entities = Vec::new();

        // Check 3x3 grid of cells
        for x in (center_cell.0 - 1)..=(center_cell.0 + 1) {
            for y in (center_cell.1 - 1)..=(center_cell.1 + 1) {
                if let Some(entity_ids) = self.grid.get(&(x, y)) {
                    nearby_entities.extend(entity_ids);
                }
            }
        }

        nearby_entities
    }

    /// Get the number of entities in each cell (for debugging)
    pub fn get_cell_statistics(&self) -> Vec<((i32, i32), usize)> {
        self.grid
            .iter()
            .map(|(cell, entities)| (*cell, entities.len()))
            .collect()
    }

    /// Check if an entity has moved to a different cell
    pub fn has_entity_moved_cells(&self, entity_id: u32, current_pos: Position) -> bool {
        let current_cell = self.world_to_grid(current_pos);

        if let Some(old_cell) = self.entity_positions.get(&entity_id) {
            *old_cell != current_cell
        } else {
            true // Entity is new
        }
    }

    /// Get all non-empty cells
    pub fn get_occupied_cells(&self) -> Vec<(i32, i32)> {
        self.grid.keys().cloned().collect()
    }

    /// Clear the grid
    pub fn clear(&mut self) {
        self.grid.clear();
        self.entity_positions.clear();
    }

    /// Get the cell size
    pub fn cell_size(&self) -> f32 {
        self.cell_size
    }

    /// Estimate memory usage of the grid
    pub fn estimated_memory_usage(&self) -> usize {
        let grid_size =
            self.grid.len() * (std::mem::size_of::<(i32, i32)>() + std::mem::size_of::<Vec<u32>>());
        let entity_positions_size = self.entity_positions.len()
            * (std::mem::size_of::<u32>() + std::mem::size_of::<(i32, i32)>());

        // Add estimated Vec contents
        let entities_in_cells: usize = self
            .grid
            .values()
            .map(|v| v.len() * std::mem::size_of::<u32>())
            .sum();

        grid_size + entity_positions_size + entities_in_cells
    }
}

/// Optimized spatial query helper
pub struct SpatialQuery<'a> {
    grid: &'a SpatialGrid,
    entities: &'a [GameEntity],
}

impl<'a> SpatialQuery<'a> {
    pub fn new(grid: &'a SpatialGrid, entities: &'a [GameEntity]) -> Self {
        Self { grid, entities }
    }

    /// Find closest entity to the given position within radius
    pub fn find_closest(
        &self,
        pos: Position,
        radius: f32,
        exclude_id: Option<u32>,
    ) -> Option<(u32, f32)> {
        let nearby_ids = self.grid.get_entities_in_radius(pos, radius);
        let mut closest_id = None;
        let mut closest_distance = f32::INFINITY;

        for &entity_id in &nearby_ids {
            if Some(entity_id) == exclude_id {
                continue;
            }

            if let Some(entity) = self.entities.iter().find(|e| e.id == entity_id) {
                let dx = entity.position.x - pos.x;
                let dy = entity.position.y - pos.y;
                let distance_sq = dx * dx + dy * dy;

                if distance_sq <= radius * radius
                    && distance_sq < closest_distance * closest_distance
                {
                    closest_distance = distance_sq.sqrt();
                    closest_id = Some(entity_id);
                }
            }
        }

        closest_id.map(|id| (id, closest_distance))
    }

    /// Get all entities of a specific type within radius
    pub fn find_by_type_in_radius(
        &self,
        pos: Position,
        radius: f32,
        entity_type: EntityType,
    ) -> Vec<&GameEntity> {
        let nearby_ids = self.grid.get_entities_in_radius(pos, radius);
        let mut matching_entities = Vec::new();

        for &entity_id in &nearby_ids {
            if let Some(entity) = self.entities.iter().find(|e| e.id == entity_id) {
                // Check if entity is within actual radius (not just grid approximation)
                let dx = entity.position.x - pos.x;
                let dy = entity.position.y - pos.y;
                let distance_sq = dx * dx + dy * dy;

                if distance_sq <= radius * radius
                    && std::mem::discriminant(&entity.entity_type)
                        == std::mem::discriminant(&entity_type)
                {
                    matching_entities.push(entity);
                }
            }
        }

        matching_entities
    }

    /// Count entities in radius (faster than collecting them)
    pub fn count_in_radius(&self, pos: Position, radius: f32) -> usize {
        let nearby_ids = self.grid.get_entities_in_radius(pos, radius);
        let radius_sq = radius * radius;

        nearby_ids
            .iter()
            .filter_map(|&id| self.entities.iter().find(|e| e.id == id))
            .filter(|entity| {
                let dx = entity.position.x - pos.x;
                let dy = entity.position.y - pos.y;
                dx * dx + dy * dy <= radius_sq
            })
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use macroquad::prelude::WHITE;

    fn create_test_entity(id: u32, x: f32, y: f32) -> GameEntity {
        GameEntity {
            id,
            position: Position { x, y },
            velocity: None,
            entity_type: EntityType::HostileInfected,
            health: Some(Health::new(100.0)),
            combat_stats: None,
            ai_state: AIState::Idle,
            blood_meter: None,
            vampire_abilities: None,
            shelter: None,
            shelter_occupancy: None,
            color: WHITE,
        }
    }

    #[test]
    fn test_spatial_grid_basic() {
        let mut grid = SpatialGrid::new(100.0);
        let entities = vec![
            create_test_entity(1, 50.0, 50.0),
            create_test_entity(2, 150.0, 150.0),
            create_test_entity(3, 250.0, 250.0),
        ];

        grid.update(&entities);

        let nearby = grid.get_entities_in_radius(Position { x: 50.0, y: 50.0 }, 50.0);
        assert!(nearby.contains(&1));
        assert!(!nearby.contains(&3));
    }

    #[test]
    fn test_spatial_query() {
        let mut grid = SpatialGrid::new(100.0);
        let entities = vec![
            create_test_entity(1, 0.0, 0.0),
            create_test_entity(2, 10.0, 10.0),
            create_test_entity(3, 200.0, 200.0),
        ];

        grid.update(&entities);
        let query = SpatialQuery::new(&grid, &entities);

        let closest = query.find_closest(Position { x: 5.0, y: 5.0 }, 50.0, None);
        assert!(closest.is_some());

        let (closest_id, _) = closest.unwrap();
        assert_eq!(closest_id, 1); // Entity 1 should be closest to (5, 5)
    }

    #[test]
    fn test_cell_movement_detection() {
        let mut grid = SpatialGrid::new(100.0);
        let entity = create_test_entity(1, 50.0, 50.0);

        grid.update(&[entity]);

        // Entity hasn't moved cells
        assert!(!grid.has_entity_moved_cells(1, Position { x: 60.0, y: 60.0 }));

        // Entity has moved to different cell
        assert!(grid.has_entity_moved_cells(1, Position { x: 150.0, y: 150.0 }));
    }

    #[test]
    fn test_grid_statistics() {
        let mut grid = SpatialGrid::new(100.0);
        let entities = vec![
            create_test_entity(1, 50.0, 50.0),
            create_test_entity(2, 60.0, 60.0), // Same cell as entity 1
            create_test_entity(3, 150.0, 150.0),
        ];

        grid.update(&entities);
        let stats = grid.get_cell_statistics();

        // Should have 2 cells: one with 2 entities, one with 1 entity
        assert_eq!(stats.len(), 2);

        let total_entities: usize = stats.iter().map(|(_, count)| count).sum();
        assert_eq!(total_entities, 3);
    }

    #[test]
    fn test_memory_usage_estimation() {
        let mut grid = SpatialGrid::new(100.0);
        let entities = vec![
            create_test_entity(1, 0.0, 0.0),
            create_test_entity(2, 100.0, 100.0),
        ];

        grid.update(&entities);
        let memory_usage = grid.estimated_memory_usage();

        assert!(memory_usage > 0);
    }
}
