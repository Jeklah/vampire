//! Integration test for the spawning system
//!
//! This test verifies that the spawning system correctly maintains entity populations
//! when integrated with the main game state and cleanup systems.

use vampire_rpg::components::*;
use vampire_rpg::systems::*;
use vampire_rpg::GameState;

#[test]
fn test_spawning_system_integration() {
    let mut game_state = GameState::new();

    // Get initial entity counts
    let initial_hostile_count = count_hostiles(&game_state.entities);
    let initial_animal_count = count_animals(&game_state.entities);

    // Simulate some time passing and entities being cleaned up
    // First, force cleanup to remove some entities
    game_state.force_cleanup();

    // Advance time significantly to trigger spawning
    for _ in 0..10 {
        // Simulate 1 second per iteration
        game_state.spawning_system.update(
            &mut game_state.entities,
            &mut game_state.next_entity_id,
            game_state.player_id,
            game_state.game_time + 10.0,
            game_state.max_entities,
        );
        game_state.game_time += 1.0;
    }

    // Check that entities have been respawned
    let final_hostile_count = count_hostiles(&game_state.entities);
    let final_animal_count = count_animals(&game_state.entities);

    // There should be some entities (though exact numbers may vary due to positioning)
    assert!(final_hostile_count > 0, "No hostile entities spawned");
    assert!(final_animal_count > 0, "No animals spawned");

    // Get spawn statistics
    let stats = game_state.spawning_system.get_stats();
    assert!(
        stats.total_spawned > 0,
        "No entities were spawned according to stats"
    );

    println!("Initial hostile count: {}", initial_hostile_count);
    println!("Final hostile count: {}", final_hostile_count);
    println!("Initial animal count: {}", initial_animal_count);
    println!("Final animal count: {}", final_animal_count);
    println!("Total spawned: {}", stats.total_spawned);
    println!("Failed spawns: {}", stats.failed_spawns);
}

#[test]
fn test_performance_mode_spawning() {
    let mut game_state = GameState::new();

    // Enable performance mode
    game_state.set_performance_mode(true);

    // Get spawn configurations
    let hostile_config = game_state.spawning_system.get_config("hostile").unwrap();
    let animal_config = game_state.spawning_system.get_config("animal").unwrap();

    // Performance mode should reduce max counts and increase intervals
    assert!(
        hostile_config.max_count <= 12,
        "Performance mode should reduce hostile max count"
    );
    assert!(
        animal_config.max_count <= 15,
        "Performance mode should reduce animal max count"
    );

    // Disable performance mode
    game_state.set_performance_mode(false);

    println!("Performance mode spawning test passed");
}

#[test]
fn test_spawn_timing_and_limits() {
    let mut game_state = GameState::new();

    // Clear existing entities except player
    game_state.entities.retain(|e| e.id == game_state.player_id);

    // Test that spawning respects timing intervals
    let initial_time = 0.0;

    // Update with no time passed - should not spawn
    game_state.spawning_system.update(
        &mut game_state.entities,
        &mut game_state.next_entity_id,
        game_state.player_id,
        initial_time,
        game_state.max_entities,
    );

    let count_after_no_time = count_total_entities(&game_state.entities) - 1; // -1 for player

    // Update with sufficient time passed - should spawn
    game_state.spawning_system.update(
        &mut game_state.entities,
        &mut game_state.next_entity_id,
        game_state.player_id,
        initial_time + 20.0, // 20 seconds later
        game_state.max_entities,
    );

    let count_after_time = count_total_entities(&game_state.entities) - 1; // -1 for player

    assert!(
        count_after_time > count_after_no_time,
        "Entities should spawn after time interval"
    );

    println!("Spawn timing test passed");
}

// Helper functions
fn count_hostiles(entities: &[GameEntity]) -> usize {
    entities
        .iter()
        .filter(|e| {
            matches!(e.entity_type, EntityType::HostileInfected)
                && !matches!(e.ai_state, AIState::Dead)
        })
        .count()
}

fn count_animals(entities: &[GameEntity]) -> usize {
    entities
        .iter()
        .filter(|e| {
            matches!(e.entity_type, EntityType::Animal) && !matches!(e.ai_state, AIState::Dead)
        })
        .count()
}

fn count_total_entities(entities: &[GameEntity]) -> usize {
    entities
        .iter()
        .filter(|e| !matches!(e.ai_state, AIState::Dead))
        .count()
}
