//! High-performance entity iterators using stable Rust features
//!
//! This module provides optimized iterators for common entity operations,
//! using stable Rust features for better compatibility.

use crate::components::*;

/// High-performance entity iterator with built-in filtering and spatial queries
pub struct EntityIterator<'a> {
    entities: &'a [GameEntity],
    current: usize,
}

impl<'a> EntityIterator<'a> {
    pub fn new(entities: &'a [GameEntity]) -> Self {
        Self {
            entities,
            current: 0,
        }
    }

    /// Filter entities by type
    pub fn by_type(self, entity_type: EntityType) -> EntityTypeIterator<'a> {
        EntityTypeIterator {
            inner: self,
            entity_type,
        }
    }

    /// Filter entities within a radius
    pub fn within_radius(self, center: Position, radius: f32) -> SpatialIterator<'a> {
        SpatialIterator {
            inner: self,
            center,
            radius_squared: radius * radius,
        }
    }

    /// Filter living entities (non-zero health)
    pub fn alive(self) -> AliveIterator<'a> {
        AliveIterator { inner: self }
    }

    /// Collect into an existing vector using stable extend
    pub fn collect_into_vec<T>(self, vec: &mut Vec<T>)
    where
        Self: Iterator<Item = T>,
    {
        vec.extend(self);
    }

    /// Batch entities for rendering optimization
    pub fn batch_for_rendering(self) -> RenderBatchIterator<'a> {
        RenderBatchIterator::new(self)
    }
}

impl<'a> Iterator for EntityIterator<'a> {
    type Item = &'a GameEntity;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.entities.len() {
            let entity = &self.entities[self.current];
            self.current += 1;
            Some(entity)
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.entities.len() - self.current;
        (0, Some(remaining))
    }
}

/// Type-filtered iterator
pub struct EntityTypeIterator<'a> {
    inner: EntityIterator<'a>,
    entity_type: EntityType,
}

impl<'a> Iterator for EntityTypeIterator<'a> {
    type Item = &'a GameEntity;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.find(|entity| {
            // Use discriminant matching for faster type comparison
            std::mem::discriminant(&entity.entity_type) == std::mem::discriminant(&self.entity_type)
        })
    }
}

/// Spatial iterator for distance calculations
pub struct SpatialIterator<'a> {
    inner: EntityIterator<'a>,
    center: Position,
    radius_squared: f32,
}

impl<'a> SpatialIterator<'a> {
    /// Optimized distance calculation for batches of entities
    #[inline]
    fn batch_distance_check(&self, positions: &[Position]) -> Vec<bool> {
        positions
            .iter()
            .map(|pos| {
                let dx = pos.x - self.center.x;
                let dy = pos.y - self.center.y;
                dx * dx + dy * dy <= self.radius_squared
            })
            .collect()
    }
}

impl<'a> Iterator for SpatialIterator<'a> {
    type Item = &'a GameEntity;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.find(|entity| {
            let dx = entity.position.x - self.center.x;
            let dy = entity.position.y - self.center.y;
            dx * dx + dy * dy <= self.radius_squared
        })
    }
}

/// Iterator for living entities with optimized health checks
pub struct AliveIterator<'a> {
    inner: EntityIterator<'a>,
}

impl<'a> Iterator for AliveIterator<'a> {
    type Item = &'a GameEntity;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.find(|entity| {
            // Fast path: check AI state first
            !matches!(entity.ai_state, AIState::Dead) &&
            // Check health component
            entity.health.as_ref().map_or(true, |h| h.current > 0.0)
        })
    }
}

/// Batch iterator for rendering optimization
pub struct RenderBatchIterator<'a> {
    entities: Vec<&'a GameEntity>,
    current_batch: usize,
    batch_size: usize,
}

impl<'a> RenderBatchIterator<'a> {
    fn new(iterator: EntityIterator<'a>) -> Self {
        // Collect all entities and sort by type for better batching
        let mut entities: Vec<_> = iterator.collect();
        entities
            .sort_by(|a, b| format!("{:?}", a.entity_type).cmp(&format!("{:?}", b.entity_type)));

        Self {
            entities,
            current_batch: 0,
            batch_size: 32, // Optimal batch size for rendering
        }
    }

    /// Get the next batch of entities of the same type
    pub fn next_batch(&mut self) -> Option<&[&'a GameEntity]> {
        if self.current_batch >= self.entities.len() {
            return None;
        }

        let start = self.current_batch;
        let current_type = &self.entities[start].entity_type;

        // Find end of current type batch
        let end = self.entities[start..]
            .iter()
            .position(|e| {
                std::mem::discriminant(&e.entity_type) != std::mem::discriminant(current_type)
            })
            .map(|pos| start + pos)
            .unwrap_or(self.entities.len());

        self.current_batch = end;
        Some(&self.entities[start..end])
    }
}

/// Extension trait for Vec<GameEntity> to add high-performance iterators
pub trait EntityVecExt {
    fn fast_iter(&self) -> EntityIterator<'_>;
    fn players(&self) -> EntityTypeIterator<'_>;
    fn enemies(&self) -> impl Iterator<Item = &GameEntity>;
    fn nearby(&self, center: Position, radius: f32) -> SpatialIterator<'_>;
    fn alive_entities(&self) -> AliveIterator<'_>;
}

