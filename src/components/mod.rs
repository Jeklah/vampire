//! Components module - Data structures for the ECS system
//!
//! This module contains all the component types used in the vampire RPG.
//! Components represent data that can be attached to entities.

pub mod combat;
pub mod entities;
pub mod environment;
pub mod game_data;
pub mod vampire;

// Re-export all component types for easy access
pub use combat::*;
pub use entities::*;
pub use environment::*;
pub use game_data::*;
pub use vampire::*;
