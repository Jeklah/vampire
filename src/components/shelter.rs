//! Shelter components for protection from sunlight
//!
//! This module contains components for shelter structures that provide
//! protection from sunlight during daytime, essential for vampire survival.

use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

/// Different types of shelters available in the world
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ShelterType {
    /// Natural cave formation - high protection, permanent
    Cave,
    /// Abandoned building - medium protection, spacious
    Building,
    /// Dense tree cover - low protection, temporary
    TreeCover,
    /// Underground bunker - maximum protection, rare
    Underground,
    /// Ancient ruins - medium protection, atmospheric
    Ruins,
    /// Small shed - low-medium protection, common
    Shed,
    /// Bridge underpass - medium protection, urban
    BridgeUnderpass,
}

impl ShelterType {
    /// Get the base protection level for this shelter type (0.0 to 1.0)
    pub fn base_protection(&self) -> f32 {
        match self {
            ShelterType::Cave => 0.9,
            ShelterType::Building => 0.8,
            ShelterType::TreeCover => 0.4,
            ShelterType::Underground => 1.0,
            ShelterType::Ruins => 0.7,
            ShelterType::Shed => 0.6,
            ShelterType::BridgeUnderpass => 0.75,
        }
    }

    /// Get the maximum capacity for this shelter type
    pub fn max_capacity(&self) -> u32 {
        match self {
            ShelterType::Cave => 3,
            ShelterType::Building => 8,
            ShelterType::TreeCover => 2,
            ShelterType::Underground => 5,
            ShelterType::Ruins => 4,
            ShelterType::Shed => 2,
            ShelterType::BridgeUnderpass => 6,
        }
    }

    /// Get the discovery range (how close you need to be to discover/interact)
    pub fn discovery_range(&self) -> f32 {
        match self {
            ShelterType::Cave => 40.0,
            ShelterType::Building => 60.0,
            ShelterType::TreeCover => 35.0,
            ShelterType::Underground => 30.0,
            ShelterType::Ruins => 50.0,
            ShelterType::Shed => 45.0,
            ShelterType::BridgeUnderpass => 55.0,
        }
    }

    /// Get the visual size for rendering
    pub fn visual_size(&self) -> (f32, f32) {
        match self {
            ShelterType::Cave => (40.0, 30.0),
            ShelterType::Building => (80.0, 60.0),
            ShelterType::TreeCover => (50.0, 50.0),
            ShelterType::Underground => (25.0, 25.0),
            ShelterType::Ruins => (70.0, 45.0),
            ShelterType::Shed => (35.0, 25.0),
            ShelterType::BridgeUnderpass => (90.0, 40.0),
        }
    }

    /// Get the primary color for rendering this shelter type
    pub fn primary_color(&self) -> Color {
        match self {
            ShelterType::Cave => Color::new(0.4, 0.3, 0.2, 1.0), // Brown
            ShelterType::Building => Color::new(0.6, 0.6, 0.6, 1.0), // Gray
            ShelterType::TreeCover => Color::new(0.2, 0.5, 0.2, 1.0), // Dark green
            ShelterType::Underground => Color::new(0.3, 0.3, 0.3, 1.0), // Dark gray
            ShelterType::Ruins => Color::new(0.5, 0.4, 0.3, 1.0), // Tan
            ShelterType::Shed => Color::new(0.4, 0.2, 0.1, 1.0), // Dark brown
            ShelterType::BridgeUnderpass => Color::new(0.5, 0.5, 0.5, 1.0), // Medium gray
        }
    }

    /// Get the secondary color for details/highlights
    pub fn secondary_color(&self) -> Color {
        match self {
            ShelterType::Cave => Color::new(0.2, 0.1, 0.1, 1.0), // Dark brown
            ShelterType::Building => Color::new(0.3, 0.3, 0.4, 1.0), // Blue-gray
            ShelterType::TreeCover => Color::new(0.1, 0.3, 0.1, 1.0), // Darker green
            ShelterType::Underground => Color::new(0.1, 0.1, 0.1, 1.0), // Black
            ShelterType::Ruins => Color::new(0.7, 0.6, 0.4, 1.0), // Light tan
            ShelterType::Shed => Color::new(0.6, 0.3, 0.1, 1.0), // Orange-brown
            ShelterType::BridgeUnderpass => Color::new(0.7, 0.7, 0.7, 1.0), // Light gray
        }
    }

    /// Get the name as a display string
    pub fn display_name(&self) -> &'static str {
        match self {
            ShelterType::Cave => "Cave",
            ShelterType::Building => "Building",
            ShelterType::TreeCover => "Tree Cover",
            ShelterType::Underground => "Underground Bunker",
            ShelterType::Ruins => "Ancient Ruins",
            ShelterType::Shed => "Shed",
            ShelterType::BridgeUnderpass => "Bridge Underpass",
        }
    }
}

