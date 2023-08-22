use std::{collections::HashMap, sync::RwLock};

use glm::{Vec2, vec2};
use once_cell::sync::Lazy;

pub static INPUT: Input = Input {
    pressed_keys: RwLock::new(Lazy::new(|| HashMap::new())),
    press_began_keys: RwLock::new(Lazy::new(|| HashMap::new())),
    press_ended_keys: RwLock::new(Lazy::new(|| HashMap::new())),
    mouse_delta: RwLock::new(Lazy::new(||vec2(0.0, 0.0))),
    mouse_position: RwLock::new(Lazy::new(||vec2(0.0, 0.0))),
};



pub struct Input {
    pub(super) pressed_keys: RwLock<Lazy<HashMap<glfw::Key, bool>>>, // true if key is currently held down
    pub(super) press_began_keys: RwLock<Lazy<HashMap<glfw::Key, bool>>>, //true if key was pressed this frame
    pub(super) press_ended_keys: RwLock<Lazy<HashMap<glfw::Key, bool>>>, //true if key was released this frame
    pub(super) mouse_delta: RwLock<Lazy<Vec2>>,
    pub(super) mouse_position: RwLock<Lazy<Vec2>>,
}

impl Input {
    pub fn is_pressed(&self, key: glfw::Key) -> bool {
        let input = self.pressed_keys.read().unwrap();
        return input.contains_key(&key) && input[&key];
    }

    pub fn did_press_begin(&self, key: glfw::Key) -> bool {
        let input = self.press_began_keys.read().unwrap();
        return input.contains_key(&key) && input[&key];
    }

    pub fn did_press_end(&self, key: glfw::Key) -> bool {
        let input = self.press_ended_keys.read().unwrap();
        return input.contains_key(&key) && input[&key];
    }

    pub fn mouse_position(&self) -> Vec2 {
        return self.mouse_position.read().unwrap().clone();
    }

    pub fn mouse_delta(&self) -> Vec2 {
        return self.mouse_delta.read().unwrap().clone();
    }
}

