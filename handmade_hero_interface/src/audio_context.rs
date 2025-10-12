use crate::game_state::GameState;
use crate::stereo_sample::StereoSample;

#[derive(Debug)]
pub struct AudioContext<'a> {
    state: &'a mut GameState,
    sound_buffer: &'a mut [StereoSample],
}

impl<'a> AudioContext<'a> {
    #[inline]
    #[must_use]
    pub fn new(state: &'a mut GameState, sound_buffer: &'a mut [StereoSample]) -> Self {
        Self {
            state,
            sound_buffer,
        }
    }

    #[inline]
    #[must_use]
    pub fn time_delta(&self) -> f32 {
        self.state.time_delta()
    }

    #[inline]
    #[must_use]
    pub fn theta(&self) -> f32 {
        self.state.sound_theta()
    }

    #[inline]
    pub fn advance_theta(&mut self, amount: f32) {
        self.state.advance_sound_theta(amount);
    }

    #[inline]
    #[must_use]
    pub fn volume(&self) -> i16 {
        self.state.sound_volume()
    }

    #[inline]
    #[must_use]
    pub fn sample_count(&self) -> usize {
        self.sound_buffer.len()
    }

    #[inline]
    pub fn set_sample(&mut self, index: usize, sample: StereoSample) {
        self.sound_buffer[index] = sample;
    }
}
