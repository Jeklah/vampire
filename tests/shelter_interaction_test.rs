//! Integration tests for shelter interaction system
//!
//! These tests verify that the F key input properly triggers shelter interactions
//! and that the shelter system works correctly with the input handler.

use macroquad::prelude::*;
use vampire_rpg::components::*;
use vampire_rpg::input::InputHandler;
use vampire_rpg::systems::shelter::ShelterSystem;

#[test]
fn test_input_handler_recognizes_f_key() {
    // This test verifies that the InputHandler can detect F key presses
    // Note: This is a unit test since we can't simulate actual key presses in integration tests

    let input_handler = InputHandler::new();

    // The input handler should be created successfully
    assert!(!input_handler.is_key_pressed(KeyCode::F));
    assert!(!input_handler.is_key_just_pressed(KeyCode::F));

    // This test mainly ensures the InputHandler compiles with F key support
    // Actual key simulation would require a more complex setup with macroquad
}

#[test]
fn test_shelter_interaction_with_no_shelters() {
    // Test the shelter interaction when no shelters are present
    let mut entities = Vec::new();

    // Create a player entity
    let player_id = 1;
    let player = GameEntity {
        id: player_id,
        position: Position { x: 100.0, y: 100.0 },
        velocity: None,
        entity_type: game_data::EntityType::Player,
        health: Some(Health::new(100.0)),
        combat_stats: None,
        ai_state: combat::AIState::Idle,
        blood_meter: Some(vampire::BloodMeter::new(100.0)),
        vampire_abilities: None,
        shelter: None,
        shelter_occupancy: Some(shelter::ShelterOccupancy::new()),
        color: RED,
    };
    entities.push(player);

    let result = ShelterSystem::handle_player_shelter_interaction(&mut entities, player_id, 0.0);

    // Should return a message indicating no shelters found
    assert!(result.is_some());
    let message = result.unwrap();
    assert!(message.contains("No shelters found in the world"));
}

#[test]
fn test_shelter_interaction_with_nearby_shelter() {
    let mut entities = Vec::new();

    // Create a player entity
    let player_id = 1;
    let player = GameEntity {
        id: player_id,
        position: Position { x: 100.0, y: 100.0 },
        velocity: None,
        entity_type: game_data::EntityType::Player,
        health: Some(Health::new(100.0)),
        combat_stats: None,
        ai_state: combat::AIState::Idle,
        blood_meter: Some(vampire::BloodMeter::new(100.0)),
        vampire_abilities: None,
        shelter: None,
        shelter_occupancy: Some(shelter::ShelterOccupancy::new()),
        color: RED,
    };
    entities.push(player);

    // Create a shelter entity nearby (within discovery range)
    let shelter_id = 2;
    let shelter_entity = GameEntity {
        id: shelter_id,
        position: Position { x: 110.0, y: 110.0 }, // 14.14 units away from player
        velocity: None,
        entity_type: game_data::EntityType::Shelter,
        health: None,
        combat_stats: None,
        ai_state: combat::AIState::Idle,
        blood_meter: None,
        vampire_abilities: None,
        shelter: Some(shelter::Shelter::new(shelter::ShelterType::Cave)), // Cave has 40.0 discovery range
        shelter_occupancy: None,
        color: BROWN,
    };
    entities.push(shelter_entity);

    let result = ShelterSystem::handle_player_shelter_interaction(&mut entities, player_id, 100.0);

    // Should successfully enter the shelter
    assert!(result.is_some());
    let message = result.unwrap();
    assert!(message.contains("Entered"));
    assert!(message.contains("Cave"));

    // Verify player's shelter occupancy was updated
    let player = entities.iter().find(|e| e.id == player_id).unwrap();
    assert!(player.shelter_occupancy.as_ref().unwrap().is_in_shelter());
    assert_eq!(
        player.shelter_occupancy.as_ref().unwrap().shelter_id,
        Some(shelter_id)
    );

    // Verify shelter's occupant list was updated
    let shelter_entity = entities.iter().find(|e| e.id == shelter_id).unwrap();
    assert!(shelter_entity
        .shelter
        .as_ref()
        .unwrap()
        .is_occupied_by(player_id));
}

