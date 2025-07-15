# Vampire RPG - Development Guidelines

## Overview

This document provides comprehensive guidelines for developing and maintaining the Vampire RPG codebase. These guidelines ensure code quality, consistency, and maintainability across the project.

## Code Style and Formatting

### Rust Style Guidelines

Follow the official Rust style guidelines with these project-specific additions:

#### Naming Conventions

```rust
// Structs, Enums, Traits: PascalCase
pub struct GameEntity { }
pub enum EntityType { }
pub trait System { }

// Functions, variables, modules: snake_case
pub fn update_player_movement() { }
let player_health = 100.0;
mod blood_system;

// Constants: SCREAMING_SNAKE_CASE
const MAX_ENTITY_COUNT: usize = 1000;
const DEFAULT_PLAYER_SPEED: f32 = 130.0;

// Type parameters: Single uppercase letter
pub fn spawn_entity<T: Component>(component: T) { }
```

#### Code Organization

```rust
// Order of items in modules:
// 1. use statements
// 2. Constants
// 3. Type definitions
// 4. Structs/Enums
// 5. Implementations
// 6. Functions
// 7. Tests (in #[cfg(test)] module)

use crate::components::*;
use macroquad::prelude::*;
use std::collections::HashMap;

const DEFAULT_SPEED: f32 = 100.0;

#[derive(Debug, Clone)]
pub struct PlayerSystem;

impl PlayerSystem {
    pub fn new() -> Self { }
    // ... other methods
}

pub fn helper_function() { }

#[cfg(test)]
mod tests {
    use super::*;
    // ... tests
}
```

### Formatting Rules

- Use `cargo fmt` for automatic formatting
- Line length: 100 characters maximum
- Indentation: 4 spaces (no tabs)
- Trailing commas in multi-line constructs

```rust
// Good: Trailing comma in multi-line
let config = GameConfig {
    window_width: 1280,
    window_height: 720,
    fullscreen: false,  // <- trailing comma
};

// Good: Single line when short
let pos = Position { x: 100.0, y: 200.0 };
```

## System Design Guidelines

### Creating New Systems

When adding a new system, follow this template:

```rust
//! System Name Module
//!
//! Brief description of what this system handles.
//! Explain the main responsibilities and any important design decisions.

use crate::components::*;
use crate::systems::SystemName;

/// System responsible for [specific functionality]
pub struct SystemName;

impl SystemName {
    /// Create a new instance of the system
    pub fn new() -> Self {
        Self
    }

    /// Main update function for the system
    pub fn update(
        entities: &mut Vec<GameEntity>,
        // ... other parameters
        delta_time: f32,
    ) {
        // Implementation
    }

    /// Helper functions should be private when possible
    fn helper_function() {
        // Implementation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_functionality() {
        // Test implementation
    }
}
```

### System Responsibilities

Each system should have a single, clear responsibility:

- **Good**: PlayerSystem handles player movement, actions, and status
- **Bad**: PlayerSystem handles player logic AND AI logic AND rendering

### Data Flow Patterns

Systems should follow these data flow patterns:

```rust
// Pattern 1: Pure functions operating on data
impl SystemName {
    pub fn process_entities(
        entities: &mut Vec<GameEntity>,
        external_data: &ExternalData,
    ) -> ProcessResult {
        // Process entities without side effects
    }
}

// Pattern 2: System with internal state (use sparingly)
pub struct StatefulSystem {
    internal_state: SomeState,
}

impl StatefulSystem {
    pub fn update(&mut self, entities: &[GameEntity]) {
        // Update internal state based on entities
    }
}
```

## Error Handling Guidelines

### Error Types

Use appropriate error handling for different scenarios:

```rust
// Use Result for operations that can fail
pub fn spawn_entity(pos: Position) -> Result<EntityId, SpawnError> {
    if !is_valid_position(pos) {
        return Err(SpawnError::InvalidPosition);
    }
    Ok(create_entity(pos))
}

// Use Option for missing data
pub fn find_entity_by_id(id: EntityId) -> Option<&GameEntity> {
    entities.iter().find(|e| e.id == id)
}

// Use panic! only for programmer errors
pub fn get_player_entity(&self) -> &GameEntity {
    self.entities
        .iter()
        .find(|e| matches!(e.entity_type, EntityType::Player))
        .expect("Player entity must exist")
}
```

### Error Propagation

Use the `?` operator for clean error propagation:

