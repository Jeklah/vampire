//! Performance optimization tests
//!
//! These tests verify that the performance improvements are working correctly
//! and that the optimizations don't break existing functionality.

use vampire_rpg::components::*;
use vampire_rpg::{GameState, Renderer};

#[test]
fn test_ground_tile_texture_data_generation() {
    // Test that ground tiles pre-generate texture data correctly
    let grass_tile = GroundTile::new(0.0, 0.0, TileType::Grass);
    let dirt_tile = GroundTile::new(64.0, 0.0, TileType::Dirt);
    let stone_tile = GroundTile::new(128.0, 0.0, TileType::Stone);

    // Verify grass tile has pre-generated patches
    assert!(!grass_tile.texture_data.grass_patches.is_empty());
    assert_eq!(grass_tile.texture_data.grass_patches.len(), 32); // 8x4 patches

    // Verify dirt tile has pre-generated spots
    assert!(!dirt_tile.texture_data.dirt_spots.is_empty());
    assert_eq!(dirt_tile.texture_data.dirt_spots.len(), 12);

    // Verify stone tile has pre-generated blocks
    assert!(!stone_tile.texture_data.stone_blocks.is_empty());
    assert_eq!(stone_tile.texture_data.stone_blocks.len(), 16); // 4x4 blocks
}

#[test]
fn test_tile_texture_data_consistency() {
    // Test that the same tile type generates consistent texture data
    let tile1 = GroundTile::new(0.0, 0.0, TileType::Grass);
    let tile2 = GroundTile::new(64.0, 64.0, TileType::Grass);

    // Both should have the same number of patches (though positions may differ due to random)
    assert_eq!(
        tile1.texture_data.grass_patches.len(),
        tile2.texture_data.grass_patches.len()
    );

    // Positions should be within valid ranges
    for (x, y, w, h) in &tile1.texture_data.grass_patches {
        assert!(*x >= -4.0 && *x <= 64.0, "X position out of range: {}", x);
        assert!(*y >= -4.0 && *y <= 64.0, "Y position out of range: {}", y);
        assert!(*w > 0.0, "Width should be positive: {}", w);
        assert!(*h > 0.0, "Height should be positive: {}", h);
    }
}

#[test]
fn test_renderer_performance_mode() {
    // Test that performance mode can be toggled
    let mut renderer = Renderer::new(None);

    // Should start in normal mode
    assert!(!renderer.performance_mode());

    // Enable performance mode
    renderer.set_performance_mode(true);
    assert!(renderer.performance_mode());

    // Disable performance mode
    renderer.set_performance_mode(false);
    assert!(!renderer.performance_mode());
}

#[test]
fn test_tile_texture_data_default() {
    // Test TileTextureData default implementation
    let texture_data = TileTextureData::default();

    assert!(texture_data.grass_patches.is_empty());
    assert!(texture_data.dirt_spots.is_empty());
    assert!(texture_data.stone_blocks.is_empty());
}

#[test]
fn test_different_tile_types_generate_appropriate_data() {
    let grass_tile = GroundTile::new(0.0, 0.0, TileType::Grass);
    let dead_grass_tile = GroundTile::new(0.0, 0.0, TileType::DeadGrass);
    let dirt_tile = GroundTile::new(0.0, 0.0, TileType::Dirt);
    let stone_tile = GroundTile::new(0.0, 0.0, TileType::Stone);

    // Grass and dead grass should have grass patches but no dirt spots or stone blocks
    assert!(!grass_tile.texture_data.grass_patches.is_empty());
    assert!(grass_tile.texture_data.dirt_spots.is_empty());
    assert!(grass_tile.texture_data.stone_blocks.is_empty());

    assert!(!dead_grass_tile.texture_data.grass_patches.is_empty());
    assert!(dead_grass_tile.texture_data.dirt_spots.is_empty());
    assert!(dead_grass_tile.texture_data.stone_blocks.is_empty());

    // Dirt should have dirt spots but no grass patches or stone blocks
    assert!(dirt_tile.texture_data.grass_patches.is_empty());
    assert!(!dirt_tile.texture_data.dirt_spots.is_empty());
    assert!(dirt_tile.texture_data.stone_blocks.is_empty());

    // Stone should have stone blocks but no grass patches or dirt spots
    assert!(stone_tile.texture_data.grass_patches.is_empty());
    assert!(stone_tile.texture_data.dirt_spots.is_empty());
    assert!(!stone_tile.texture_data.stone_blocks.is_empty());
}

