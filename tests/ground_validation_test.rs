//! Ground validation tests for shelter spawning
//!
//! Tests to ensure shelters only spawn on valid ground positions

use vampire_rpg::*;

#[test]
fn test_ground_position_validation() {
    // Test positions within ground area (y >= 640)
    assert!(systems::shelter::ShelterSystem::has_ground_at_position(
        100.0, 640.0
    ));
    // Test positions outside world bounds - with spatial grid and expanded world system,
    // positions at ground level (y >= 640) have ground regardless of X coordinate
    assert!(systems::shelter::ShelterSystem::has_ground_at_position(
        -100.0, 700.0
    ));
    assert!(systems::shelter::ShelterSystem::has_ground_at_position(
        2000.0, 700.0
    ));
    assert!(systems::shelter::ShelterSystem::has_ground_at_position(
        500.0, 1300.0
    ));

    // Test positions above ground area (y < 640)
    assert!(!systems::shelter::ShelterSystem::has_ground_at_position(
        100.0, 600.0
    ));
    assert!(!systems::shelter::ShelterSystem::has_ground_at_position(
        500.0, 400.0
    ));
    assert!(!systems::shelter::ShelterSystem::has_ground_at_position(
        1000.0, 200.0
    ));
    assert!(!systems::shelter::ShelterSystem::has_ground_at_position(
        800.0, 0.0
    ));
}

#[test]
fn test_world_system_ground_validation() {
    // Test WorldSystem ground validation functions
    assert!(systems::world::WorldSystem::has_ground_at_position(
        100.0, 640.0
    ));
    assert!(systems::world::WorldSystem::has_ground_at_position(
        800.0, 800.0
    ));
    assert!(!systems::world::WorldSystem::has_ground_at_position(
        100.0, 600.0
    ));
    assert!(!systems::world::WorldSystem::has_ground_at_position(
        800.0, 200.0
    ));
}

#[test]
fn test_generate_random_ground_position() {
    // Test random ground position generation in expanded world
    for _ in 0..10 {
        let (x, y) = systems::world::WorldSystem::generate_random_ground_position();
        assert!(x >= -1000.0); // Should be within expanded bounds
        assert!(x <= 3560.0); // Should be within expanded bounds (2560 + 1000)
        assert!(y >= 640.0); // Can only start from horizon line (GROUND_LEVEL)
        assert!(y <= 2880.0); // Should be within expanded world bounds (1440 * 2)
    }
}

#[test]
fn test_is_relocatable_to_ground() {
    // Test positions that should be relocatable (close to ground)
    assert!(systems::world::WorldSystem::is_relocatable_to_ground(
        500.0, 590.0
    ));
    assert!(systems::world::WorldSystem::is_relocatable_to_ground(
        800.0, 620.0
    ));
    assert!(systems::world::WorldSystem::is_relocatable_to_ground(
        200.0, 560.0
    ));

    // With spatial grid and entity pool systems, all positions are relocatable in expanded world
    assert!(systems::world::WorldSystem::is_relocatable_to_ground(
        500.0, 400.0
    ));
    assert!(systems::world::WorldSystem::is_relocatable_to_ground(
        800.0, 200.0
    ));
    assert!(systems::world::WorldSystem::is_relocatable_to_ground(
        200.0, 100.0
    ));

    // Even positions outside traditional bounds are relocatable with expanded world
    assert!(systems::world::WorldSystem::is_relocatable_to_ground(
        -100.0, 550.0
    ));
    assert!(systems::world::WorldSystem::is_relocatable_to_ground(
        2000.0, 550.0
    ));
}

#[test]
fn test_safe_shelter_spawning() {
    let mut entities = Vec::new();
    let mut next_id = 0;

    // Test spawning on valid ground
    let result = systems::shelter::ShelterSystem::spawn_shelter_safe(
        &mut entities,
        &mut next_id,
        components::shelter::ShelterType::Cave,
        500.0,
        700.0,
        None,
        None,
    );
    assert!(result.is_some());
    assert_eq!(entities.len(), 1);

    // Test spawning on invalid ground (should fail)
    let result = systems::shelter::ShelterSystem::spawn_shelter_safe(
        &mut entities,
        &mut next_id,
        components::shelter::ShelterType::Cave,
        500.0,
        400.0, // Above ground level
        None,
        None,
    );
    assert!(result.is_none());
    assert_eq!(entities.len(), 1); // Should still be 1, no new entity added
}

