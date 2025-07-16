//! Shelter System Usage Example
//!
//! This example demonstrates how to use the shelter system components
//! and functions in the Vampire RPG.

use vampire_rpg::*;

fn main() {
    println!("=== Vampire RPG Shelter System Example ===\n");

    // Example 1: Creating different shelter types
    example_shelter_creation();

    // Example 2: Shelter protection calculations
    example_protection_calculations();

    // Example 3: Occupancy management
    example_occupancy_management();

    // Example 4: Shelter discovery and interaction
    example_shelter_interaction();

    // Example 5: NPC shelter seeking behavior
    example_npc_shelter_seeking();
}

fn example_shelter_creation() {
    println!("1. Creating Different Shelter Types");
    println!("==================================");

    // Create various shelter types
    let cave = Shelter::new(ShelterType::Cave);
    let building = Shelter::with_condition(ShelterType::Building, ShelterCondition::Damaged);
    let bunker = Shelter::with_name(
        ShelterType::Underground,
        "Emergency Bunker Alpha".to_string(),
    );

    println!(
        "Cave: {} protection, {} capacity",
        cave.effective_protection(),
        cave.shelter_type.max_capacity()
    );

    println!(
        "Damaged Building: {} protection, {} capacity",
        building.effective_protection(),
        building.shelter_type.max_capacity()
    );

    println!(
        "Named Bunker: {} protection, {} capacity",
        bunker.effective_protection(),
        bunker.shelter_type.max_capacity()
    );

    println!(
        "Cave provides adequate protection at 80% sunlight: {}",
        cave.provides_adequate_protection(0.8)
    );
    println!();
}

fn example_protection_calculations() {
    println!("2. Protection Calculations");
    println!("=========================");

    let shelters = vec![
        (
            "Cave (Good)",
            Shelter::with_condition(ShelterType::Cave, ShelterCondition::Good),
        ),
        (
            "Building (Pristine)",
            Shelter::with_condition(ShelterType::Building, ShelterCondition::Pristine),
        ),
        (
            "Tree Cover (Good)",
            Shelter::with_condition(ShelterType::TreeCover, ShelterCondition::Good),
        ),
        (
            "Shed (Damaged)",
            Shelter::with_condition(ShelterType::Shed, ShelterCondition::Damaged),
        ),
        (
            "Ruins (Poor)",
            Shelter::with_condition(ShelterType::Ruins, ShelterCondition::Poor),
        ),
    ];

    println!("Sunlight Protection Analysis:");
    println!("Shelter Type         | Protection | Safe at Noon?");
    println!("---------------------|------------|---------------");

    for (name, shelter) in &shelters {
        let protection = shelter.effective_protection();
        let safe_at_noon = shelter.provides_adequate_protection(1.0); // 100% sunlight intensity
        println!(
            "{:20} | {:8.1}% | {}",
            name,
            protection * 100.0,
            if safe_at_noon { "Yes" } else { "No" }
        );
    }
    println!();
}

fn example_occupancy_management() {
    println!("3. Occupancy Management");
    println!("======================");

    let mut shed = Shelter::new(ShelterType::Shed); // Max capacity: 2

    println!(
        "Initial capacity: {}/{}",
        shed.occupant_count(),
        shed.shelter_type.max_capacity()
    );
    println!("Can accommodate? {}", shed.can_accommodate());

    // Add occupants
    let player_id = 1;
    let npc_id = 2;
    let overflow_id = 3;

    println!("\nAdding player (ID: {})...", player_id);
    let success = shed.add_occupant(player_id);
    println!(
        "Success: {}, Capacity: {}/{}",
        success,
        shed.occupant_count(),
        shed.shelter_type.max_capacity()
    );

    println!("\nAdding NPC (ID: {})...", npc_id);
    let success = shed.add_occupant(npc_id);
    println!(
        "Success: {}, Capacity: {}/{}",
        success,
        shed.occupant_count(),
        shed.shelter_type.max_capacity()
    );

    println!("\nAttempting to add third entity (ID: {})...", overflow_id);
    let success = shed.add_occupant(overflow_id);
    println!("Success: {} (should be false - capacity full)", success);

    println!("\nRemoving player...");
    shed.remove_occupant(player_id);
    println!(
        "Capacity after removal: {}/{}",
        shed.occupant_count(),
        shed.shelter_type.max_capacity()
    );
    println!();
}

fn example_shelter_interaction() {
    println!("4. Shelter Discovery and Interaction");
    println!("===================================");

    // Simulate shelter discovery ranges
    let shelters = vec![
        ("Cave", ShelterType::Cave),
        ("Building", ShelterType::Building),
        ("Tree Cover", ShelterType::TreeCover),
        ("Underground", ShelterType::Underground),
    ];

    println!("Discovery Ranges:");
    for (name, shelter_type) in shelters {
        println!("{}: {}m", name, shelter_type.discovery_range());
    }

    // Simulate occupancy component usage
    let mut occupancy = ShelterOccupancy::new();
    println!("\nOccupancy Status:");
    println!("Initially in shelter: {}", occupancy.is_in_shelter());

    let shelter_id = 5;
    let current_time = 100.0;
    occupancy.enter_shelter(shelter_id, current_time);
    println!(
        "After entering shelter {}: {}",
        shelter_id,
        occupancy.is_in_shelter()
    );

    let time_in_shelter = occupancy.time_in_shelter(current_time + 50.0);
    println!("Time spent in shelter: {:.1} seconds", time_in_shelter);

    occupancy.leave_shelter();
    println!("After leaving: {}", occupancy.is_in_shelter());
    println!();
}

