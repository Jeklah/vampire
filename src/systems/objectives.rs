//! Objectives System Module
//!
//! Handles objective tracking, progress monitoring, and completion logic.
//! This system manages the player's progression through different game phases.

use crate::components::*;
use std::collections::HashMap;

/// Objectives system responsible for tracking game progress and goals
pub struct ObjectivesSystem;

impl ObjectivesSystem {
    /// Check all objectives and update completion status
    pub fn check_objectives(
        entities: &[GameEntity],
        player_id: u32,
        time_system: &super::time::TimeSystem,
        clans: &HashMap<String, Clan>,
        kills: u32,
        feeding_count: u32,
        phase_objectives: &mut Vec<String>,
        completed_objectives: &mut Vec<String>,
    ) {
        // Check survival objectives
        Self::check_survival_objectives(time_system, phase_objectives, completed_objectives);

        // Check player ability objectives
        Self::check_ability_objectives(entities, player_id, phase_objectives, completed_objectives);

        // Check feeding objectives
        Self::check_feeding_objectives(feeding_count, phase_objectives, completed_objectives);

        // Check combat objectives
        Self::check_combat_objectives(kills, phase_objectives, completed_objectives);

        // Check shelter objectives
        Self::check_shelter_objectives(
            entities,
            player_id,
            time_system,
            phase_objectives,
            completed_objectives,
        );

        // Check clan objectives
        Self::check_clan_objectives(clans, phase_objectives, completed_objectives);

        // Check exploration objectives
        Self::check_exploration_objectives(
            entities,
            player_id,
            phase_objectives,
            completed_objectives,
        );
    }

    /// Check survival-related objectives
    fn check_survival_objectives(
        time_system: &super::time::TimeSystem,
        phase_objectives: &mut Vec<String>,
        completed_objectives: &mut Vec<String>,
    ) {
        // Weekly survival milestones
        if time_system.day_count() >= 7 {
            Self::complete_objective(
                "Survive your first week",
                phase_objectives,
                completed_objectives,
            );
        }

        if time_system.day_count() >= 30 {
            Self::complete_objective(
                "Survive for a month",
                phase_objectives,
                completed_objectives,
            );
        }

        if time_system.day_count() >= 365 {
            Self::complete_objective("Survive for a year", phase_objectives, completed_objectives);
        }
    }

    /// Check ability development objectives
    fn check_ability_objectives(
        entities: &[GameEntity],
        player_id: u32,
        phase_objectives: &mut Vec<String>,
        completed_objectives: &mut Vec<String>,
    ) {
        if let Some(player) = entities.iter().find(|e| e.id == player_id) {
            if let Some(abilities) = &player.vampire_abilities {
                // Basic ability improvement
                if abilities.strength > 1.0 || abilities.speed > 1.0 {
                    Self::complete_objective(
                        "Discover your vampire abilities",
                        phase_objectives,
                        completed_objectives,
                    );
                }

                // Advanced ability milestones
                if abilities.strength > 2.0 {
                    Self::complete_objective(
                        "Develop superhuman strength",
                        phase_objectives,
                        completed_objectives,
                    );
                }

                if abilities.speed > 2.0 {
                    Self::complete_objective(
                        "Achieve supernatural speed",
                        phase_objectives,
                        completed_objectives,
                    );
                }

                if abilities.blood_sense > 3.0 {
                    Self::complete_objective(
                        "Master blood sensing",
                        phase_objectives,
                        completed_objectives,
                    );
                }

                if abilities.shadow_movement > 2.0 {
                    Self::complete_objective(
                        "Learn shadow manipulation",
                        phase_objectives,
                        completed_objectives,
                    );
                }
            }
        }
    }

    /// Check feeding-related objectives
    fn check_feeding_objectives(
        feeding_count: u32,
        phase_objectives: &mut Vec<String>,
        completed_objectives: &mut Vec<String>,
    ) {
        if feeding_count >= 5 {
            Self::complete_objective(
                "Feed on blood sources",
                phase_objectives,
                completed_objectives,
            );
        }

        if feeding_count >= 25 {
            Self::complete_objective(
                "Master the art of feeding",
                phase_objectives,
                completed_objectives,
            );
        }

        if feeding_count >= 100 {
            Self::complete_objective(
                "Become an apex predator",
                phase_objectives,
                completed_objectives,
            );
        }
    }

    /// Check combat-related objectives
    fn check_combat_objectives(
        kills: u32,
        phase_objectives: &mut Vec<String>,
        completed_objectives: &mut Vec<String>,
    ) {
        if kills >= 10 {
            Self::complete_objective(
                "Prove your combat prowess",
                phase_objectives,
                completed_objectives,
            );
        }

        if kills >= 50 {
            Self::complete_objective(
                "Become a feared warrior",
                phase_objectives,
                completed_objectives,
            );
        }

        if kills >= 200 {
            Self::complete_objective(
                "Earn the title of Apex Hunter",
                phase_objectives,
                completed_objectives,
            );
        }
    }

