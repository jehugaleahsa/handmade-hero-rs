use std::any::Any;

use crate::{back_buffer::BackBuffer, game_state::GameState, stereo_sample::StereoSample};

#[derive(Debug)]
pub struct InitializeContext<'a> {
    pub state: &'a mut GameState,
    pub back_buffer: &'a mut BackBuffer,
    pub sound_buffer: Option<&'a mut [StereoSample]>,
    pub plugin_game_state: Option<&'a mut dyn Any>,
    pub plugin_audio_state: Option<&'a mut dyn Any>,
}