#[test]
fn test_ensure_ground_near_player() {
    let mut ground_tiles = Vec::new();
    let mut debug_messages = Vec::new();

    // Test with no existing ground tiles near player
    let player_x = 500.0;
    let player_y = 700.0; // Below ground level

    systems::world::WorldSystem::ensure_ground_near_player(
        &mut ground_tiles,
        player_x,
        player_y,
        &mut debug_messages,
    );

    // This method is now deprecated and only adds debug messages
    // Ground tiles should remain empty as functionality moved to ExplorationSystem
    assert!(ground_tiles.is_empty());
    assert!(!debug_messages.is_empty());
    assert!(debug_messages[0].contains("LEGACY: WorldSystem::ensure_ground_near_player called"));

    // Test with player at horizon level - should also only add debug message
    let mut ground_tiles2 = Vec::new();
    let mut debug_messages2 = Vec::new();
    let player_y_horizon = 640.0; // At horizon line

    systems::world::WorldSystem::ensure_ground_near_player(
        &mut ground_tiles2,
        player_x,
        player_y_horizon,
        &mut debug_messages2,
    );

    // Should not spawn tiles - method is deprecated
    assert!(ground_tiles2.is_empty());
    assert!(!debug_messages2.is_empty());
    assert!(debug_messages2[0].contains("LEGACY: WorldSystem::ensure_ground_near_player called"));

    // Test with player far outside original world bounds but below horizon
    let mut ground_tiles3 = Vec::new();
    let mut debug_messages3 = Vec::new();
    let player_x_outside = -500.0; // Left of original world bounds
    let player_y_outside = 800.0; // Below horizon line

    systems::world::WorldSystem::ensure_ground_near_player(
        &mut ground_tiles3,
        player_x_outside,
        player_y_outside,
        &mut debug_messages3,
    );

    // Should not spawn tiles - method is deprecated
    assert!(ground_tiles3.is_empty());
    assert!(!debug_messages3.is_empty());
    assert!(debug_messages3[0].contains("LEGACY: WorldSystem::ensure_ground_near_player called"));

    // Test with existing ground coverage - should still only add debug message
    let initial_count = ground_tiles.len();
    systems::world::WorldSystem::ensure_ground_near_player(
        &mut ground_tiles,
        player_x,
        player_y,
        &mut debug_messages,
    );

    // Should not spawn any new tiles since method is deprecated
    let new_count = ground_tiles.len();
    assert_eq!(new_count, initial_count); // No new tiles should be added
}

#[test]
fn test_shelter_coordinates_after_fix() {
    // Test that the problematic shelter coordinates from the original code
    // would now be properly validated
    let test_coordinates = vec![
        (150.0, 400.0),  // Temple Ruins - above ground
        (650.0, 200.0),  // Castle Remains - above ground
        (450.0, 450.0),  // Shed - above ground
        (750.0, 350.0),  // Shed - above ground
        (300.0, 500.0),  // Dense Grove - above ground
        (1150.0, 200.0), // Tree Cover - above ground
        (600.0, 400.0),  // Highway Underpass - above ground
        (950.0, 550.0),  // Bridge Underpass - above ground
        (1100.0, 640.0), // Ruins - on ground (should be valid)
        (550.0, 650.0),  // Shed - on ground (should be valid)
    ];

    for (x, y) in test_coordinates {
        let has_ground = systems::shelter::ShelterSystem::has_ground_at_position(x, y);
        let expected_has_ground = y >= 640.0;

        assert_eq!(
            has_ground, expected_has_ground,
            "Ground validation failed for position ({}, {}). Expected: {}, Got: {}",
            x, y, expected_has_ground, has_ground
        );

        // With spatial grid and expanded world system, all positions should be relocatable
        assert!(
            systems::world::WorldSystem::is_relocatable_to_ground(x, y),
            "Position ({}, {}) should be relocatable in expanded world system",
            x,
            y
        );
    }
}

#[test]
fn test_world_initialization_with_ground_validation() {
    let mut entities = Vec::new();
    let mut clans = std::collections::HashMap::new();
    let mut stars = Vec::new();
    let mut moon = components::environment::Moon::new();
    let mut ground_tiles = Vec::new();
    let mut next_entity_id = 0;

    // Initialize the world
    let mut debug_messages = Vec::new();
    let player_id = systems::world::WorldSystem::initialize_world(
        &mut entities,
        &mut clans,
        &mut stars,
        &mut moon,
        &mut ground_tiles,
        &mut next_entity_id,
        &mut debug_messages,
    );

    // Check that ground tiles were created
    assert!(
        !ground_tiles.is_empty(),
        "Ground tiles should be initialized"
    );

    // Verify ground tiles are only at valid positions (y >= 640)
    for tile in &ground_tiles {
        assert!(
            tile.y >= 640.0,
            "Ground tile at ({}, {}) is above ground level",
            tile.x,
            tile.y
        );
    }

    // Check that some shelters were spawned (may be fewer than original due to skipping invalid positions)
    let shelter_count = entities
        .iter()
        .filter(|e| matches!(e.entity_type, components::game_data::EntityType::Shelter))
        .count();

    assert!(
        shelter_count > 0,
        "At least some shelters should be spawned during world initialization"
    );

    // Verify all spawned shelters are on valid ground
    for entity in &entities {
        if matches!(
            entity.entity_type,
            components::game_data::EntityType::Shelter
        ) {
            assert!(
                systems::shelter::ShelterSystem::has_ground_at_position(
                    entity.position.x,
                    entity.position.y
                ),
                "Shelter spawned at invalid ground position ({}, {})",
                entity.position.x,
                entity.position.y
            );
        }
    }
}
