use crate::mouse_state::MouseState;
use crate::{button_state::ButtonState, controller_state::ControllerState};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct InputState {
    mouse: MouseState,
    keyboard: ControllerState,
    controllers: Vec<ControllerState>,
}

impl InputState {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        // The keyboard is always enabled. Controllers will be enabled/disabled
        // as they are detected.
        let mut keyboard = ControllerState::default();
        keyboard.set_enabled(true);

        Self {
            mouse: MouseState::default(),
            keyboard,
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
    pub fn keyboard(&self) -> &ControllerState {
        &self.keyboard
    }

    #[inline]
    #[must_use]
    pub fn keyboard_mut(&mut self) -> &mut ControllerState {
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

    #[inline]
    pub fn track_down(button_state: &mut ButtonState, is_pressed: bool) {
        button_state.track_down(is_pressed);
    }

    pub fn reset_counts(&mut self) {
        self.keyboard.reset_counts();
        for controller in &mut self.controllers {
            controller.reset_counts();
        }
    }
}

impl Default for InputState {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
