use crate::game_state::GameState;
use crate::input_state::InputState;
use crate::plugin_state::PluginState;
use crate::stereo_sample::StereoSample;

#[derive(Debug)]
pub struct AudioContext<'a> {
    pub state: &'a mut GameState,
    pub plugin_state: &'a mut dyn PluginState,
    pub input_state: &'a InputState,
    pub sound_buffer: &'a mut [StereoSample],
}