/// Shelter condition affecting protection effectiveness
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ShelterCondition {
    /// Perfect condition - full protection
    Pristine,
    /// Good condition - minor protection loss
    Good,
    /// Damaged condition - reduced protection
    Damaged,
    /// Poor condition - significantly reduced protection
    Poor,
    /// Collapsed/destroyed - no protection
    Ruined,
}

impl ShelterCondition {
    /// Get the protection multiplier for this condition (0.0 to 1.0)
    pub fn protection_multiplier(&self) -> f32 {
        match self {
            ShelterCondition::Pristine => 1.0,
            ShelterCondition::Good => 0.9,
            ShelterCondition::Damaged => 0.7,
            ShelterCondition::Poor => 0.4,
            ShelterCondition::Ruined => 0.0,
        }
    }

    /// Get the display color for this condition
    pub fn status_color(&self) -> Color {
        match self {
            ShelterCondition::Pristine => GREEN,
            ShelterCondition::Good => YELLOW,
            ShelterCondition::Damaged => ORANGE,
            ShelterCondition::Poor => RED,
            ShelterCondition::Ruined => DARKGRAY,
        }
    }
}

/// Main shelter component
#[derive(Debug, Clone)]
pub struct Shelter {
    /// Type of shelter
    pub shelter_type: ShelterType,
    /// Current condition of the shelter
    pub condition: ShelterCondition,
    /// Whether this shelter has been discovered by the player
    pub discovered: bool,
    /// Whether the shelter is currently occupied
    pub occupied: bool,
    /// List of entity IDs currently taking shelter
    pub occupants: Vec<u32>,
    /// Optional name for named/special shelters
    pub name: Option<String>,
    /// Whether this shelter can be entered (some are just visual/partial cover)
    pub enterable: bool,
    /// Time when shelter was last used (for cooldowns/degradation)
    pub last_used: f32,
}

impl Shelter {
    /// Create a new shelter of the specified type
    pub fn new(shelter_type: ShelterType) -> Self {
        Self {
            shelter_type,
            condition: ShelterCondition::Good, // Most shelters start in good condition
            discovered: false,
            occupied: false,
            occupants: Vec::new(),
            name: None,
            enterable: true,
            last_used: 0.0,
        }
    }

    /// Create a shelter with custom condition
    pub fn with_condition(shelter_type: ShelterType, condition: ShelterCondition) -> Self {
        Self {
            condition,
            ..Self::new(shelter_type)
        }
    }

    /// Create a named shelter
    pub fn with_name(shelter_type: ShelterType, name: String) -> Self {
        Self {
            name: Some(name),
            ..Self::new(shelter_type)
        }
    }

    /// Calculate the effective protection this shelter provides
    pub fn effective_protection(&self) -> f32 {
        self.shelter_type.base_protection() * self.condition.protection_multiplier()
    }

    /// Check if this shelter can accommodate another occupant
    pub fn can_accommodate(&self) -> bool {
        self.enterable
            && !matches!(self.condition, ShelterCondition::Ruined)
            && self.occupants.len() < self.shelter_type.max_capacity() as usize
    }

    /// Add an occupant to this shelter
    pub fn add_occupant(&mut self, entity_id: u32) -> bool {
        if self.can_accommodate() && !self.occupants.contains(&entity_id) {
            self.occupants.push(entity_id);
            self.occupied = !self.occupants.is_empty();
            true
        } else {
            false
        }
    }

    /// Remove an occupant from this shelter
    pub fn remove_occupant(&mut self, entity_id: u32) -> bool {
        if let Some(index) = self.occupants.iter().position(|&id| id == entity_id) {
            self.occupants.remove(index);
            self.occupied = !self.occupants.is_empty();
            true
        } else {
            false
        }
    }

    /// Check if a specific entity is occupying this shelter
    pub fn is_occupied_by(&self, entity_id: u32) -> bool {
        self.occupants.contains(&entity_id)
    }

    /// Get the number of current occupants
    pub fn occupant_count(&self) -> usize {
        self.occupants.len()
    }

    /// Get remaining capacity
    pub fn remaining_capacity(&self) -> u32 {
        self.shelter_type
            .max_capacity()
            .saturating_sub(self.occupants.len() as u32)
    }

    /// Mark this shelter as discovered
    pub fn discover(&mut self) {
        self.discovered = true;
    }

    /// Update shelter (for degradation, repairs, etc.)
    pub fn update(&mut self, delta_time: f32, current_time: f32) {
        // Shelters might degrade over time or improve based on usage patterns
        // This is a placeholder for future shelter maintenance mechanics

        // For now, just track usage time
        if self.occupied {
            self.last_used = current_time;
        }
    }

