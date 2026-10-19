use std::fmt::Debug;

use crate::{back_buffer::BackBuffer, game_state::GameState, io::Io, plugin_state::PluginState};

/// What the game sees when it starts fresh. The window exists by now, so the back buffer has
/// its real dimensions. No sound has been written yet; the first frame has to run before the
/// audio cursors mean anything, so there's no sound buffer to hand over here.
pub struct InitializeContext<'a> {
    pub game_state: &'a mut GameState,
    pub plugin_state: &'a mut dyn PluginState,
    pub back_buffer: &'a mut BackBuffer,
    pub io: &'a dyn Io,
}

impl Debug for InitializeContext<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InitializeContext")
            .field("game_state", &self.game_state)
            .field("plugin_state", &self.plugin_state)
            .field("back_buffer", &self.back_buffer)
            .finish_non_exhaustive()
    }
}
