use crate::controller_state::ControllerState;
use crate::keyboard_state::KeyboardState;
use crate::mouse_state::MouseState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct InputState {
    mouse: MouseState,
    keyboard: KeyboardState,
    controllers: Vec<ControllerState>,
}

impl InputState {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            mouse: MouseState::default(),
            keyboard: KeyboardState::default(),
            controllers: Vec::new(),
        }
    }

    #[inline]
    #[must_use]
    pub fn mouse(&self) -> &MouseState {
        &self.mouse
    }

    #[inline]
    #[must_use]
    pub fn mouse_mut(&mut self) -> &mut MouseState {
        &mut self.mouse
    }

    #[inline]
    #[must_use]
    pub fn keyboard(&self) -> &KeyboardState {
        &self.keyboard
    }

    #[inline]
    #[must_use]
    pub fn keyboard_mut(&mut self) -> &mut KeyboardState {
        &mut self.keyboard
    }

    #[must_use]
    pub fn get_or_insert_controller_mut(&mut self, index: usize) -> &mut ControllerState {
        if index >= self.controllers.len() {
            self.controllers
                .resize(index + 1, ControllerState::default());
        }
        &mut self.controllers[index]
    }

    #[inline]
    #[must_use]
    pub fn controllers(&self) -> &[ControllerState] {
        &self.controllers
    }

    pub fn reset_counts(&mut self) {
        self.mouse.clear();
        self.keyboard.reset_counts();
        for controller in &mut self.controllers {
            controller.reset_counts();
        }
    }
}
