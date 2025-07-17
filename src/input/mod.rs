//! Input Handler Module
//!
//! This module provides centralized input handling for the Vampire RPG.

use macroquad::prelude::*;
use std::collections::HashSet;

pub struct InputHandler {
    keys_pressed: HashSet<KeyCode>,
    keys_just_pressed: HashSet<KeyCode>,
    keys_just_released: HashSet<KeyCode>,
    previous_keys: HashSet<KeyCode>,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            keys_pressed: HashSet::new(),
            keys_just_pressed: HashSet::new(),
            keys_just_released: HashSet::new(),
            previous_keys: HashSet::new(),
        }
    }

    pub fn update(&mut self) {
        // Clear just pressed/released from previous frame
        self.keys_just_pressed.clear();
        self.keys_just_released.clear();

        // Get currently pressed keys
        let mut current_keys = HashSet::new();

        // Check all relevant keys
        let keys_to_check = [
            KeyCode::W,
            KeyCode::A,
            KeyCode::S,
            KeyCode::D,
            KeyCode::Space,
            KeyCode::R,
            KeyCode::E,
            KeyCode::F,
            KeyCode::Escape,
            KeyCode::Tab,
            KeyCode::L,
            KeyCode::H,
            KeyCode::Q,
            KeyCode::LeftControl,
        ];

        for &key in &keys_to_check {
            if is_key_down(key) {
                current_keys.insert(key);
            }
        }

        // Determine just pressed keys (in current but not in previous)
        for &key in &current_keys {
            if !self.previous_keys.contains(&key) {
                self.keys_just_pressed.insert(key);
            }
        }

        // Determine just released keys (in previous but not in current)
        for &key in &self.previous_keys {
            if !current_keys.contains(&key) {
                self.keys_just_released.insert(key);
            }
        }

        // Update state
        self.keys_pressed = current_keys.clone();
        self.previous_keys = current_keys;
    }

    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.keys_pressed.contains(&key)
    }

    pub fn is_key_just_pressed(&self, key: KeyCode) -> bool {
        self.keys_just_pressed.contains(&key)
    }

    pub fn is_key_just_released(&self, key: KeyCode) -> bool {
        self.keys_just_released.contains(&key)
    }

    pub fn is_quit_requested(&self) -> bool {
        self.is_key_pressed(KeyCode::Q) && self.is_key_pressed(KeyCode::LeftControl)
    }
}

impl Default for InputHandler {
    fn default() -> Self {
        Self::new()
    }
}