#[test]
fn test_ground_tile_creation_with_texture_data() {
    // Test that creating a ground tile automatically generates texture data
    let tile = GroundTile::new(100.0, 200.0, TileType::Grass);

    assert_eq!(tile.x, 100.0);
    assert_eq!(tile.y, 200.0);
    assert!(matches!(tile.tile_type, TileType::Grass));

    // Should have texture data generated
    match tile.tile_type {
        TileType::Grass => {
            assert!(!tile.texture_data.grass_patches.is_empty());
        }
        _ => panic!("Expected grass tile"),
    }
}

#[test]
fn test_performance_optimizations_dont_break_game_state() {
    // Test that performance optimizations don't interfere with game state
    let mut game_state = GameState::new();
    let mut renderer = Renderer::new(None);

    // Game state should initialize normally
    assert!(game_state.entities.len() > 0); // Should have at least the player
    assert!(game_state.ground_tiles.len() > 0); // Should have ground tiles

    // Performance mode should not affect game state creation
    renderer.set_performance_mode(true);

    // Should still be able to add debug messages
    game_state.add_debug_message("Performance test message".to_string());
    assert!(game_state
        .debug_messages
        .contains(&"Performance test message".to_string()));
}

#[test]
fn test_dirt_spot_generation_bounds() {
    // Test that dirt spots are generated within reasonable bounds
    let dirt_tile = GroundTile::new(0.0, 0.0, TileType::Dirt);

    for (x, y, radius) in &dirt_tile.texture_data.dirt_spots {
        // Positions should be within tile bounds
        assert!(*x >= 0.0 && *x <= 64.0, "Dirt spot X out of bounds: {}", x);
        assert!(*y >= 0.0 && *y <= 64.0, "Dirt spot Y out of bounds: {}", y);

        // Radius should be reasonable
        assert!(
            *radius >= 2.0 && *radius <= 4.0,
            "Dirt spot radius out of range: {}",
            radius
        );
    }
}

#[test]
fn test_stone_block_generation_pattern() {
    // Test that stone blocks are generated in a grid pattern
    let stone_tile = GroundTile::new(0.0, 0.0, TileType::Stone);

    // Should have exactly 16 blocks (4x4 grid)
    assert_eq!(stone_tile.texture_data.stone_blocks.len(), 16);

    // Blocks should be arranged in a grid
    let mut grid_positions = std::collections::HashSet::new();
    for (x, y, _w, _h) in &stone_tile.texture_data.stone_blocks {
        // Positions should be multiples of 16
        assert_eq!(*x % 16.0, 0.0, "Stone block X not aligned to grid: {}", x);
        assert_eq!(*y % 16.0, 0.0, "Stone block Y not aligned to grid: {}", y);

        // Should not have duplicate positions
        assert!(
            grid_positions.insert((*x as i32, *y as i32)),
            "Duplicate stone block position: ({}, {})",
            x,
            y
        );
    }
}

#[test]
fn test_grass_patch_count_differences() {
    // Test that grass and dead grass have different patch counts
    let grass_tile = GroundTile::new(0.0, 0.0, TileType::Grass);
    let dead_grass_tile = GroundTile::new(0.0, 0.0, TileType::DeadGrass);

    // Grass: 8x4 = 32 patches
    assert_eq!(grass_tile.texture_data.grass_patches.len(), 32);

    // Dead grass: 6x3 = 18 patches
    assert_eq!(dead_grass_tile.texture_data.grass_patches.len(), 18);
}
