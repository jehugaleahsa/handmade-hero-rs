use crate::controller_state::ControllerState;

#[derive(Debug)]
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

    #[inline]
    #[must_use]
    pub fn get_controller(&self, index: usize) -> Option<&ControllerState> {
        self.controllers.get(index)
    }

    #[must_use]
    pub fn get_or_insert_controller_mut(&mut self, index: usize) -> &mut ControllerState {
        if index >= self.controllers.len() {
            self.controllers.resize_with(index + 1, || {
                // We create a controller even if it's not connected, so we
                // don't enable the controller by default.
                let mut default_controller = ControllerState::default();
                default_controller.set_analog(true);
                default_controller
            });
        }
        &mut self.controllers[index]
    }
}

impl Default for InputState {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
