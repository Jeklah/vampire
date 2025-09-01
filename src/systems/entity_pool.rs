//! Entity Pool System for Object Reuse and Memory Optimization
//!
//! This module provides an object pool for GameEntity instances to reduce
//! allocations and improve performance by reusing entity objects.

use crate::components::*;
use std::collections::VecDeque;

/// Pool for reusing GameEntity objects
pub struct EntityPool {
    /// Available entities ready for reuse
    available: VecDeque<GameEntity>,
    /// Maximum pool size to prevent unbounded growth
    max_size: usize,
    /// Statistics for monitoring pool usage
    stats: PoolStats,
}

/// Statistics for monitoring pool performance
#[derive(Debug, Clone, Default)]
pub struct PoolStats {
    /// Total entities created (new allocations)
    pub total_created: usize,
    /// Total entities reused from pool
    pub total_reused: usize,
    /// Total entities returned to pool
    pub total_returned: usize,
    /// Peak pool size reached
    pub peak_size: usize,
    /// Current pool utilization ratio
    pub utilization_ratio: f32,
}

impl EntityPool {
    /// Create a new entity pool with the specified maximum size
    pub fn new(max_size: usize) -> Self {
        Self {
            available: VecDeque::with_capacity(max_size),
            max_size,
            stats: PoolStats::default(),
        }
    }

    /// Create a pool with reasonable defaults for the vampire game
    pub fn with_defaults() -> Self {
        Self::new(200) // Allow up to 200 pooled entities
    }

    /// Get an entity from the pool or create a new one
    pub fn acquire(&mut self) -> GameEntity {
        if let Some(mut entity) = self.available.pop_front() {
            // Reset entity to default state
            self.reset_entity(&mut entity);
            self.stats.total_reused += 1;
            entity
        } else {
            // Create new entity if pool is empty
            self.stats.total_created += 1;
            self.create_default_entity()
        }
    }

    /// Return an entity to the pool for reuse
    pub fn release(&mut self, entity: GameEntity) {
        if self.available.len() < self.max_size {
            self.available.push_back(entity);
            self.stats.total_returned += 1;

            // Update peak size tracking
            if self.available.len() > self.stats.peak_size {
                self.stats.peak_size = self.available.len();
            }
        }
        // If pool is full, just drop the entity (let it be garbage collected)

        self.update_utilization_ratio();
    }

    /// Spawn a specific entity type using the pool
    pub fn spawn_entity(
        &mut self,
        entity_type: EntityType,
        position: Position,
        next_entity_id: &mut u32,
    ) -> GameEntity {
        let mut entity = self.acquire();

        // Configure the entity
        entity.id = *next_entity_id;
        entity.position = position;
        entity.entity_type = entity_type.clone();

        // Set type-specific defaults
        match entity_type {
            EntityType::Player => {
                entity.health = Some(Health::new(100.0));
                entity.blood_meter = Some(BloodMeter::new(100.0));
                entity.vampire_abilities = Some(VampireAbilities {
                    strength: 1.0,
                    speed: 1.2,
                    blood_sense: 200.0,
                    shadow_movement: 0.0,
                });
                entity.combat_stats = Some(CombatStats::new(20.0, 10.0));
                entity.ai_state = AIState::Idle;
                entity.color = macroquad::prelude::BLUE;
            }
            EntityType::HostileInfected => {
                entity.health = Some(Health::new(30.0));
                entity.combat_stats = Some(CombatStats::new(15.0, 5.0));
                entity.ai_state = AIState::Hostile;
                entity.color = macroquad::prelude::RED;
                entity.velocity = Some(Velocity { x: 0.0, y: 0.0 });
            }
            EntityType::Animal => {
                entity.health = Some(Health::new(20.0));
                entity.combat_stats = Some(CombatStats::new(5.0, 2.0));
                entity.ai_state = AIState::Idle;
                entity.color = macroquad::prelude::BROWN;
                entity.velocity = Some(Velocity { x: 0.0, y: 0.0 });
            }
            EntityType::ClanLeader(_) => {
                entity.health = Some(Health::new(80.0));
                entity.combat_stats = Some(CombatStats::new(25.0, 15.0));
                entity.ai_state = AIState::Idle;
                entity.color = macroquad::prelude::PURPLE;
                entity.velocity = Some(Velocity { x: 0.0, y: 0.0 });
            }
            EntityType::ClanMember(_) => {
                entity.health = Some(Health::new(50.0));
                entity.combat_stats = Some(CombatStats::new(18.0, 8.0));
                entity.ai_state = AIState::Idle;
                entity.color = macroquad::prelude::DARKPURPLE;
                entity.velocity = Some(Velocity { x: 0.0, y: 0.0 });
            }
            EntityType::Shelter => {
                entity.shelter = Some(Shelter::new(ShelterType::Cave));
                entity.color = macroquad::prelude::GREEN;
                entity.ai_state = AIState::Idle;
            }
        }

        *next_entity_id += 1;
        entity
    }

