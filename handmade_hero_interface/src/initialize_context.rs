use crate::{back_buffer::BackBuffer, game_state::GameState, plugin_state::PluginState};

/// What the game sees when it starts fresh. The window exists by now, so the back buffer has
/// its real dimensions. No sound has been written yet; the first frame has to run before the
/// audio cursors mean anything, so there's no sound buffer to hand over here.
#[derive(Debug)]
pub struct InitializeContext<'a> {
    pub game_state: &'a mut GameState,
    pub plugin_state: &'a mut dyn PluginState,
    pub back_buffer: &'a mut BackBuffer,
}