#[test]
fn test_shelter_exit_interaction() {
    let mut entities = Vec::new();

    // Create a player entity already in a shelter
    let player_id = 1;
    let shelter_id = 2;

    let mut player_occupancy = shelter::ShelterOccupancy::new();
    player_occupancy.enter_shelter(shelter_id, 50.0);

    let player = GameEntity {
        id: player_id,
        position: Position { x: 100.0, y: 100.0 },
        velocity: None,
        entity_type: game_data::EntityType::Player,
        health: Some(Health::new(100.0)),
        combat_stats: None,
        ai_state: combat::AIState::Idle,
        blood_meter: Some(vampire::BloodMeter::new(100.0)),
        vampire_abilities: None,
        shelter: None,
        shelter_occupancy: Some(player_occupancy),
        color: RED,
    };
    entities.push(player);

    // Create the shelter entity with player already as occupant
    let mut shelter = shelter::Shelter::new(shelter::ShelterType::Building);
    shelter.add_occupant(player_id);

    let shelter_entity = GameEntity {
        id: shelter_id,
        position: Position { x: 110.0, y: 110.0 },
        velocity: None,
        entity_type: game_data::EntityType::Shelter,
        health: None,
        combat_stats: None,
        ai_state: combat::AIState::Idle,
        blood_meter: None,
        vampire_abilities: None,
        shelter: Some(shelter),
        shelter_occupancy: None,
        color: GRAY,
    };
    entities.push(shelter_entity);

    let result = ShelterSystem::handle_player_shelter_interaction(&mut entities, player_id, 100.0);

    // Should successfully exit the shelter
    assert!(result.is_some());
    let message = result.unwrap();
    assert_eq!(message, "Exited shelter");

    // Verify player's shelter occupancy was cleared
    let player = entities.iter().find(|e| e.id == player_id).unwrap();
    assert!(!player.shelter_occupancy.as_ref().unwrap().is_in_shelter());

    // Verify shelter's occupant list was updated
    let shelter_entity = entities.iter().find(|e| e.id == shelter_id).unwrap();
    assert!(!shelter_entity
        .shelter
        .as_ref()
        .unwrap()
        .is_occupied_by(player_id));
}

#[test]
fn test_shelter_interaction_too_far() {
    let mut entities = Vec::new();

    // Create a player entity
    let player_id = 1;
    let player = GameEntity {
        id: player_id,
        position: Position { x: 100.0, y: 100.0 },
        velocity: None,
        entity_type: game_data::EntityType::Player,
        health: Some(Health::new(100.0)),
        combat_stats: None,
        ai_state: combat::AIState::Idle,
        blood_meter: Some(vampire::BloodMeter::new(100.0)),
        vampire_abilities: None,
        shelter: None,
        shelter_occupancy: Some(shelter::ShelterOccupancy::new()),
        color: RED,
    };
    entities.push(player);

    // Create a shelter entity too far away (outside discovery range)
    let shelter_id = 2;
    let shelter_entity = GameEntity {
        id: shelter_id,
        position: Position { x: 200.0, y: 200.0 }, // 141.42 units away from player
        velocity: None,
        entity_type: game_data::EntityType::Shelter,
        health: None,
        combat_stats: None,
        ai_state: combat::AIState::Idle,
        blood_meter: None,
        vampire_abilities: None,
        shelter: Some(shelter::Shelter::new(shelter::ShelterType::Cave)), // Cave has 40.0 discovery range
        shelter_occupancy: None,
        color: BROWN,
    };
    entities.push(shelter_entity);

    let result = ShelterSystem::handle_player_shelter_interaction(&mut entities, player_id, 100.0);

    // Should indicate no shelters nearby
    assert!(result.is_some());
    let message = result.unwrap();
    assert!(message.contains("No shelters nearby"));
    assert!(message.contains("found 1 shelters in world"));
}

#[test]
fn test_shelter_full_capacity() {
    let mut entities = Vec::new();

    // Create a player entity
    let player_id = 1;
    let player = GameEntity {
        id: player_id,
        position: Position { x: 100.0, y: 100.0 },
        velocity: None,
        entity_type: game_data::EntityType::Player,
        health: Some(Health::new(100.0)),
        combat_stats: None,
        ai_state: combat::AIState::Idle,
        blood_meter: Some(vampire::BloodMeter::new(100.0)),
        vampire_abilities: None,
        shelter: None,
        shelter_occupancy: Some(shelter::ShelterOccupancy::new()),
        color: RED,
    };
    entities.push(player);

    // Create a shed (capacity 2) that's already full
    let shelter_id = 2;
    let mut shelter = shelter::Shelter::new(shelter::ShelterType::Shed);
    shelter.add_occupant(10); // First occupant
    shelter.add_occupant(11); // Second occupant (shed is now full)

    let shelter_entity = GameEntity {
        id: shelter_id,
        position: Position { x: 110.0, y: 110.0 },
        velocity: None,
        entity_type: game_data::EntityType::Shelter,
        health: None,
        combat_stats: None,
        ai_state: combat::AIState::Idle,
        blood_meter: None,
        vampire_abilities: None,
        shelter: Some(shelter),
        shelter_occupancy: None,
        color: BROWN,
    };
    entities.push(shelter_entity);

    let result = ShelterSystem::handle_player_shelter_interaction(&mut entities, player_id, 100.0);

    // Should indicate shelter is full
    assert!(result.is_some());
    let message = result.unwrap();
    assert!(message.contains("Shelter cannot be entered"));
    assert!(message.contains("2/2 occupants"));
}
