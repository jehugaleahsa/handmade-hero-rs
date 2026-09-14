use std::any::Any;

use crate::game_state::GameState;
use crate::input_state::InputState;
use crate::stereo_sample::StereoSample;

#[derive(Debug)]
pub struct AudioContext<'a> {
    pub state: &'a mut GameState,
    pub input_state: &'a InputState,
    pub sound_buffer: &'a mut [StereoSample],
    pub plugin_game_state: Option<&'a mut dyn Any>,
    pub plugin_audio_state: Option<&'a mut dyn Any>,
}
