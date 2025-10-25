use crate::game_state::GameState;
use crate::stereo_sample::StereoSample;

#[derive(Debug)]
pub struct AudioContext<'a> {
    pub state: &'a mut GameState,
    pub sound_buffer: &'a mut [StereoSample],
}
