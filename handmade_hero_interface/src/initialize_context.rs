use crate::{
    back_buffer::BackBuffer, game_state::GameState, plugin_state::PluginState,
    stereo_sample::StereoSample,
};

#[derive(Debug)]
pub struct InitializeContext<'a> {
    pub game_state: &'a mut GameState,
    pub plugin_state: &'a mut dyn PluginState,
    pub back_buffer: &'a mut BackBuffer,
    pub sound_buffer: Option<&'a mut [StereoSample]>,
}
