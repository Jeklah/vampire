# Vampire RPG - Technical Architecture Documentation

## Overview

This document describes the technical architecture of the Vampire RPG, a 2D survival game built with Rust and Macroquad. The architecture follows modern software engineering principles with a focus on modularity, maintainability, and performance.

## Architecture Principles

### Core Design Philosophy

1. **Single Responsibility Principle**: Each module has one clear, focused purpose
2. **Composition over Inheritance**: Systems are composed rather than inherited
3. **Dependency Injection**: Systems operate on data passed to them
4. **Immutable Data Flow**: Data flows unidirectionally through the system
5. **Zero-Cost Abstractions**: Leverage Rust's type system for performance

### Architectural Patterns

- **Entity-Component Pattern**: Entities are composed of components
- **System Architecture**: Focused systems operate on entity data
- **Coordinator Pattern**: Central GameState orchestrates systems
- **Observer Pattern**: Systems react to state changes
- **Strategy Pattern**: Pluggable AI behaviors and abilities

## System Architecture

### High-Level Overview

```
┌─────────────────────────────────────────────────────────┐
│                    Application Layer                     │
├─────────────────────────────────────────────────────────┤
│  main.rs  →  GameState  →  InputHandler  →  Renderer   │
├─────────────────────────────────────────────────────────┤
│                     Systems Layer                       │
├─────────────────────────────────────────────────────────┤
│  TimeSystem  │  WorldSystem  │  PlayerSystem  │  AI     │
│  BloodSystem │  ObjectivesSystem              │  etc.   │
├─────────────────────────────────────────────────────────┤
│                   Components Layer                      │
├─────────────────────────────────────────────────────────┤
│  Entities  │  Vampire  │  Combat  │  Environment  │ ... │
├─────────────────────────────────────────────────────────┤
│                    Foundation Layer                     │
├─────────────────────────────────────────────────────────┤
│            Macroquad  │  Serde  │  Rust STD            │
└─────────────────────────────────────────────────────────┘
```

### Data Flow Architecture

```
Input → InputHandler → GameState → Systems → Components → Renderer → Display
  ↑                                    ↓
  └─────────── Game Loop ──────────────┘
```

## Module Structure

### Core Modules

#### `main.rs` - Application Entry Point
- **Responsibility**: Game loop and window management
- **Size**: ~50 lines
- **Dependencies**: GameState, InputHandler, Renderer

```rust
// Simplified main loop structure
#[macroquad::main(window_conf)]
async fn main() {
    let mut game_state = GameState::new();
    let mut input_handler = InputHandler::new();
    let renderer = Renderer::new();
    
    loop {
        input_handler.update();
        game_state.update(&input_handler, delta_time);
        renderer.render(&game_state);
        next_frame().await;
    }
}
```

#### `game_state.rs` - System Coordinator
- **Responsibility**: Orchestrates all game systems
- **Size**: ~295 lines
- **Pattern**: Coordinator/Mediator
- **Key Methods**:
  - `new()` - Initialize all systems
  - `update()` - Coordinate system updates
  - System-specific update methods

### Systems Layer

#### `systems/time.rs` - Time Management
- **Responsibility**: Day/night cycle, time progression
- **Size**: ~227 lines
- **Key Features**:
  - Configurable day length
  - Sunlight intensity calculations
  - Time period detection
  - Vampire-specific danger assessment

#### `systems/world.rs` - World Management
- **Responsibility**: Entity spawning, world initialization
- **Size**: ~447 lines
- **Key Features**:
  - Safe spawn positioning
  - Collision detection
  - Environment setup
  - Spawn validation

#### `systems/player.rs` - Player Logic
- **Responsibility**: Player actions, movement, progression
- **Size**: ~490 lines
- **Key Features**:
  - Action validation
  - Ability progression
  - Status tracking
  - Movement with modifiers

#### `systems/ai.rs` - AI Behavior
- **Responsibility**: NPC behavior, decision making
- **Size**: ~425 lines
- **Key Features**:
  - Multiple AI states
  - Behavior switching
  - Personality traits
  - Batch processing

#### `systems/blood.rs` - Survival Mechanics
- **Responsibility**: Blood management, feeding, survival
- **Size**: ~487 lines
- **Key Features**:
  - Blood drain calculations
  - Feeding mechanics
  - Survival scoring
  - Status monitoring

#### `systems/objectives.rs` - Progress Tracking
- **Responsibility**: Objective tracking, phase progression
- **Size**: ~551 lines
- **Key Features**:
  - Multi-phase objectives
  - Progress detection
  - Achievement tracking
  - Analytics

### Component Layer

#### `components/entities.rs` - Core Entity Types
- **Core Components**: Position, Velocity, Health, GameEntity
- **Pattern**: Entity-Component
- **Features**: Distance calculations, status tracking

#### `components/vampire.rs` - Vampire-Specific Components
- **Key Components**: BloodMeter, VampireAbilities, SunlightVulnerability
- **Features**: Blood management, ability progression

#### `components/combat.rs` - Combat Components
- **Key Components**: CombatStats, AIState, AIBehavior
- **Features**: Combat calculations, AI state management

#### `components/game_data.rs` - Game Progression
- **Key Components**: GamePhase, EntityType, Clan, Player
- **Features**: Phase management, clan relations