fn example_npc_shelter_seeking() {
    println!("5. NPC Shelter Seeking Behavior");
    println!("==============================");

    // Simulate shelter seeking logic
    let mut entities = Vec::new();
    let mut next_id = 0;

    // Create a test shelter (Cave)
    let shelter_id =
        spawn_test_shelter(&mut entities, &mut next_id, ShelterType::Cave, 100.0, 100.0);

    // Create a vampire NPC that needs shelter
    let npc_id = spawn_test_vampire(&mut entities, &mut next_id, 150.0, 120.0);

    println!("Created shelter at (100, 100) with ID: {}", shelter_id);
    println!("Created vampire NPC at (150, 120) with ID: {}", npc_id);

    // Simulate shelter seeking
    let player_pos = Position { x: 150.0, y: 120.0 };
    if let Some(nearest_shelter) = find_nearest_shelter(&entities, player_pos, 200.0) {
        println!("Nearest shelter found: ID {}", nearest_shelter);

        let distance = ((150.0_f32 - 100.0_f32).powi(2) + (120.0_f32 - 100.0_f32).powi(2)).sqrt();
        println!("Distance to shelter: {:.1}m", distance);

        let cave_discovery_range = ShelterType::Cave.discovery_range();
        println!("Cave discovery range: {}m", cave_discovery_range);
        println!(
            "Within discovery range: {}",
            distance <= cave_discovery_range
        );
    }

    println!("\nShelter seeking simulation complete!");
    println!();
}

// Helper functions for examples
fn spawn_test_shelter(
    entities: &mut Vec<GameEntity>,
    next_id: &mut u32,
    shelter_type: ShelterType,
    x: f32,
    y: f32,
) -> u32 {
    let id = *next_id;
    *next_id += 1;

    let shelter = Shelter::new(shelter_type);
    let entity = GameEntity {
        id,
        position: Position { x, y },
        velocity: Some(Velocity { x: 0.0, y: 0.0 }),
        entity_type: EntityType::Shelter,
        health: None,
        combat_stats: None,
        ai_state: AIState::Idle,
        blood_meter: None,
        vampire_abilities: None,
        shelter: Some(shelter),
        shelter_occupancy: None,
        color: WHITE,
    };

    entities.push(entity);
    id
}

fn spawn_test_vampire(entities: &mut Vec<GameEntity>, next_id: &mut u32, x: f32, y: f32) -> u32 {
    let id = *next_id;
    *next_id += 1;

    let entity = GameEntity {
        id,
        position: Position { x, y },
        velocity: Some(Velocity { x: 0.0, y: 0.0 }),
        entity_type: EntityType::ClanMember("Test Clan".to_string()),
        health: Some(Health {
            current: 100.0,
            max: 100.0,
        }),
        combat_stats: Some(CombatStats::new(15.0, 5.0)),
        ai_state: AIState::Idle,
        blood_meter: Some(BloodMeter {
            current: 50.0,
            maximum: 100.0,
            drain_rate: 1.0,
        }),
        vampire_abilities: Some(VampireAbilities {
            strength: 1.0,
            speed: 1.0,
            blood_sense: 0.0,
            shadow_movement: 0.0,
        }),
        shelter: None,
        shelter_occupancy: Some(ShelterOccupancy::new()),
        color: PURPLE,
    };

    entities.push(entity);
    id
}

fn find_nearest_shelter(entities: &[GameEntity], pos: Position, max_distance: f32) -> Option<u32> {
    let mut nearest: Option<(u32, f32)> = None;

    for entity in entities {
        if let Some(_shelter) = &entity.shelter {
            let distance =
                ((pos.x - entity.position.x).powi(2) + (pos.y - entity.position.y).powi(2)).sqrt();

            if distance <= max_distance {
                if nearest.is_none() || distance < nearest.as_ref().unwrap().1 {
                    nearest = Some((entity.id, distance));
                }
            }
        }
    }

    nearest.map(|(id, _)| id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_shelter_creation() {
        let cave = Shelter::new(ShelterType::Cave);
        assert_eq!(cave.shelter_type, ShelterType::Cave);
        assert!(cave.effective_protection() > 0.8);
    }

    #[test]
    fn test_example_protection() {
        let pristine_building =
            Shelter::with_condition(ShelterType::Building, ShelterCondition::Pristine);
        let damaged_building =
            Shelter::with_condition(ShelterType::Building, ShelterCondition::Damaged);

        assert!(pristine_building.effective_protection() > damaged_building.effective_protection());
    }

    #[test]
    fn test_example_occupancy() {
        let mut shed = Shelter::new(ShelterType::Shed);
        assert!(shed.add_occupant(1));
        assert!(shed.add_occupant(2));
        assert!(!shed.add_occupant(3)); // Should fail - capacity is 2
    }
}
