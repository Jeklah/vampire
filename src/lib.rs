//! Vampire RPG - A 2D vampire survival game
//!
//! This crate implements a complete vampire RPG with pixel art graphics,
//! atmospheric environments, and survival mechanics.

pub mod components;
pub mod game_state;
pub mod input;
pub mod rendering;
pub mod systems;

// Re-export commonly used types for convenience
pub use components::{
    combat::{AIState, CombatStats},
    entities::{GameEntity, Health, Position, Velocity},
    environment::{BloodParticle, GroundTile, Moon, Star, TileType},
    game_data::{Clan, EntityType, GamePhase},
    shelter::{Shelter, ShelterCondition, ShelterOccupancy, ShelterType},
    vampire::{BloodMeter, VampireAbilities},
};
pub use game_state::GameState;
pub use input::InputHandler;
pub use rendering::Renderer;
pub use systems::{
    AISystem, BloodStatus, BloodSystem, ObjectiveProgress, ObjectivesSystem, PlayerStatus,
    PlayerSystem, ShelterInfo, ShelterSystem, TimeSystem, WorldSystem,
};

// Common imports for external use
pub use macroquad::prelude::*;
pub use serde::{Deserialize, Serialize};
pub use std::collections::HashMap;