#### `components/environment.rs` - Environmental Elements
- **Key Components**: Star, Moon, BloodParticle, GroundTile
- **Features**: Atmospheric effects, terrain system

### Input/Output Layer

#### `input/mod.rs` - Input Management
- **Responsibility**: Centralized input handling
- **Features**: Key state tracking, input abstraction

#### `rendering/mod.rs` - Rendering System
- **Responsibility**: All visual output
- **Features**: Sprite rendering, UI, particle effects

## Data Models

### Entity Representation

```rust
pub struct GameEntity {
    pub id: u32,
    pub position: Position,
    pub velocity: Velocity,
    pub health: Option<Health>,
    pub blood_meter: Option<BloodMeter>,
    pub abilities: Option<VampireAbilities>,
    pub combat_stats: Option<CombatStats>,
    pub entity_type: EntityType,
    pub color: Color,
    pub ai_target: Option<u32>,
    pub ai_state: AIState,
    pub facing_direction: f32,
}
```

### System State

Each system maintains its own state and operates on shared entity data:

```rust
// Systems are stateless and operate on data
impl PlayerSystem {
    pub fn update_movement(
        entities: &mut Vec<GameEntity>,
        input_handler: &InputHandler,
        player_id: u32,
        is_day: bool,
        delta_time: f32,
    ) { /* ... */ }
}
```

## Performance Considerations

### Memory Management

1. **Entity Storage**: Vec-based storage for cache efficiency
2. **Component Sparsity**: Optional components for memory efficiency
3. **Batch Operations**: Systems process entities in batches
4. **Zero Allocations**: Minimal runtime allocations in hot paths

### Update Optimization

1. **System Ordering**: Dependencies respected in update order
2. **Early Returns**: Skip expensive operations when paused
3. **Spatial Partitioning**: Distance-based culling for AI/rendering
4. **Delta Time**: Frame-rate independent updates

### Rendering Optimization

1. **Frustum Culling**: Only render on-screen entities
2. **Level of Detail**: Different sprite sizes based on zoom
3. **Batch Rendering**: Group similar rendering operations
4. **UI Optimization**: Cached UI elements where possible

## Error Handling Strategy

### Error Types

1. **Recoverable Errors**: Use `Result<T, E>` for operations that can fail
2. **Optional Values**: Use `Option<T>` for missing data
3. **Panics**: Reserved for programmer errors and unrecoverable states
4. **Validation**: Input validation at system boundaries

### Error Propagation

```rust
// Example error handling pattern
pub fn attempt_feeding(&mut self) -> Result<BloodGain, FeedingError> {
    let target = self.find_feeding_target()
        .ok_or(FeedingError::NoTarget)?;
    
    let blood_gained = self.calculate_blood_gain(&target)?;
    self.apply_feeding_effects(blood_gained)?;
    
    Ok(BloodGain(blood_gained))
}
```

## Testing Strategy

### Unit Testing

- Each system has comprehensive unit tests
- Component behavior is tested in isolation
- Mock data used for system testing

### Integration Testing

- System interactions tested through GameState
- End-to-end scenarios validated
- Performance benchmarks for critical paths

### Test Coverage

- **Target**: 80%+ line coverage
- **Focus**: Critical game mechanics
- **Tools**: `cargo test`, `cargo tarpaulin`

## Security Considerations

### Input Validation

- All player input validated at system boundaries
- Numeric inputs clamped to reasonable ranges
- State transitions validated for consistency

### Save Game Integrity

- Serialization through serde with validation
- Version compatibility checks
- Checksum validation for save files

## Scalability Considerations

### Code Organization

- Modular systems allow parallel development
- Clear interfaces enable team scaling
- Documentation supports onboarding

### Performance Scaling

- Entity count scales with efficient data structures
- System complexity isolated and optimizable
- Rendering scales with culling and LOD

### Feature Scaling

- New systems follow established patterns
- Plugin architecture possible with current design
- Component system supports new entity types

## Dependencies

### Core Dependencies

```toml
[dependencies]
macroquad = "0.4"     # Game engine and rendering
serde = "1.0"         # Serialization
rand = "0.8"          # Random number generation
```

### Development Dependencies

```toml
[dev-dependencies]
criterion = "0.4"     # Benchmarking
mockall = "0.11"      # Mocking for tests
```

## Deployment Architecture

### Build Configuration

- **Debug**: Fast compilation, debug symbols
- **Release**: Optimizations enabled, no debug info
- **Target Platforms**: Windows, Linux, macOS

### Asset Management

- Embedded resources for simple deployment
- External assets for modding support
- Compressed assets for reduced file size

## Future Architecture Considerations

### Potential Improvements

1. **Event System**: Decoupled communication between systems
2. **Entity Query System**: More efficient entity iteration
3. **Resource Management**: Centralized asset loading
4. **Scripting Support**: Lua/WASM integration for modding

### Scalability Paths

1. **Networking**: Multiplayer architecture considerations
2. **Save System**: Robust persistence layer
3. **Mod Support**: Plugin architecture
4. **Performance**: Multithreading for heavy systems

## Documentation Standards

### Code Documentation

- All public APIs documented with rustdoc
- Examples provided for complex systems
- Architecture decisions recorded in code comments

### System Documentation

- Each system has design rationale documented
- Performance characteristics noted
- Usage patterns and gotchas documented

This architecture provides a solid foundation for the Vampire RPG while maintaining flexibility for future enhancements and modifications.