    /// Check shelter and survival objectives
    fn check_shelter_objectives(
        entities: &[GameEntity],
        player_id: u32,
        time_system: &super::time::TimeSystem,
        phase_objectives: &mut Vec<String>,
        completed_objectives: &mut Vec<String>,
    ) {
        // Check if player survived day with reasonable health
        if time_system.day_count() >= 1 && time_system.is_day() {
            if let Some(player) = entities.iter().find(|e| e.id == player_id) {
                if let Some(health) = &player.health {
                    if health.current > 20.0 {
                        Self::complete_objective(
                            "Find shelter from sunlight",
                            phase_objectives,
                            completed_objectives,
                        );
                    }
                }
            }
        }

        // Advanced shelter objectives
        if time_system.day_count() >= 7 {
            if let Some(player) = entities.iter().find(|e| e.id == player_id) {
                if let Some(health) = &player.health {
                    if health.current > 50.0 && time_system.is_day() {
                        Self::complete_objective(
                            "Master daytime survival",
                            phase_objectives,
                            completed_objectives,
                        );
                    }
                }
            }
        }
    }

    /// Check clan-related objectives
    fn check_clan_objectives(
        clans: &HashMap<String, Clan>,
        phase_objectives: &mut Vec<String>,
        completed_objectives: &mut Vec<String>,
    ) {
        let allied_count = clans.values().filter(|clan| clan.is_allied).count();
        let defeated_count = clans.values().filter(|clan| clan.is_defeated).count();

        // First contact
        if allied_count >= 1 {
            Self::complete_objective(
                "Establish contact with clan leaders",
                phase_objectives,
                completed_objectives,
            );
        }

        // Diplomatic achievements
        if allied_count >= 2 {
            Self::complete_objective(
                "Form alliances with multiple clans",
                phase_objectives,
                completed_objectives,
            );
        }

        if allied_count == clans.len() {
            Self::complete_objective(
                "Unite all vampire clans",
                phase_objectives,
                completed_objectives,
            );
        }

        // Conquest achievements
        if defeated_count >= 1 {
            Self::complete_objective(
                "Defeat a rival clan",
                phase_objectives,
                completed_objectives,
            );
        }

        if defeated_count == clans.len() {
            Self::complete_objective(
                "Conquer all vampire clans",
                phase_objectives,
                completed_objectives,
            );
        }

        // Check for high trust levels
        let high_trust_count = clans
            .values()
            .filter(|clan| clan.trust_towards_player > 0.8)
            .count();

        if high_trust_count >= 1 {
            Self::complete_objective(
                "Earn the deep trust of a clan",
                phase_objectives,
                completed_objectives,
            );
        }
    }

    /// Check exploration and world objectives
    fn check_exploration_objectives(
        entities: &[GameEntity],
        player_id: u32,
        phase_objectives: &mut Vec<String>,
        completed_objectives: &mut Vec<String>,
    ) {
        if let Some(player) = entities.iter().find(|e| e.id == player_id) {
            // Check if player has explored different areas of the map
            let x = player.position.x;
            let y = player.position.y;

            // Define exploration zones
            let explored_zones = Self::get_explored_zones(x, y);

            if explored_zones.len() >= 3 {
                Self::complete_objective(
                    "Explore the vampire territories",
                    phase_objectives,
                    completed_objectives,
                );
            }

            if explored_zones.len() >= 5 {
                Self::complete_objective(
                    "Map the entire realm",
                    phase_objectives,
                    completed_objectives,
                );
            }
        }
    }