```rust
pub fn complex_operation() -> Result<Success, GameError> {
    let entity = find_target_entity()?;
    let result = process_entity(entity)?;
    apply_result(result)?;
    Ok(Success)
}
```

## Testing Guidelines

### Unit Testing

Every system should have comprehensive unit tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Helper function for creating test data
    fn create_test_entity() -> GameEntity {
        GameEntity {
            id: 0,
            position: Position { x: 100.0, y: 100.0 },
            // ... other fields
        }
    }

    #[test]
    fn test_basic_functionality() {
        let mut entities = vec![create_test_entity()];
        let result = SystemName::update(&mut entities, 1.0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_edge_case() {
        // Test edge cases and error conditions
        let empty_entities = vec![];
        let result = SystemName::update(&mut empty_entities, 0.0);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn test_performance() {
        // Basic performance tests for critical paths
        let start = std::time::Instant::now();
        perform_operation();
        assert!(start.elapsed().as_millis() < 16); // Under 1 frame at 60fps
    }
}
```

### Integration Testing

Test system interactions:

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_player_feeding_workflow() {
        let mut game_state = GameState::new();
        
        // Setup test scenario
        let target_id = spawn_test_animal(&mut game_state);
        
        // Execute workflow
        let feeding_result = PlayerSystem::attempt_feeding(
            &mut game_state.entities,
            game_state.player_id,
        );
        
        // Verify results
        assert!(feeding_result);
        assert!(target_is_dead(&game_state.entities, target_id));
        assert!(player_blood_increased(&game_state.entities, game_state.player_id));
    }
}
```

### Test Organization

- Unit tests in the same file as the code they test
- Integration tests in separate files under `tests/` directory
- Helper functions in `test_utils` module
- Mock data generators for consistent test scenarios

## Documentation Guidelines

### Code Documentation

Use rustdoc for all public APIs:

```rust
/// Attempts to feed the player on a nearby target entity.
///
/// This function searches for valid feeding targets within range and executes
/// the feeding action if a target is found. Feeding restores blood and health
/// while killing the target entity.
///
/// # Arguments
///
/// * `entities` - Mutable reference to the entity collection
/// * `player_id` - ID of the player entity
///
/// # Returns
///
/// * `true` if feeding was successful
/// * `false` if no valid target was found
///
/// # Examples
///
/// ```rust
/// let success = PlayerSystem::attempt_feeding(&mut entities, player_id);
/// if success {
///     println!("Player fed successfully!");
/// }
/// ```
pub fn attempt_feeding(entities: &mut Vec<GameEntity>, player_id: u32) -> bool {
    // Implementation
}
```

### Module Documentation

Each module should have comprehensive documentation:

```rust
//! Player System Module
//!
//! This module handles all player-specific logic including movement, actions,
//! and progression. The PlayerSystem is responsible for:
//!
//! - Processing player input and movement
//! - Executing player actions (feeding, attacking, interacting)
//! - Managing player progression and abilities
//! - Validating player actions and resource costs
//!
//! # Design Decisions
//!
//! The player system operates on entity data passed to it rather than
//! maintaining internal state. This makes testing easier and keeps the
//! system pure and predictable.
//!
//! # Performance Considerations
//!
//! Player updates are optimized for single-entity operations since there's
//! only one player. More expensive operations like line-of-sight checks
//! are cached when possible.
```

### Architecture Documentation

Document important architectural decisions:

```rust
// ARCHITECTURE NOTE: We use Vec<GameEntity> instead of a more sophisticated
// ECS because:
// 1. Simple iteration patterns for small entity counts (<1000)
// 2. Cache-friendly for our use case
// 3. Easier to debug and reason about
// 4. Sufficient performance for our scale
```

## Performance Guidelines

### Memory Management

Follow these patterns for efficient memory usage:

```rust
// Pre-allocate collections when size is known
let mut entities = Vec::with_capacity(expected_count);

// Reuse allocations when possible
fn update_system(entities: &mut Vec<GameEntity>, temp_buffer: &mut Vec<UpdateData>) {
    temp_buffer.clear(); // Reuse existing allocation
    // ... process entities
}

// Use references to avoid unnecessary clones
fn process_entity(entity: &GameEntity) -> ProcessResult {
    // Work with reference, don't clone unless necessary
}
```

### Performance Patterns

```rust
// Early returns to avoid expensive operations
fn update_ai(entities: &mut Vec<GameEntity>) {
    for entity in entities.iter_mut() {
        // Skip dead entities early
        if matches!(entity.ai_state, AIState::Dead) {
            continue;
        }
        
        // Expensive AI processing only for active entities
        expensive_ai_update(entity);
    }
}

// Batch operations when possible
fn update_positions(entities: &mut Vec<GameEntity>, delta_time: f32) {
    for entity in entities.iter_mut() {
        entity.position.x += entity.velocity.x * delta_time;
        entity.position.y += entity.velocity.y * delta_time;
    }
}
```

### Profiling Guidelines

- Use `cargo bench` for performance testing
- Profile with `perf` or similar tools for hotspot identification
- Measure frame times, not just algorithmic complexity
- Test with realistic data sizes and scenarios

## Git Workflow

### Branch Strategy

- `main` - Stable, release-ready code
- `develop` - Integration branch for features
- `feature/system-name` - Individual feature branches
- `bugfix/issue-description` - Bug fix branches

### Commit Guidelines

Use conventional commit format:

```
type(scope): description

- feat(player): add ability progression system
- fix(ai): correct hostile entity targeting bug
- docs(readme): update installation instructions
- refactor(blood): extract feeding logic to separate function
- test(objectives): add tests for phase progression
```

### Pull Request Process

1. Create feature branch from `develop`
2. Implement changes with tests
3. Update documentation
4. Ensure all tests pass
5. Submit PR with clear description
6. Address review comments
7. Merge after approval

## Code Review Guidelines

### Review Checklist

- [ ] Code follows style guidelines
- [ ] Functions are well-documented
- [ ] Tests cover new functionality
- [ ] Performance implications considered
- [ ] Error handling is appropriate
- [ ] No obvious bugs or edge cases

### Review Comments

Be constructive and specific in review comments:

```
Good: "Consider using Option<T> here instead of unwrap() to handle the case 
      where the entity might not exist."

Bad: "This doesn't look right."
```

## Security Guidelines

### Input Validation

Always validate input at system boundaries:

```rust
pub fn set_player_health(entities: &mut Vec<GameEntity>, player_id: u32, health: f32) {
    // Validate input
    if health < 0.0 || health > MAX_HEALTH {
        warn!("Invalid health value: {}", health);
        return;
    }
    
    // Process valid input
    if let Some(player) = entities.iter_mut().find(|e| e.id == player_id) {
        if let Some(player_health) = &mut player.health {
            player_health.current = health.clamp(0.0, player_health.maximum);
        }
    }
}
```

### State Validation

Validate game state consistency:

```rust
fn validate_game_state(game_state: &GameState) -> Vec<ValidationError> {
    let mut errors = Vec::new();
    
    // Player must exist
    if !game_state.entities.iter().any(|e| e.id == game_state.player_id) {
        errors.push(ValidationError::MissingPlayer);
    }
    
    // Entity IDs must be unique
    // Health values must be valid
    // etc.
    
    errors
}
```

## Debugging Guidelines

### Logging

Use structured logging for debugging:

```rust
use log::{debug, info, warn, error};

// Log important state changes
info!("Player {} fed on entity {}", player_id, target_id);

// Debug information for development
debug!("Player position: ({}, {})", player.position.x, player.position.y);

// Warnings for recoverable issues
warn!("Entity {} has invalid health value: {}", entity.id, health);

// Errors for serious problems
error!("Failed to spawn entity: {:?}", spawn_error);
```

### Debug Features

Implement debug features behind feature flags:

```rust
#[cfg(feature = "debug")]
pub fn debug_render_ai_state(entity: &GameEntity) {
    draw_text(
        &format!("AI: {:?}", entity.ai_state),
        entity.position.x,
        entity.position.y - 20.0,
        16.0,
        RED,
    );
}
```

## Release Guidelines

### Version Management

Follow semantic versioning (SemVer):

- `MAJOR.MINOR.PATCH`
- Major: Breaking changes
- Minor: New features, backward compatible
- Patch: Bug fixes, backward compatible

### Release Checklist

- [ ] All tests pass
- [ ] Documentation updated
- [ ] Changelog updated
- [ ] Performance benchmarks run
- [ ] Memory usage checked
- [ ] Cross-platform testing completed

### Deployment

1. Tag release version
2. Generate release notes
3. Build release binaries
4. Test release builds
5. Publish release

These guidelines ensure consistent, high-quality code across the Vampire RPG project while maintaining performance and maintainability standards.