//! Centralized color palette for the vampire RPG game
//!
//! This module provides a consistent color system across the entire application,
//! making it easier to maintain visual consistency and implement theme changes.

use macroquad::prelude::Color;

/// Game color palette with semantic naming for better maintainability
pub struct GameColors;

impl GameColors {
    // ===== GROUND & TERRAIN COLORS =====

    /// Base grass color - dark green
    pub const GROUND_GRASS_BASE: Color = Color::new(0.2, 0.4, 0.1, 1.0);

    /// Grass detail patches - brighter green
    pub const GROUND_GRASS_DETAIL: Color = Color::new(0.3, 0.6, 0.2, 1.0);

    /// Dead grass base color - brownish yellow
    pub const GROUND_DEAD_GRASS_BASE: Color = Color::new(0.4, 0.3, 0.1, 1.0);

    /// Dead grass detail patches - lighter brown
    pub const GROUND_DEAD_GRASS_DETAIL: Color = Color::new(0.5, 0.4, 0.2, 1.0);

    /// Base dirt color - medium brown
    pub const GROUND_DIRT_BASE: Color = Color::new(0.4, 0.2, 0.1, 1.0);

    /// Dirt spots - darker brown
    pub const GROUND_DIRT_SPOTS: Color = Color::new(0.3, 0.15, 0.05, 1.0);

    /// Stone base color - light brown (changed from grey to distinguish from fog)
    pub const GROUND_STONE_BASE: Color = Color::new(0.6, 0.4, 0.2, 1.0);

    /// Stone detail blocks - lighter brown
    pub const GROUND_STONE_DETAIL: Color = Color::new(0.7, 0.5, 0.3, 1.0);

    // ===== FOG OF WAR COLORS =====

    /// Fog of war color - black with high opacity for maximum visibility
    pub const FOG_BLACK: Color = Color::new(0.0, 0.0, 0.0, 0.7);

    /// Minimum fog alpha for guaranteed visibility
    pub const FOG_MIN_ALPHA: f32 = 0.6;

    // ===== BACKGROUND & ENVIRONMENT =====

    /// Night sky background - dark blue
    pub const NIGHT_SKY: Color = Color::new(0.05, 0.05, 0.15, 1.0);

    /// Horizon indicator when active - bright yellow
    pub const HORIZON_ACTIVE: Color = Color::new(1.0, 0.8, 0.2, 0.6);

    /// Horizon indicator when inactive - dim blue
    pub const HORIZON_INACTIVE: Color = Color::new(0.4, 0.4, 0.6, 0.2);

    /// Horizon text - bright yellow
    pub const HORIZON_TEXT: Color = Color::new(1.0, 1.0, 0.0, 1.0);

    // ===== ENTITY & COMBAT COLORS =====

    /// Blood particles - bright red
    pub const BLOOD_PARTICLE: Color = Color::new(1.0, 0.0, 0.0, 1.0);

    /// Health bar background - dark red
    pub const HEALTH_BAR_BACKGROUND: Color = Color::new(0.3, 0.0, 0.0, 0.8);

    // ===== UI COLORS =====

    /// Blood meter background - dark red
    pub const UI_BLOOD_METER_BG: Color = Color::new(0.3, 0.0, 0.0, 1.0);

    /// Other meter background - dark blue
    pub const UI_METER_BG: Color = Color::new(0.0, 0.0, 0.3, 1.0);

    /// Pause menu overlay - semi-transparent black
    pub const UI_PAUSE_OVERLAY: Color = Color::new(0.0, 0.0, 0.0, 0.7);

    /// Clan menu background - dark blue-grey
    pub const UI_CLAN_MENU_BG: Color = Color::new(0.1, 0.1, 0.2, 0.9);

    /// Legend background - semi-transparent black
    pub const UI_LEGEND_BG: Color = Color::new(0.0, 0.0, 0.0, 0.8);

    // ===== SHELTER COLORS =====

    /// Cave shelter primary color - medium brown
    pub const SHELTER_CAVE_PRIMARY: Color = Color::new(0.4, 0.3, 0.2, 1.0);

    /// Cave shelter secondary color - dark brown
    pub const SHELTER_CAVE_SECONDARY: Color = Color::new(0.2, 0.1, 0.1, 1.0);

    /// Building shelter primary color - light brown
    pub const SHELTER_BUILDING_PRIMARY: Color = Color::new(0.6, 0.4, 0.3, 1.0);

    /// Building shelter secondary color - dark brown
    pub const SHELTER_BUILDING_SECONDARY: Color = Color::new(0.4, 0.3, 0.2, 1.0);

