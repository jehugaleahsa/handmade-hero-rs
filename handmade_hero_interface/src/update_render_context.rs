use crate::back_buffer::BackBuffer;
use crate::display::Display;
use crate::game_state::GameState;
use crate::input_state::InputState;
use crate::plugin_state::PluginState;

#[derive(Debug)]
pub struct UpdateRenderContext<'a> {
    pub input_state: &'a InputState,
    pub game_state: &'a mut GameState,
    pub plugin_state: &'a mut dyn PluginState,
    pub buffer: &'a mut BackBuffer,
    pub display: Display<'a>,
}