    /// Pre-populate the pool with entities
    pub fn pre_populate(&mut self, count: usize) {
        for _ in 0..count.min(self.max_size) {
            let entity = self.create_default_entity();
            self.available.push_back(entity);
        }
    }

    /// Get current pool statistics
    pub fn get_stats(&mut self) -> PoolStats {
        self.update_utilization_ratio();
        self.stats.clone()
    }

    /// Clear the pool (useful for level changes or game resets)
    pub fn clear(&mut self) {
        self.available.clear();
        self.stats = PoolStats::default();
    }

    /// Get the current size of the pool
    pub fn current_size(&self) -> usize {
        self.available.len()
    }

    /// Get the maximum pool size
    pub fn max_size(&self) -> usize {
        self.max_size
    }

    /// Check if the pool is empty
    pub fn is_empty(&self) -> bool {
        self.available.is_empty()
    }

    /// Check if the pool is full
    pub fn is_full(&self) -> bool {
        self.available.len() >= self.max_size
    }

    /// Resize the pool (won't shrink below current size)
    pub fn resize(&mut self, new_max_size: usize) {
        self.max_size = new_max_size;

        // If new size is smaller than current, trim excess entities
        while self.available.len() > self.max_size {
            self.available.pop_back();
        }
    }

    /// Get pool efficiency (reuse ratio)
    pub fn efficiency(&self) -> f32 {
        let total_acquisitions = self.stats.total_created + self.stats.total_reused;
        if total_acquisitions > 0 {
            self.stats.total_reused as f32 / total_acquisitions as f32
        } else {
            0.0
        }
    }

    // Private helper methods

    /// Reset an entity to default state for reuse
    fn reset_entity(&self, entity: &mut GameEntity) {
        entity.id = 0;
        entity.position = Position { x: 0.0, y: 0.0 };
        entity.velocity = None;
        entity.entity_type = EntityType::HostileInfected; // Default type
        entity.health = None;
        entity.combat_stats = None;
        entity.ai_state = AIState::Idle;
        entity.blood_meter = None;
        entity.vampire_abilities = None;
        entity.shelter = None;
        entity.shelter_occupancy = None;
        entity.color = macroquad::prelude::WHITE;
    }

    /// Create a new default entity
    fn create_default_entity(&self) -> GameEntity {
        GameEntity {
            id: 0,
            position: Position { x: 0.0, y: 0.0 },
            velocity: None,
            entity_type: EntityType::HostileInfected,
            health: None,
            combat_stats: None,
            ai_state: AIState::Idle,
            blood_meter: None,
            vampire_abilities: None,
            shelter: None,
            shelter_occupancy: None,
            color: macroquad::prelude::WHITE,
        }
    }

    /// Update utilization ratio for statistics
    fn update_utilization_ratio(&mut self) {
        if self.max_size > 0 {
            self.stats.utilization_ratio = self.available.len() as f32 / self.max_size as f32;
        }
    }
}

/// Builder for configuring entity pools
pub struct EntityPoolBuilder {
    max_size: usize,
    pre_populate_count: usize,
}

impl EntityPoolBuilder {
    /// Create a new pool builder
    pub fn new() -> Self {
        Self {
            max_size: 100,
            pre_populate_count: 0,
        }
    }

    /// Set maximum pool size
    pub fn with_max_size(mut self, max_size: usize) -> Self {
        self.max_size = max_size;
        self
    }

    /// Set number of entities to pre-populate
    pub fn with_pre_population(mut self, count: usize) -> Self {
        self.pre_populate_count = count;
        self
    }

    /// Build the entity pool
    pub fn build(self) -> EntityPool {
        let mut pool = EntityPool::new(self.max_size);

        if self.pre_populate_count > 0 {
            pool.pre_populate(self.pre_populate_count);
        }

        pool
    }
}

impl Default for EntityPoolBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Global entity pool manager for the game
pub struct GlobalEntityPool {
    pools: std::collections::HashMap<String, EntityPool>,
}

impl GlobalEntityPool {
    /// Create a new global pool manager
    pub fn new() -> Self {
        Self {
            pools: std::collections::HashMap::new(),
        }
    }

    /// Get or create a pool by name
    pub fn get_pool(&mut self, name: &str) -> &mut EntityPool {
        self.pools
            .entry(name.to_string())
            .or_insert_with(|| EntityPool::with_defaults())
    }

