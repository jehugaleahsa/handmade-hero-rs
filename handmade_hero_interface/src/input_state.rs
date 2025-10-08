use crate::controller_state::ControllerState;
use bincode::{Decode, Encode};

#[derive(Debug, Encode, Decode)]
pub struct InputState {
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
            keyboard,
            controllers: Vec::new(),
        }
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
}

impl Default for InputState {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