    /// Get status text for UI display
    pub fn get_status_text(&self) -> String {
        let protection_pct = (self.effective_protection() * 100.0) as u32;
        let occupancy = format!(
            "{}/{}",
            self.occupants.len(),
            self.shelter_type.max_capacity()
        );

        match &self.name {
            Some(name) => format!(
                "{} ({}): {}% protection, {} occupants",
                name,
                self.shelter_type.display_name(),
                protection_pct,
                occupancy
            ),
            None => format!(
                "{}: {}% protection, {} occupants",
                self.shelter_type.display_name(),
                protection_pct,
                occupancy
            ),
        }
    }

    /// Check if this shelter provides adequate protection against given sunlight intensity
    pub fn provides_adequate_protection(&self, sunlight_intensity: f32) -> bool {
        self.effective_protection() >= sunlight_intensity * 0.8 // Need 80% of sunlight intensity in protection
    }
}

/// Component to track shelter occupancy status for entities
#[derive(Debug, Clone)]
pub struct ShelterOccupancy {
    /// ID of the shelter entity this entity is occupying (None if not in shelter)
    pub shelter_id: Option<u32>,
    /// Time when entity entered current shelter
    pub entered_at: f32,
    /// Whether entity is actively seeking shelter
    pub seeking_shelter: bool,
    /// Last time entity attempted to find shelter
    pub last_shelter_search: f32,
}

impl ShelterOccupancy {
    /// Create a new shelter occupancy component
    pub fn new() -> Self {
        Self {
            shelter_id: None,
            entered_at: 0.0,
            seeking_shelter: false,
            last_shelter_search: 0.0,
        }
    }

    /// Check if entity is currently in a shelter
    pub fn is_in_shelter(&self) -> bool {
        self.shelter_id.is_some()
    }

    /// Enter a shelter
    pub fn enter_shelter(&mut self, shelter_id: u32, current_time: f32) {
        self.shelter_id = Some(shelter_id);
        self.entered_at = current_time;
        self.seeking_shelter = false;
    }

    /// Leave current shelter
    pub fn leave_shelter(&mut self) {
        self.shelter_id = None;
        self.entered_at = 0.0;
    }

    /// Start seeking shelter
    pub fn start_seeking(&mut self, current_time: f32) {
        if !self.is_in_shelter() {
            self.seeking_shelter = true;
            self.last_shelter_search = current_time;
        }
    }

    /// Stop seeking shelter
    pub fn stop_seeking(&mut self) {
        self.seeking_shelter = false;
    }

    /// Get time spent in current shelter
    pub fn time_in_shelter(&self, current_time: f32) -> f32 {
        if self.is_in_shelter() {
            current_time - self.entered_at
        } else {
            0.0
        }
    }
}

impl Default for ShelterOccupancy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shelter_creation() {
        let shelter = Shelter::new(ShelterType::Cave);
        assert_eq!(shelter.shelter_type, ShelterType::Cave);
        assert!(!shelter.discovered);
        assert!(!shelter.occupied);
        assert!(shelter.enterable);
    }

    #[test]
    fn test_shelter_protection() {
        let cave = Shelter::new(ShelterType::Cave);
        let tree_cover = Shelter::new(ShelterType::TreeCover);

        assert!(cave.effective_protection() > tree_cover.effective_protection());
        assert!(cave.provides_adequate_protection(0.8));
        assert!(!tree_cover.provides_adequate_protection(0.8));
    }

    #[test]
    fn test_shelter_occupancy() {
        let mut shelter = Shelter::new(ShelterType::Shed);

        assert!(shelter.can_accommodate());
        assert!(shelter.add_occupant(1));
        assert!(shelter.add_occupant(2));
        assert!(!shelter.add_occupant(3)); // Shed max capacity is 2

        assert!(shelter.remove_occupant(1));
        assert!(shelter.can_accommodate());
    }

    #[test]
    fn test_shelter_condition_effects() {
        let mut shelter = Shelter::new(ShelterType::Cave);
        let base_protection = shelter.effective_protection();

        shelter.condition = ShelterCondition::Damaged;
        assert!(shelter.effective_protection() < base_protection);

        shelter.condition = ShelterCondition::Ruined;
        assert!(!shelter.can_accommodate());
        assert_eq!(shelter.effective_protection(), 0.0);
    }

    #[test]
    fn test_occupancy_component() {
        let mut occupancy = ShelterOccupancy::new();

        assert!(!occupancy.is_in_shelter());

        occupancy.enter_shelter(5, 100.0);
        assert!(occupancy.is_in_shelter());
        assert_eq!(occupancy.shelter_id, Some(5));

        occupancy.leave_shelter();
        assert!(!occupancy.is_in_shelter());
    }
}
