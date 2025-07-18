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
    assert!(systems::shelter::ShelterSystem::has_ground_at_position(
        500.0, 700.0
    ));
    assert!(systems::shelter::ShelterSystem::has_ground_at_position(
        1000.0, 1000.0
    ));
    assert!(systems::shelter::ShelterSystem::has_ground_at_position(
        1500.0, 1200.0
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

    // Test positions outside world bounds
    assert!(!systems::shelter::ShelterSystem::has_ground_at_position(
        -100.0, 700.0
    ));
    assert!(!systems::shelter::ShelterSystem::has_ground_at_position(
        2000.0, 700.0
    ));
    assert!(!systems::shelter::ShelterSystem::has_ground_at_position(
        500.0, 1300.0
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
    // Test random ground position generation
    for _ in 0..10 {
        let (x, y) = systems::world::WorldSystem::generate_random_ground_position();
        assert!(x >= 64.0); // Should be within padded bounds
        assert!(x <= 1536.0); // Should be within padded bounds
        assert!(y >= 704.0); // Should be at ground level + padding (640 + 64)
        assert!(y <= 1136.0); // Should be within world bounds - padding
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

    // Test positions that should not be relocatable (too far from ground)
    assert!(!systems::world::WorldSystem::is_relocatable_to_ground(
        500.0, 400.0
    ));
    assert!(!systems::world::WorldSystem::is_relocatable_to_ground(
        800.0, 200.0
    ));
    assert!(!systems::world::WorldSystem::is_relocatable_to_ground(
        200.0, 100.0
    ));

    // Test positions outside world bounds
    assert!(!systems::world::WorldSystem::is_relocatable_to_ground(
        -100.0, 550.0
    ));
    assert!(!systems::world::WorldSystem::is_relocatable_to_ground(
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

        // Test relocation logic
        if systems::world::WorldSystem::is_relocatable_to_ground(x, y) {
            // Should be considered relocatable if close to ground
            assert!(
                y >= 540.0 && y < 640.0,
                "Position ({}, {}) should be relocatable but isn't in expected range",
                x,
                y
            );
        } else if y < 540.0 {
            // Should not be relocatable if too far from ground
            assert!(
                !systems::world::WorldSystem::is_relocatable_to_ground(x, y),
                "Position ({}, {}) should not be relocatable but is",
                x,
                y
            );
        }
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
    let player_id = systems::world::WorldSystem::initialize_world(
        &mut entities,
        &mut clans,
        &mut stars,
        &mut moon,
        &mut ground_tiles,
        &mut next_entity_id,
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