    /// Tree cover shelter primary color - dark green
    pub const SHELTER_TREE_PRIMARY: Color = Color::new(0.2, 0.5, 0.2, 1.0);

    /// Tree cover shelter secondary color - darker green
    pub const SHELTER_TREE_SECONDARY: Color = Color::new(0.1, 0.3, 0.1, 1.0);

    /// Underground shelter primary color - brown
    pub const SHELTER_UNDERGROUND_PRIMARY: Color = Color::new(0.4, 0.3, 0.2, 1.0);

    /// Underground shelter secondary color - dark brown
    pub const SHELTER_UNDERGROUND_SECONDARY: Color = Color::new(0.3, 0.2, 0.1, 1.0);

    /// Ruins shelter primary color - tan
    pub const SHELTER_RUINS_PRIMARY: Color = Color::new(0.5, 0.4, 0.3, 1.0);

    /// Ruins shelter secondary color - light tan
    pub const SHELTER_RUINS_SECONDARY: Color = Color::new(0.7, 0.6, 0.4, 1.0);

    /// Shed shelter primary color - dark brown
    pub const SHELTER_SHED_PRIMARY: Color = Color::new(0.4, 0.2, 0.1, 1.0);

    /// Shed shelter secondary color - orange-brown
    pub const SHELTER_SHED_SECONDARY: Color = Color::new(0.6, 0.3, 0.1, 1.0);

    /// Bridge underpass primary color - brown
    pub const SHELTER_BRIDGE_PRIMARY: Color = Color::new(0.5, 0.4, 0.3, 1.0);

    /// Bridge underpass secondary color - brown
    pub const SHELTER_BRIDGE_SECONDARY: Color = Color::new(0.6, 0.5, 0.4, 1.0);
}

/// Extension trait to add semantic color methods to the Color type
pub trait ColorExt {
    /// Create a color with reduced alpha for transparency effects
    fn with_alpha(self, alpha: f32) -> Color;

    /// Create a darker version of this color
    fn darker(self, factor: f32) -> Color;

    /// Create a lighter version of this color
    fn lighter(self, factor: f32) -> Color;
}

impl ColorExt for Color {
    fn with_alpha(mut self, alpha: f32) -> Color {
        self.a = alpha.clamp(0.0, 1.0);
        self
    }

    fn darker(mut self, factor: f32) -> Color {
        let factor = factor.clamp(0.0, 1.0);
        self.r *= factor;
        self.g *= factor;
        self.b *= factor;
        self
    }

    fn lighter(mut self, factor: f32) -> Color {
        let factor = factor.max(1.0);
        self.r = (self.r * factor).min(1.0);
        self.g = (self.g * factor).min(1.0);
        self.b = (self.b * factor).min(1.0);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_with_alpha() {
        let color = GameColors::GROUND_GRASS_BASE.with_alpha(0.5);
        assert_eq!(color.a, 0.5);
    }

    #[test]
    fn test_color_darker() {
        let original = GameColors::GROUND_GRASS_BASE;
        let darker = original.darker(0.5);
        assert!(darker.r < original.r);
        assert!(darker.g < original.g);
        assert!(darker.b < original.b);
    }

    #[test]
    fn test_color_lighter() {
        let original = GameColors::GROUND_GRASS_BASE;
        let lighter = original.lighter(1.5);
        assert!(lighter.r >= original.r);
        assert!(lighter.g >= original.g);
        assert!(lighter.b >= original.b);
    }

    #[test]
    fn test_all_colors_are_valid() {
        // Test that all color values are within valid ranges
        let colors = [
            GameColors::GROUND_GRASS_BASE,
            GameColors::GROUND_DIRT_BASE,
            GameColors::GROUND_STONE_BASE,
            GameColors::FOG_BLACK,
            GameColors::NIGHT_SKY,
            GameColors::BLOOD_PARTICLE,
        ];

        for color in &colors {
            assert!(
                color.r >= 0.0 && color.r <= 1.0,
                "Red component out of range"
            );
            assert!(
                color.g >= 0.0 && color.g <= 1.0,
                "Green component out of range"
            );
            assert!(
                color.b >= 0.0 && color.b <= 1.0,
                "Blue component out of range"
            );
            assert!(
                color.a >= 0.0 && color.a <= 1.0,
                "Alpha component out of range"
            );
        }
    }

    #[test]
    fn test_fog_min_alpha() {
        assert!(GameColors::FOG_MIN_ALPHA >= 0.0 && GameColors::FOG_MIN_ALPHA <= 1.0);
    }
}
