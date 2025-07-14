# Development Guide - Vampire RPG

This guide provides information for developers who want to extend and improve the Vampire RPG game.

## Project Overview

The Vampire RPG is built using Rust and the Macroquad game framework. It implements a simplified Entity-Component-System (ECS) architecture with custom game systems for managing vampire mechanics, combat, and clan relationships.

## Architecture

### Core Structure

```
GameState
├── entities: Vec<GameEntity>     # All game objects
├── time: TimeSystem             # Day/night cycle
├── clans: HashMap<String, Clan> # Clan management
├── phase: GamePhase            # Game progression
└── Various counters & flags
```

### Entity System

Each `GameEntity` contains:
- **Core Components**: Position, Velocity, Health
- **Optional Components**: BloodMeter, VampireAbilities, CombatStats
- **AI State**: Current behavior and target
- **Type**: Player, ClanLeader, HostileInfected, Animal

### Game Systems

1. **TimeSystem**: Manages day/night cycles and sunlight effects
2. **Blood System**: Handles feeding, starvation, and blood-based abilities
3. **Combat System**: Manages attacks, damage calculation, and death
4. **AI System**: Controls NPC behavior and movement
5. **Interaction System**: Handles player-NPC interactions

## Adding New Features

### Adding a New Entity Type

1. **Define the Type**:
```rust
// In EntityType enum
NewEntityType(String),
```

2. **Create Spawn Function**:
```rust
fn spawn_new_entity(&mut self, x: f32, y: f32) {
    let entity = GameEntity {
        id: self.next_entity_id,
        position: Position { x, y },
        // ... set other components
        entity_type: EntityType::NewEntityType("data".to_string()),
        // ...
    };
    self.entities.push(entity);
    self.next_entity_id += 1;
}
```

3. **Handle in Update Loop**:
```rust
// In appropriate update functions
match entity.entity_type {
    EntityType::NewEntityType(ref data) => {
        // Custom behavior
    }
    // ... other types
}
```

### Adding New Vampire Abilities

1. **Extend VampireAbilities**:
```rust
struct VampireAbilities {
    // ... existing abilities
    new_ability: f32,
    new_ability_unlocked: bool,
}
```

2. **Add Blood Cost**:
```rust
fn use_new_ability(&mut self, player_id: u32) -> bool {
    let blood_cost = 15.0;
    if let Some(player) = self.entities.iter_mut().find(|e| e.id == player_id) {
        if let Some(blood_meter) = &mut player.blood_meter {
            if blood_meter.current >= blood_cost {
                blood_meter.current -= blood_cost;
                // Apply ability effect
                return true;
            }
        }
    }
    false
}
```

3. **Add Key Binding**:
```rust
// In handle_input()
if is_key_pressed(KeyCode::Q) {
    self.use_new_ability(self.player_id);
}
```

### Adding New Game Phases

1. **Extend GamePhase Enum**:
```rust
enum GamePhase {
    // ... existing phases
    NewPhase,
}
```

2. **Add Phase Objectives**:
```rust
// In phase transition logic
GamePhase::NewPhase => {
    self.phase_objectives = vec![
        "Complete new objective".to_string(),
        "Discover new mechanic".to_string(),
    ];
}
```

3. **Handle Phase Logic**:
```rust
// In check_objectives()
match self.phase {
    GamePhase::NewPhase => {
        // Check completion conditions
        if some_condition {
            self.complete_objective("Complete new objective");
        }
    }
}
```

### Adding New Clan Types

1. **Initialize in GameState**:
```rust
// In initialize_world()
self.clans.insert(
    "New-Clan".to_string(),
    Clan::new("New-Clan", "Leader Name", 20),
);
```

2. **Spawn Clan Entities**:
```rust
self.spawn_clan_leader("Leader Name", "New-Clan", 300.0, 300.0, ORANGE);
```

3. **Add Clan-Specific Behavior**:
```rust
// In interaction logic
if clan_name == "New-Clan" {
    // Special interaction behavior
}
```

## Performance Considerations

### Entity Management

