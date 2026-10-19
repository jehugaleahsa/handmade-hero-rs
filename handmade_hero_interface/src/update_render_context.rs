use std::fmt::Debug;

use crate::back_buffer::BackBuffer;
use crate::display::Display;
use crate::game_state::GameState;
use crate::input_state::InputState;
use crate::io::Io;
use crate::plugin_state::PluginState;

pub struct UpdateRenderContext<'a> {
    pub input_state: &'a InputState,
    pub game_state: &'a mut GameState,
    pub plugin_state: &'a mut dyn PluginState,
    pub buffer: &'a mut BackBuffer,
    pub display: Display<'a>,
    pub io: &'a dyn Io,
}

impl Debug for UpdateRenderContext<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UpdateRenderContext")
            .field("input_state", &self.input_state)
            .field("game_state", &self.game_state)
            .field("plugin_state", &self.plugin_state)
            .field("buffer", &self.buffer)
            .field("display", &self.display)
            .finish_non_exhaustive()
    }
}
