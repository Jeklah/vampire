# Entity Cleanup Bug Fix

**Date:** September 2025

**Issue:** All entities (enemies, clans, shelters) disappearing due to incorrect cleanup logic

**Status:** Fixed

**Impact:** Critical gameplay fix - entities now persist correctly based on type and importance

---

## Problem Description

### The "Vanishing Entities" Issue

After the ground spawning improvements, a critical bug emerged where:

1. **All Entities Disappearing**: Enemies, clan members, clan leaders, and shelters would vanish from the game world
2. **Immediate Cleanup**: Entities would be removed shortly after spawning (every 3 seconds during cleanup cycles)
3. **No Persistence**: Important entities like shelters and clan leaders were being removed despite their importance
4. **Broken Gameplay**: Players couldn't interact with clans or find shelter, making the game unplayable

### Root Cause Analysis

The issue was in the `batch_cleanup_with_macroquad()` method with **critically flawed logic**:

```rust
// PROBLEMATIC CODE (FIXED)
let should_keep = if entity.id == self.player_id {
    true // Never remove player
} else {
    let entity_vec = vec2(entity.position.x, entity.position.y);
    let distance_sq = player_vec.distance_squared(entity_vec);
    let is_alive = !matches!(entity.ai_state, AIState::Dead)
        && !entity.health.as_ref().map_or(false, |h| h.current <= 0.0);

    distance_sq < cleanup_distance_sq && is_alive  // ← CRITICAL BUG!
};
```

#### The Logic Error

The condition `distance_sq < cleanup_distance_sq && is_alive` means:
- **Keep entities that are CLOSE to player AND alive**
- **Remove entities that are FAR from player OR dead**

This was backwards! It should remove entities that are **far AND unimportant** OR **dead**.

#### Problems with the Logic

1. **Distance Blindness**: All distant entities were removed, regardless of importance
2. **No Type Awareness**: Shelters (permanent structures) treated same as common enemies
3. **Aggressive Distance**: 800px cleanup distance was too small for important NPCs
4. **Boolean Logic Error**: Used AND instead of proper conditional logic

---

## Solution Implementation

### 1. Fixed Core Logic Error

**Before (Broken):**
```rust
// This removed ALL distant entities
distance_sq < cleanup_distance_sq && is_alive
```

**After (Fixed):**
```rust
// Check if entity is dead first
let is_dead = matches!(entity.ai_state, AIState::Dead)
    || entity.health.as_ref().map_or(false, |h| h.current <= 0.0);

if is_dead {
    false // Always remove dead entities
} else {
    // Apply entity-type-specific cleanup rules for living entities
    let cleanup_distance = match &entity.entity_type {
        EntityType::Shelter => f32::INFINITY, // Never cleanup
        EntityType::ClanLeader(_) => base_cleanup_distance * 3.0, // 2400px
        EntityType::ClanMember(_) => base_cleanup_distance * 2.0, // 1600px
        EntityType::HostileInfected | EntityType::Animal => base_cleanup_distance, // 800px
        EntityType::Player => f32::INFINITY, // Never cleanup
    };

    distance_sq < cleanup_distance.powi(2)
}
```

### 2. Entity-Type-Aware Cleanup Rules

Implemented a hierarchy of importance for different entity types:

#### **Permanent Entities (Never Cleaned Up)**
- **Player**: The player character should never be removed
- **Shelter**: Permanent world structures that provide gameplay functionality

#### **Important NPCs (Large Cleanup Distance)**
- **ClanLeader**: 3x standard distance (2400px) - critical for story progression
- **ClanMember**: 2x standard distance (1600px) - valuable for interaction

#### **Common Entities (Standard Distance)**
- **HostileInfected**: Standard distance (800px) - can be respawned
- **Animal**: Standard distance (800px) - common and replaceable

#### **Dead Entities (Always Removed)**
- Any entity with `AIState::Dead` or health <= 0 is immediately cleaned up

### 3. Maintainable Code Structure

Created helper method for clean organization:

```rust
/// Get the cleanup distance for a specific entity type.
fn get_entity_cleanup_distance(&self, entity_type: &EntityType) -> f32 {
    match entity_type {
        EntityType::Shelter => f32::INFINITY, // Never cleanup shelters
        EntityType::ClanLeader(_) => self.entity_cleanup_distance * 3.0, // Very large distance
        EntityType::ClanMember(_) => self.entity_cleanup_distance * 2.0, // Large distance
        EntityType::HostileInfected | EntityType::Animal => self.entity_cleanup_distance, // Standard
        EntityType::Player => f32::INFINITY, // Never cleanup player
    }
}
```

---

## Technical Implementation Details

### Enhanced Cleanup Algorithm

```rust
pub fn batch_cleanup_with_macroquad(&mut self) {
    // For each entity (except player):
    // 1. Check if dead → always remove
    // 2. If alive → apply type-specific distance rules
    // 3. Preserve important entities at larger distances
    // 4. Only remove common entities when very distant
}
```

### Cleanup Distances by Type