- **Dead Entity Cleanup**: Remove entities with health <= 0
- **Spatial Culling**: Only update/render entities near the player
- **Component Caching**: Cache frequently accessed components

### Optimization Tips

1. **Batch Operations**: Group similar operations together
2. **Limit AI Updates**: Don't update all AI every frame
3. **Use Object Pools**: Reuse entity slots instead of creating new ones
4. **Spatial Partitioning**: Use quadtrees for large worlds

## Testing Your Changes

### Manual Testing Checklist

- [ ] Player movement works correctly
- [ ] Blood system functions (feeding, starvation)
- [ ] Day/night cycle affects gameplay
- [ ] Combat system is responsive
- [ ] Clan interactions work
- [ ] Objectives update properly
- [ ] UI displays correctly
- [ ] Game doesn't crash on edge cases

### Debug Features

Enable debug mode by adding to main.rs:
```rust
const DEBUG_MODE: bool = true;

// In render()
if DEBUG_MODE {
    // Draw entity IDs, health bars, etc.
    for entity in &self.entities {
        draw_text(&format!("{}", entity.id), entity.position.x, entity.position.y - 20.0, 12.0, YELLOW);
    }
}
```

## Common Patterns

### State Management
```rust
// Use enums for state machines
enum PlayerState {
    Normal,
    Feeding,
    InCombat,
    InDialogue,
}

// Track state changes
if old_state != new_state {
    self.on_state_change(old_state, new_state);
}
```

### Event System
```rust
// Simple event queue
enum GameEvent {
    EntityDied(u32),
    ClanAllied(String),
    PhaseCompleted,
}

// Process events after updates
for event in &self.event_queue {
    match event {
        GameEvent::EntityDied(id) => {
            // Handle death
        }
        // ... other events
    }
}
self.event_queue.clear();
```

### Resource Management
```rust
// Use constants for balance
const BLOOD_DRAIN_RATE: f32 = 1.0;
const SUNLIGHT_DAMAGE: f32 = 5.0;
const FEED_RANGE: f32 = 50.0;

// Group related values
struct GameBalance {
    blood_drain_rate: f32,
    sunlight_damage: f32,
    feed_range: f32,
}
```

## Future Features to Implement

### High Priority
- [ ] Save/Load system using serde
- [ ] Sound effects and background music
- [ ] Better graphics and animations
- [ ] More sophisticated AI behaviors

### Medium Priority
- [ ] Territory control system
- [ ] Inventory and items
- [ ] More vampire abilities
- [ ] Dynamic dialogue system

### Low Priority
- [ ] Multiplayer support
- [ ] Procedural world generation
- [ ] Achievement system
- [ ] Mod support

## Contributing Guidelines

1. **Code Style**: Follow standard Rust conventions
2. **Documentation**: Add docs for public functions
3. **Testing**: Test new features thoroughly
4. **Performance**: Profile changes that affect update loops
5. **Compatibility**: Ensure changes work on target platforms

## Debugging Tips

### Common Issues

1. **Entity Not Updating**: Check if entity has required components
2. **Performance Problems**: Profile with `cargo build --release`
3. **Rendering Issues**: Verify camera positioning and screen bounds
4. **Input Not Working**: Check key mapping and event handling

### Useful Debug Commands

```rust
// Print entity count
println!("Entities: {}", self.entities.len());

// Print player stats
if let Some(player) = self.entities.iter().find(|e| e.id == self.player_id) {
    println!("Player health: {:?}", player.health);
}

// Track game time
println!("Game time: {:.2}, Day: {}", self.time.current_time, self.time.day_count);
```

## Resources

- [Macroquad Documentation](https://docs.rs/macroquad/)
- [Rust Game Development Book](https://pragprog.com/titles/hwrust/hands-on-rust/)
- [ECS Pattern Overview](https://github.com/SanderMertens/ecs-faq)
- [Game Programming Patterns](https://gameprogrammingpatterns.com/)

Remember: Start small, test frequently, and iterate on your designs. The current architecture is designed to be extensible while remaining simple to understand and modify.