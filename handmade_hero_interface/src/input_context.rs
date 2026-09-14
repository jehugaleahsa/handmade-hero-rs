use crate::{game_state::GameState, input_state::InputState, plugin_state::PluginState};

#[derive(Debug)]
pub struct InputContext<'a> {
    pub input_state: &'a InputState,
    pub game_state: &'a mut GameState,
    pub plugin_state: &'a mut dyn PluginState,
}
