use crate::game_state::GameState;
use crate::input_state::InputState;
use crate::plugin_state::PluginState;
use crate::sound_buffer::SoundBufferWindow;

#[derive(Debug)]
pub struct AudioContext<'a> {
    pub game_state: &'a mut GameState,
    pub plugin_state: &'a mut dyn PluginState,
    pub input_state: &'a InputState,
    /// The bytes to fill this frame.
    pub sound_buffer: SoundBufferWindow<'a>,
}
