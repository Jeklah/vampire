//! Components module - Data structures for the ECS system
//!
//! This module contains all the component types used in the vampire RPG.
//! Components represent data that can be attached to entities.

pub mod combat;
pub mod entities;
pub mod entity_iterator;
pub mod environment;
pub mod game_data;
pub mod shelter;
pub mod vampire;

// Re-export all component types for easy access
pub use combat::*;
pub use entities::*;
pub use entity_iterator::*;
pub use environment::*;
pub use game_data::*;
pub use shelter::*;
pub use vampire::*;