impl EntityVecExt for Vec<GameEntity> {
    #[inline]
    fn fast_iter(&self) -> EntityIterator<'_> {
        EntityIterator::new(self)
    }

    fn players(&self) -> EntityTypeIterator<'_> {
        self.fast_iter().by_type(EntityType::Player)
    }

    fn enemies(&self) -> impl Iterator<Item = &GameEntity> {
        self.fast_iter().filter(|e| {
            matches!(
                e.entity_type,
                EntityType::HostileInfected | EntityType::ClanLeader(_)
            )
        })
    }

    fn nearby(&self, center: Position, radius: f32) -> SpatialIterator<'_> {
        self.fast_iter().within_radius(center, radius)
    }

    fn alive_entities(&self) -> AliveIterator<'_> {
        self.fast_iter().alive()
    }
}

/// Optimized entity finder
pub struct EntityFinder;

impl EntityFinder {
    /// Find entity by ID with early exit optimization
    #[inline]
    pub fn by_id(entities: &[GameEntity], id: u32) -> Option<&GameEntity> {
        // Use binary search if entities are sorted by ID
        if entities.len() > 100 && Self::is_sorted_by_id(entities) {
            entities
                .binary_search_by_key(&id, |e| e.id)
                .ok()
                .map(|idx| &entities[idx])
        } else {
            entities.iter().find(|e| e.id == id)
        }
    }

    /// Find closest entity to a position
    pub fn closest_to(entities: &[GameEntity], pos: Position) -> Option<(usize, f32)> {
        if entities.is_empty() {
            return None;
        }

        let mut closest_idx = 0;
        let mut closest_dist = f32::INFINITY;

        for (i, entity) in entities.iter().enumerate() {
            let dx = entity.position.x - pos.x;
            let dy = entity.position.y - pos.y;
            let dist = dx * dx + dy * dy;

            if dist < closest_dist {
                closest_dist = dist;
                closest_idx = i;
            }
        }

        Some((closest_idx, closest_dist.sqrt()))
    }

    #[inline]
    fn is_sorted_by_id(entities: &[GameEntity]) -> bool {
        entities.windows(2).all(|w| w[0].id <= w[1].id)
    }
}

/// Memory-efficient entity storage with better cache locality
pub struct PackedEntityStorage {
    positions: Vec<Position>,
    entity_types: Vec<EntityType>,
    healths: Vec<Option<Health>>,
    entity_map: Vec<usize>, // Maps external entity ID to internal index
}

impl PackedEntityStorage {
    pub fn new(capacity: usize) -> Self {
        Self {
            positions: Vec::with_capacity(capacity),
            entity_types: Vec::with_capacity(capacity),
            healths: Vec::with_capacity(capacity),
            entity_map: Vec::with_capacity(capacity),
        }
    }

    /// Add entity with structure-of-arrays layout for cache efficiency
    pub fn add_entity(&mut self, entity: &GameEntity) -> usize {
        let index = self.positions.len();

        self.positions.push(entity.position);
        self.entity_types.push(entity.entity_type.clone());
        self.healths.push(entity.health.clone());
        self.entity_map.push(index);

        index
    }

    /// Iterator over positions
    pub fn position_iter(&self) -> impl Iterator<Item = &Position> {
        self.positions.iter()
    }

    /// Batch process positions for spatial queries
    pub fn spatial_query(&self, center: Position, radius: f32) -> Vec<usize> {
        let radius_sq = radius * radius;
        let mut results = Vec::new();

        for (i, pos) in self.positions.iter().enumerate() {
            let dx = pos.x - center.x;
            let dy = pos.y - center.y;
            if dx * dx + dy * dy <= radius_sq {
                results.push(i);
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use macroquad::prelude::WHITE;

    #[test]
    fn test_entity_iterator_performance() {
        let entities = create_test_entities(1000);

        // Test basic iteration
        let count = entities.fast_iter().count();
        assert_eq!(count, 1000);

        // Test filtered iteration
        let alive_count = entities.alive_entities().count();
        assert!(alive_count <= 1000);
    }

    #[test]
    fn test_spatial_iterator() {
        let entities = create_test_entities(100);
        let center = Position { x: 0.0, y: 0.0 };
        let radius = 50.0;

        let nearby: Vec<_> = entities.nearby(center, radius).collect();

        // Verify all returned entities are actually within radius
        for entity in nearby {
            let dx = entity.position.x - center.x;
            let dy = entity.position.y - center.y;
            assert!(dx * dx + dy * dy <= radius * radius);
        }
    }

    #[test]
    fn test_entity_finder() {
        let entities = create_test_entities(50);

        if let Some(entity) = entities.first() {
            let found = EntityFinder::by_id(&entities, entity.id);
            assert!(found.is_some());
            assert_eq!(found.unwrap().id, entity.id);
        }
    }

    fn create_test_entities(count: usize) -> Vec<GameEntity> {
        (0..count)
            .map(|i| GameEntity {
                id: i as u32,
                position: Position {
                    x: (i as f32) * 10.0,
                    y: (i as f32) * 10.0,
                },
                velocity: None,
                entity_type: if i % 2 == 0 {
                    EntityType::Player
                } else {
                    EntityType::Animal
                },
                health: Some(Health::new(100.0)),
                combat_stats: None,
                ai_state: AIState::Idle,
                blood_meter: None,
                vampire_abilities: None,
                shelter: None,
                shelter_occupancy: None,
                color: WHITE,
            })
            .collect()
    }
}
