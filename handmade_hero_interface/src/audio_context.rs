use crate::game_state::GameState;
use crate::input_state::InputState;
use crate::stereo_sample::StereoSample;

#[derive(Debug)]
pub struct AudioContext<'a> {
    pub state: &'a mut GameState,
    pub input_state: &'a InputState,
    pub sound_buffer: &'a mut [StereoSample],
}
