use crate::controller_state::ControllerState;

#[derive(Debug, Default)]
pub struct InputState {
    controllers: Vec<ControllerState>,
}

impl InputState {
    pub fn get_controller(&self, index: usize) -> Option<&ControllerState> {
        self.controllers.get(index)
    }

    pub fn get_or_insert_controller_mut(&mut self, index: usize) -> &mut ControllerState {
        if index >= self.controllers.len() {
            self.controllers
                .resize(index + 1, ControllerState::default());
        }
        &mut self.controllers[index]
    }
}