| Entity Type | Cleanup Distance | Reasoning |
|-------------|------------------|-----------|
| **Player** | ∞ (Never) | Player character |
| **Shelter** | ∞ (Never) | Permanent structures |
| **ClanLeader** | 2400px (3x) | Story-critical NPCs |
| **ClanMember** | 1600px (2x) | Valuable interactions |
| **HostileInfected** | 800px (1x) | Common, respawnable |
| **Animal** | 800px (1x) | Common, respawnable |
| **Dead** | 0px (Always) | Should be removed |

### Performance Considerations

- **Efficient Vector Math**: Uses macroquad's `distance_squared()` to avoid expensive sqrt operations
- **Early Exit**: Dead entities removed immediately without distance calculations
- **Type-Based Branching**: Match expressions for O(1) type checking
- **Batch Processing**: All entities processed in single iteration

---

## Quality Assurance

### Comprehensive Test Coverage

Added `test_entity_cleanup_preserves_important_entities()` that verifies:

```rust
#[test]
fn test_entity_cleanup_preserves_important_entities() {
    // Test setup: Player at (400, 650)
    // - Shelter at 1500px distance → Should be preserved (never cleaned)
    // - ClanLeader at 1800px distance → Should be preserved (3x distance)
    // - HostileInfected at 1500px distance → Should be removed (too far)
    // - Dead entity close by → Should be removed (always clean dead)

    // Verify correct cleanup behavior for each type
    assert!(remaining_entities.contains(&shelter_id), "Shelter should never be cleaned up");
    assert!(remaining_entities.contains(&clan_leader_id), "Clan leader should be preserved");
    assert!(!remaining_entities.contains(&hostile_id), "Distant hostile should be cleaned up");
    assert!(!remaining_entities.contains(&dead_id), "Dead entity should be cleaned up");
}
```

### Debug Improvements

Enhanced cleanup logging:
```rust
self.add_debug_message(format!(
    "Smart cleanup: removed {} entities (preserving important types)",
    removed_count
));
```

---

## Results

### Before Fix
```
World State: Empty after 3 seconds
- Player: ✅ Exists
- Shelters: ❌ All removed
- Clan Leaders: ❌ All removed
- Clan Members: ❌ All removed
- Animals: ❌ All removed
- Hostiles: ❌ All removed
Result: Unplayable game
```

### After Fix
```
World State: Populated and persistent
- Player: ✅ Always preserved
- Shelters: ✅ Never cleaned up (permanent)
- Clan Leaders: ✅ Preserved at large distances (2400px)
- Clan Members: ✅ Preserved at medium distances (1600px)
- Animals: ✅ Cleaned up only when very distant (800px)
- Hostiles: ✅ Cleaned up only when very distant (800px)
Result: Fully functional gameplay
```

### Key Improvements

✅ **Shelters Persist**: Permanent world structures never disappear
✅ **Clan System Works**: Leaders and members preserved for interaction
✅ **Balanced Cleanup**: Common entities cleaned at appropriate distances
✅ **Dead Entity Removal**: Corpses properly cleaned up
✅ **Performance Maintained**: Efficient distance-based cleanup
✅ **Type Safety**: Entity-aware cleanup rules
✅ **Test Coverage**: Comprehensive verification of fix

---

## Prevention Strategy

### 1. Code Review Guidelines

**Critical Logic Patterns to Watch:**
- Boolean conditions involving AND/OR with distance checks
- Entity cleanup logic that doesn't consider entity types
- Uniform treatment of different entity categories

**Required Checks:**
```rust
// ❌ DANGEROUS: Treats all entities the same
if distance > cleanup_distance { remove_entity(); }

// ✅ SAFE: Entity-type-aware cleanup
match entity.entity_type {
    EntityType::Shelter => false, // Never remove
    EntityType::ClanLeader(_) => distance > (cleanup_distance * 3.0),
    // ... etc
}
```

### 2. Testing Requirements

**Mandatory tests for cleanup changes:**
```bash
cargo test test_entity_cleanup_preserves_important_entities
cargo test game_state::tests  # All game state tests
```

### 3. Entity Type Guidelines

**New Entity Types Must:**
- Be added to `get_entity_cleanup_distance()`
- Have appropriate cleanup rules based on gameplay importance
- Be covered in cleanup tests
- Consider persistence requirements

### 4. Debug Monitoring

```rust
// Always log cleanup actions for monitoring
self.add_debug_message(format!(
    "Smart cleanup: removed {} entities (preserving important types)",
    removed_count
));
```

---

## Impact Assessment

### Gameplay Impact
- **Restored Functionality**: Clan system, shelter system, and enemy encounters work correctly
- **Improved Balance**: Important entities persist while managing performance
- **Player Experience**: Consistent, predictable world populated with entities

### Technical Impact
- **Bug Resolution**: Fixed critical logic error that was removing all entities
- **Code Quality**: Entity-type-aware cleanup with clear separation of concerns
- **Maintainability**: Well-documented system with comprehensive test coverage
- **Performance**: Maintained efficient cleanup while adding type awareness

### Future Considerations
- Entity cleanup rules can be easily extended for new entity types
- Cleanup distances can be tuned per entity type for balance
- System architecture supports additional complexity (e.g., relationship-based cleanup)

---

This fix resolves the fundamental entity persistence issue while establishing a robust foundation for entity lifecycle management in the vampire RPG.
