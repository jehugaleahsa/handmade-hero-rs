use serde::{Deserialize, Serialize};
use uom::num::Zero;

use crate::{
    audio_state::AudioState, sample::Sample, stereo_sample::StereoSample, units::si::time::Time,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct GameState {
    frame_duration: Time,
    audio: AudioState,
}

impl GameState {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        let sample = StereoSample::default();
        let channel_count = sample.channel_count();
        let channel_size = sample.channel_size();
        Self {
            frame_duration: Time::zero(),
            audio: AudioState::new(channel_count, channel_size),
        }
    }

    #[inline]
    #[must_use]
    pub fn frame_duration(&self) -> Time {
        self.frame_duration
    }

    #[inline]
    pub fn set_frame_duration(&mut self, value: Time) {
        self.frame_duration = value;
    }

    #[inline]
    #[must_use]
    pub fn audio(&self) -> &AudioState {
        &self.audio
    }

    #[inline]
    #[must_use]
    pub fn audio_mut(&mut self) -> &mut AudioState {
        &mut self.audio
    }
}

impl Default for GameState {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