    /// Initialize common pools for the game
    pub fn initialize_game_pools(&mut self) {
        // Pool for hostile entities
        self.pools.insert(
            "hostile".to_string(),
            EntityPoolBuilder::new()
                .with_max_size(150)
                .with_pre_population(20)
                .build(),
        );

        // Pool for animals
        self.pools.insert(
            "animals".to_string(),
            EntityPoolBuilder::new()
                .with_max_size(50)
                .with_pre_population(10)
                .build(),
        );

        // Pool for clan members
        self.pools.insert(
            "clan".to_string(),
            EntityPoolBuilder::new()
                .with_max_size(30)
                .with_pre_population(5)
                .build(),
        );

        // Pool for shelters
        self.pools.insert(
            "shelters".to_string(),
            EntityPoolBuilder::new()
                .with_max_size(20)
                .with_pre_population(3)
                .build(),
        );
    }

    /// Get statistics for all pools
    pub fn get_all_stats(&mut self) -> std::collections::HashMap<String, PoolStats> {
        self.pools
            .iter_mut()
            .map(|(name, pool)| (name.clone(), pool.get_stats()))
            .collect()
    }

    /// Clear all pools
    pub fn clear_all(&mut self) {
        for pool in self.pools.values_mut() {
            pool.clear();
        }
    }
}

impl Default for GlobalEntityPool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use macroquad::prelude::WHITE;

    #[test]
    fn test_entity_pool_basic() {
        let mut pool = EntityPool::new(10);
        let mut next_id = 1;

        // Acquire an entity
        let entity = pool.spawn_entity(
            EntityType::HostileInfected,
            Position { x: 100.0, y: 100.0 },
            &mut next_id,
        );

        assert_eq!(entity.id, 1);
        assert_eq!(entity.position.x, 100.0);

        // Return entity to pool
        pool.release(entity);
        assert_eq!(pool.current_size(), 1);

        // Acquire again should reuse
        let reused_entity = pool.spawn_entity(
            EntityType::Animal,
            Position { x: 200.0, y: 200.0 },
            &mut next_id,
        );

        assert_eq!(reused_entity.id, 2); // Should have new ID
        assert_eq!(reused_entity.position.x, 200.0);
    }

    #[test]
    fn test_pool_statistics() {
        let mut pool = EntityPool::new(5);
        let mut next_id = 1;

        // Create some entities - first one creates new, rest reuse
        let mut entities = Vec::new();
        for _ in 0..3 {
            let entity = pool.spawn_entity(
                EntityType::HostileInfected,
                Position { x: 0.0, y: 0.0 },
                &mut next_id,
            );
            entities.push(entity);
        }

        // Return all entities to pool
        for entity in entities {
            pool.release(entity);
        }

        let stats = pool.get_stats();
        assert_eq!(stats.total_created, 3);
        assert_eq!(stats.total_returned, 3);
        assert_eq!(stats.total_reused, 0); // No reuse in this test
    }

    #[test]
    fn test_pool_efficiency() {
        let mut pool = EntityPool::new(5);
        let mut next_id = 1;

        // Create and return an entity
        let entity = pool.spawn_entity(
            EntityType::HostileInfected,
            Position { x: 0.0, y: 0.0 },
            &mut next_id,
        );
        pool.release(entity);

        // Reuse the entity
        let _reused = pool.spawn_entity(
            EntityType::Animal,
            Position { x: 0.0, y: 0.0 },
            &mut next_id,
        );

        // Should have 50% efficiency (1 reused out of 2 total acquisitions)
        assert!((pool.efficiency() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_pool_builder() {
        let pool = EntityPoolBuilder::new()
            .with_max_size(20)
            .with_pre_population(5)
            .build();

        assert_eq!(pool.max_size(), 20);
        assert_eq!(pool.current_size(), 5);
    }

    #[test]
    fn test_global_pool_manager() {
        let mut global_pool = GlobalEntityPool::new();
        global_pool.initialize_game_pools();

        let hostile_pool = global_pool.get_pool("hostile");
        assert!(!hostile_pool.is_empty()); // Should be pre-populated

        let stats = global_pool.get_all_stats();
        assert!(stats.contains_key("hostile"));
        assert!(stats.contains_key("animals"));
    }

    #[test]
    fn test_pool_max_size_enforcement() {
        let mut pool = EntityPool::new(2);
        let mut next_id = 1;

        // Fill the pool beyond capacity
        for _ in 0..5 {
            let entity = pool.spawn_entity(
                EntityType::HostileInfected,
                Position { x: 0.0, y: 0.0 },
                &mut next_id,
            );
            pool.release(entity);
        }

        // Pool should not exceed max size
        assert!(pool.current_size() <= pool.max_size());
    }
}
