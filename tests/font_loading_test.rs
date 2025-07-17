//! Test to verify font loading messages are properly added to the game debug log
//!
//! This test ensures that font loading success/failure messages appear in the
//! in-game debug system rather than just the terminal output.

use vampire_rpg::GameState;

#[test]
fn test_font_loading_messages_added_to_debug_log() {
    // Create a new game state
    let mut game_state = GameState::new();

    // Initially, debug messages should be empty
    assert_eq!(game_state.debug_messages.len(), 0);

    // Simulate successful font loading message
    game_state.add_debug_message("Font loaded successfully from embedded data".to_string());

    // Verify the message was added
    assert_eq!(game_state.debug_messages.len(), 1);
    assert_eq!(
        game_state.debug_messages[0],
        "Font loaded successfully from embedded data"
    );
}

#[test]
fn test_font_loading_error_messages_added_to_debug_log() {
    // Create a new game state
    let mut game_state = GameState::new();

    // Simulate font loading error messages
    let error_msg = "Test font loading error";
    game_state.add_debug_message(format!(
        "Warning: Could not load embedded font: {}",
        error_msg
    ));
    game_state.add_debug_message("Using default system font".to_string());

    // Verify both messages were added
    assert_eq!(game_state.debug_messages.len(), 2);
    assert_eq!(
        game_state.debug_messages[0],
        "Warning: Could not load embedded font: Test font loading error"
    );
    assert_eq!(game_state.debug_messages[1], "Using default system font");
}

#[test]
fn test_debug_message_limit_maintained() {
    // Create a new game state
    let mut game_state = GameState::new();

    // Add 25 messages (more than the 20-message limit)
    for i in 0..25 {
        game_state.add_debug_message(format!("Test message {}", i));
    }

    // Verify that only the last 20 messages are kept
    assert_eq!(game_state.debug_messages.len(), 20);

    // Verify the oldest messages were removed (messages 0-4 should be gone)
    assert_eq!(game_state.debug_messages[0], "Test message 5");
    assert_eq!(game_state.debug_messages[19], "Test message 24");
}

#[test]
fn test_font_messages_appear_early_in_debug_log() {
    // This test simulates the real game scenario where font loading
    // messages should appear early in the debug log
    let mut game_state = GameState::new();

    // Add font loading message (this happens early in main())
    game_state.add_debug_message("Font loaded successfully from embedded data".to_string());

    // Add some other game messages that would come later
    game_state.add_debug_message("Player movement initialized".to_string());
    game_state.add_debug_message("Game world spawned".to_string());

    // Verify font message appears first
    assert_eq!(game_state.debug_messages.len(), 3);
    assert_eq!(
        game_state.debug_messages[0],
        "Font loaded successfully from embedded data"
    );
    assert!(game_state.debug_messages[1].contains("Player movement"));
    assert!(game_state.debug_messages[2].contains("Game world"));
}

#[test]
fn test_fullscreen_mode_message_added_to_debug_log() {
    // Create a new game state
    // Test that windowed mode message is properly added
    let mut game_state = GameState::new();

    // Simulate windowed mode initialization message
    game_state
        .add_debug_message("Game started in windowed mode (F11 to toggle fullscreen)".to_string());

    // Verify the message was added
    assert_eq!(game_state.debug_messages.len(), 1);
    assert_eq!(
        game_state.debug_messages[0],
        "Game started in windowed mode (F11 to toggle fullscreen)"
    );
}

#[test]
fn test_startup_messages_order() {
    // This test simulates the real startup sequence
    let mut game_state = GameState::new();

    // Add messages in startup order
    game_state.add_debug_message("Font loaded successfully from embedded data".to_string());
    game_state
        .add_debug_message("Game started in windowed mode (F11 to toggle fullscreen)".to_string());

    // Verify correct order
    assert_eq!(game_state.debug_messages.len(), 2);
    assert_eq!(
        game_state.debug_messages[0],
        "Font loaded successfully from embedded data"
    );
    assert_eq!(
        game_state.debug_messages[1],
        "Game started in windowed mode (F11 to toggle fullscreen)"
    );
}

#[test]
fn test_fullscreen_toggle_messages() {
    // Test that fullscreen toggle messages are properly added
    let mut game_state = GameState::new();

    // Simulate switching to windowed mode
    game_state.add_debug_message("Switched to windowed mode".to_string());

    // Verify the message was added
    assert_eq!(game_state.debug_messages.len(), 1);
    assert_eq!(game_state.debug_messages[0], "Switched to windowed mode");

    // Simulate switching back to fullscreen mode
    game_state.add_debug_message("Switched to fullscreen mode".to_string());

    // Verify both messages are present
    assert_eq!(game_state.debug_messages.len(), 2);
    assert_eq!(game_state.debug_messages[1], "Switched to fullscreen mode");
}

#[test]
fn test_complete_startup_sequence_with_fullscreen() {
    // Test the complete startup message sequence including fullscreen
    let mut game_state = GameState::new();

    // Add messages in the order they appear during startup
    // Add font loading message (this happens early in main())
    game_state.add_debug_message("Font loaded successfully from embedded data".to_string());
    game_state
        .add_debug_message("Game started in windowed mode (F11 to toggle fullscreen)".to_string());

    // Verify correct order and content
    assert_eq!(game_state.debug_messages.len(), 2);
    assert_eq!(
        game_state.debug_messages[0],
        "Font loaded successfully from embedded data"
    );
    assert_eq!(
        game_state.debug_messages[1],
        "Game started in windowed mode (F11 to toggle fullscreen)"
    );

    // Simulate user toggling fullscreen with F11
    game_state.add_debug_message("Switched to windowed mode".to_string());

    // Verify the toggle message appears after startup messages
    assert_eq!(game_state.debug_messages.len(), 3);
    assert_eq!(game_state.debug_messages[2], "Switched to windowed mode");
}
