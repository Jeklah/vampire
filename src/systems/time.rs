//! Time System Module
//!
//! Manages the day/night cycle and time progression in the Vampire RPG.
//! This system handles time advancement, sunlight calculations, and day counting.

use serde::{Deserialize, Serialize};

/// Time system responsible for day/night cycle management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSystem {
    /// Current time in hours (0.0 to 24.0)
    current_time: f32,
    /// Real-time seconds for a full day cycle
    day_length: f32,
    /// Number of days that have passed
    day_count: u32,
    /// Whether it's currently daytime
    is_day: bool,
}

impl TimeSystem {
    /// Create a new time system starting at night
    pub fn new() -> Self {
        Self {
            current_time: 20.0, // Start at 8 PM (night time)
            day_length: 120.0,  // 2 minutes per full day
            day_count: 0,
            is_day: false,
        }
    }

    /// Create a time system with custom settings
    pub fn with_settings(start_time: f32, day_length: f32) -> Self {
        let is_day = start_time >= 6.0 && start_time < 18.0;
        Self {
            current_time: start_time,
            day_length,
            day_count: 0,
            is_day,
        }
    }

    /// Update the time system
    pub fn update(&mut self, delta_time: f32) {
        // Advance time based on day length
        self.current_time += (delta_time / self.day_length) * 24.0;

        // Handle day rollover
        if self.current_time >= 24.0 {
            self.current_time -= 24.0;
            self.day_count += 1;
        }

        // Update day/night status (6 AM to 6 PM is day)
        self.is_day = self.current_time >= 6.0 && self.current_time < 18.0;
    }

    /// Get formatted time string (HH:MM)
    pub fn get_time_string(&self) -> String {
        let hours = self.current_time as u32;
        let minutes = ((self.current_time - hours as f32) * 60.0) as u32;
        format!("{:02}:{:02}", hours, minutes)
    }

    /// Get current hour (0-23)
    pub fn get_hour(&self) -> u32 {
        self.current_time as u32
    }

    /// Get current minute (0-59)
    pub fn get_minute(&self) -> u32 {
        ((self.current_time - self.get_hour() as f32) * 60.0) as u32
    }

    /// Check if it's currently day time
    pub fn is_day(&self) -> bool {
        self.is_day
    }

    /// Check if it's currently night time
    pub fn is_night(&self) -> bool {
        !self.is_day
    }

    /// Get number of days that have passed
    pub fn day_count(&self) -> u32 {
        self.day_count
    }

    /// Get current time as float (0.0-24.0)
    pub fn current_time(&self) -> f32 {
        self.current_time
    }

    /// Calculate sunlight intensity (0.0 to 1.0)
    /// Returns 0.0 at night, peaks at 1.0 at noon
    pub fn get_sunlight_intensity(&self) -> f32 {
        if !self.is_day {
            return 0.0;
        }

        // Calculate distance from noon (12:00)
        let noon_distance = (self.current_time - 12.0).abs();
        let max_distance = 6.0; // 6 hours from noon (6 AM or 6 PM)

        if noon_distance > max_distance {
            0.0
        } else {
            // Linear interpolation from 0 at sunrise/sunset to 1 at noon
            1.0 - (noon_distance / max_distance)
        }
    }

    /// Get time until next dawn (in hours)
    pub fn time_until_dawn(&self) -> f32 {
        if self.current_time < 6.0 {
            6.0 - self.current_time
        } else {
            24.0 - self.current_time + 6.0
        }
    }

    /// Get time until next dusk (in hours)
    pub fn time_until_dusk(&self) -> f32 {
        if self.current_time < 18.0 {
            18.0 - self.current_time
        } else {
            24.0 - self.current_time + 18.0
        }
    }

    /// Get time period as a string (Dawn, Morning, Noon, Afternoon, Dusk, Night, Late Night)
    pub fn get_time_period(&self) -> &'static str {
        match self.current_time {
            t if t >= 5.0 && t < 7.0 => "Dawn",
            t if t >= 7.0 && t < 11.0 => "Morning",
            t if t >= 11.0 && t < 13.0 => "Noon",
            t if t >= 13.0 && t < 17.0 => "Afternoon",
            t if t >= 17.0 && t < 19.0 => "Dusk",
            t if t >= 19.0 && t < 23.0 => "Night",
            _ => "Late Night",
        }
    }

    /// Check if it's a dangerous time for vampires (high sunlight)
    pub fn is_dangerous_for_vampires(&self) -> bool {
        self.get_sunlight_intensity() > 0.5
    }

    /// Set the current time (for testing or events)
    pub fn set_time(&mut self, time: f32) {
        self.current_time = time.clamp(0.0, 24.0);
        self.is_day = self.current_time >= 6.0 && self.current_time < 18.0;
    }

    /// Advance time by a specific number of hours
    pub fn advance_hours(&mut self, hours: f32) {
        self.current_time += hours;
        while self.current_time >= 24.0 {
            self.current_time -= 24.0;
            self.day_count += 1;
        }
        self.is_day = self.current_time >= 6.0 && self.current_time < 18.0;
    }
}

impl Default for TimeSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_system_creation() {
        let time_system = TimeSystem::new();
        assert_eq!(time_system.current_time(), 20.0);
        assert!(!time_system.is_day());
        assert_eq!(time_system.day_count(), 0);
    }

    #[test]
    fn test_time_advancement() {
        let mut time_system = TimeSystem::new();
        time_system.update(60.0); // 1 minute with 120s day length = 12 hours
        assert_eq!(time_system.current_time(), 8.0); // 20 + 12 - 24 = 8 (next day)
        assert!(time_system.is_day());
        assert_eq!(time_system.day_count(), 1);
    }

    #[test]
    fn test_sunlight_intensity() {
        let mut time_system = TimeSystem::new();

        // Test noon (maximum intensity)
        time_system.set_time(12.0);
        assert_eq!(time_system.get_sunlight_intensity(), 1.0);

        // Test night (no intensity)
        time_system.set_time(0.0);
        assert_eq!(time_system.get_sunlight_intensity(), 0.0);

        // Test dawn/dusk (minimal intensity)
        time_system.set_time(6.0);
        assert_eq!(time_system.get_sunlight_intensity(), 0.0);
    }

    #[test]
    fn test_time_periods() {
        let mut time_system = TimeSystem::new();

        time_system.set_time(6.0);
        assert_eq!(time_system.get_time_period(), "Dawn");

        time_system.set_time(9.0);
        assert_eq!(time_system.get_time_period(), "Morning");

        time_system.set_time(12.0);
        assert_eq!(time_system.get_time_period(), "Noon");

        time_system.set_time(21.0);
        assert_eq!(time_system.get_time_period(), "Night");
    }
}