    /// Get list of zones the player has explored based on position
    fn get_explored_zones(x: f32, y: f32) -> Vec<&'static str> {
        // Define zone boundaries and conditions using iterator pattern
        [
            ((x < 400.0 && y < 800.0), "Northwest Territory"),
            (
                (x >= 400.0 && x < 800.0 && y < 800.0),
                "North Central Territory",
            ),
            ((x >= 800.0 && y < 800.0), "Northeast Territory"),
            ((x < 400.0 && y >= 800.0), "Southwest Territory"),
            (
                (x >= 400.0 && x < 800.0 && y >= 800.0),
                "South Central Territory",
            ),
            ((x >= 800.0 && y >= 800.0), "Southeast Territory"),
        ]
        .iter()
        .filter_map(|&(condition, zone)| if condition { Some(zone) } else { None })
        .collect()
    }

    /// Complete an objective if it exists in the phase objectives
    fn complete_objective(
        objective: &str,
        phase_objectives: &mut Vec<String>,
        completed_objectives: &mut Vec<String>,
    ) {
        if let Some(pos) = phase_objectives.iter().position(|obj| obj == objective) {
            let completed = phase_objectives.remove(pos);
            completed_objectives.push(completed);
        }
    }

    /// Get initial objectives for a specific game phase
    pub fn get_initial_objectives(phase: &GamePhase) -> Vec<String> {
        match phase {
            GamePhase::SurvivalAndDiscovery => vec![
                "Survive your first week".to_string(),
                "Discover your vampire abilities".to_string(),
                "Find shelter from sunlight".to_string(),
                "Feed on blood sources".to_string(),
                "Explore the vampire territories".to_string(),
            ],
            GamePhase::ClanEncounters => vec![
                "Establish contact with clan leaders".to_string(),
                "Form alliances with multiple clans".to_string(),
                "Prove your combat prowess".to_string(),
                "Master the art of feeding".to_string(),
                "Develop superhuman strength".to_string(),
            ],
            GamePhase::EmpireBuilding => vec![
                "Unite all vampire clans".to_string(),
                "Become a feared warrior".to_string(),
                "Master daytime survival".to_string(),
                "Achieve supernatural speed".to_string(),
                "Map the entire realm".to_string(),
            ],
            GamePhase::WorldReaction => vec![
                "Conquer all vampire clans".to_string(),
                "Earn the title of Apex Hunter".to_string(),
                "Become an apex predator".to_string(),
                "Master blood sensing".to_string(),
                "Learn shadow manipulation".to_string(),
            ],
        }
    }

    /// Check if phase advancement is possible
    pub fn can_advance_phase(
        current_phase: &GamePhase,
        completed_objectives: &[String],
        day_count: u32,
        allied_clans: usize,
    ) -> bool {
        match current_phase {
            GamePhase::SurvivalAndDiscovery => {
                // Advance when basic survival is mastered
                day_count >= 7 && completed_objectives.len() >= 3
            }
            GamePhase::ClanEncounters => {
                // Advance when clan relations are established
                allied_clans >= 1 && completed_objectives.len() >= 8
            }
            GamePhase::EmpireBuilding => {
                // Advance when player has significant power
                allied_clans >= 2 && completed_objectives.len() >= 12
            }
            GamePhase::WorldReaction => {
                // Final phase - no advancement
                false
            }
        }
    }

    /// Get next phase after current one
    pub fn get_next_phase(current_phase: &GamePhase) -> Option<GamePhase> {
        match current_phase {
            GamePhase::SurvivalAndDiscovery => Some(GamePhase::ClanEncounters),
            GamePhase::ClanEncounters => Some(GamePhase::EmpireBuilding),
            GamePhase::EmpireBuilding => Some(GamePhase::WorldReaction),
            GamePhase::WorldReaction => None,
        }
    }

    /// Get objectives progress summary
    pub fn get_progress_summary(
        completed_objectives: &[String],
        phase_objectives: &[String],
        current_phase: &GamePhase,
    ) -> ObjectiveProgress {
        let total_completed = completed_objectives.len();
        let total_remaining = phase_objectives.len();
        let completion_percentage = if total_completed + total_remaining > 0 {
            (total_completed as f32 / (total_completed + total_remaining) as f32) * 100.0
        } else {
            0.0
        };

        ObjectiveProgress {
            current_phase: current_phase.clone(),
            completed_count: total_completed,
            remaining_count: total_remaining,
            completion_percentage,
            recent_completions: completed_objectives.iter().rev().take(3).cloned().collect(),
        }
    }
}

/// Objective progress information
#[derive(Debug, Clone)]
pub struct ObjectiveProgress {
    pub current_phase: GamePhase,
    pub completed_count: usize,
    pub remaining_count: usize,
    pub completion_percentage: f32,
    pub recent_completions: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_initial_objectives() {
        let objectives = ObjectivesSystem::get_initial_objectives(&GamePhase::SurvivalAndDiscovery);
        assert_eq!(objectives.len(), 5);
        assert!(objectives.contains(&"Survive your first week".to_string()));
    }

    #[test]
    fn test_can_advance_phase() {
        let completed = vec!["obj1".to_string(), "obj2".to_string(), "obj3".to_string()];

        assert!(ObjectivesSystem::can_advance_phase(
            &GamePhase::SurvivalAndDiscovery,
            &completed,
            7,
            0
        ));

        assert!(!ObjectivesSystem::can_advance_phase(
            &GamePhase::SurvivalAndDiscovery,
            &completed,
            3,
            0
        ));
    }

    #[test]
    fn test_get_next_phase() {
        assert_eq!(
            ObjectivesSystem::get_next_phase(&GamePhase::SurvivalAndDiscovery),
            Some(GamePhase::ClanEncounters)
        );

        assert_eq!(
            ObjectivesSystem::get_next_phase(&GamePhase::WorldReaction),
            None
        );
    }

    #[test]
    fn test_explored_zones() {
        let zones = ObjectivesSystem::get_explored_zones(200.0, 700.0);
        assert!(zones.contains(&"Northwest Territory"));

        let zones = ObjectivesSystem::get_explored_zones(900.0, 900.0);
        assert!(zones.contains(&"Southeast Territory"));
    }

    #[test]
    fn test_progress_summary() {
        let completed = vec!["obj1".to_string(), "obj2".to_string()];
        let remaining = vec!["obj3".to_string(), "obj4".to_string()];

        let progress = ObjectivesSystem::get_progress_summary(
            &completed,
            &remaining,
            &GamePhase::SurvivalAndDiscovery,
        );

        assert_eq!(progress.completed_count, 2);
        assert_eq!(progress.remaining_count, 2);
        assert_eq!(progress.completion_percentage, 50.0);
    }
}
