//! Systems Module
//!
//! This module contains all game systems that handle specific aspects of gameplay.
//! Each system is responsible for a focused area of game logic and operates on
//! game state data in a functional manner.

pub mod ai;
pub mod blood;
pub mod objectives;
pub mod player;
pub mod time;
pub mod world;

// Re-export systems for easier access
pub use ai::AISystem;
pub use blood::BloodSystem;
pub use objectives::ObjectivesSystem;
pub use player::PlayerSystem;
pub use time::TimeSystem;
pub use world::WorldSystem;

// Re-export common types used by systems
pub use blood::{ActivityLevel, BloodStatus, SurvivalScore};
pub use objectives::ObjectiveProgress;
pub use player::{ExperienceType, PlayerAction, PlayerStatus};

/// System update order for consistent game logic
pub enum SystemUpdateOrder {
    Input = 0,
    Player = 1,
    AI = 2,
    Blood = 3,
    Time = 4,
    Objectives = 5,
}

/// Trait for systems that need regular updates
pub trait System {
    /// Update the system with the given delta time
    fn update(&mut self, delta_time: f32);

    /// Get the system's name for debugging
    fn name(&self) -> &'static str;

    /// Check if the system is enabled
    fn is_enabled(&self) -> bool {
        true
    }
}
