use serde::{Deserialize, Serialize};
use uom::si::frequency::hertz;

use crate::units::si::{
    frequency::Frequency, information::Information, information_rate::InformationRate,
};

const DEFAULT_SAMPLES_PER_SECOND: u32 = 48_000u32;
const DEFAULT_VOLUME: i16 = 3_000;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AudioState {
    channel_count: u16,
    channel_size: Information,
    frequency: Frequency,
    volume: i16,
    buffer_length: Option<Information>,
    play_cursor: Option<usize>,
    write_cursor: Option<usize>,
}

impl AudioState {
    #[inline]
    #[must_use]
    pub fn new(channel_count: u16, channel_size: Information) -> Self {
        let frequency = Frequency::new::<hertz>(DEFAULT_SAMPLES_PER_SECOND);
        Self {
            channel_count,
            channel_size,
            frequency,
            volume: DEFAULT_VOLUME,
            buffer_length: None,
            play_cursor: None,
            write_cursor: None,
        }
    }

    #[inline]
    #[must_use]
    pub fn channel_count(&self) -> u16 {
        self.channel_count
    }

    #[inline]
    pub fn set_channel_count(&mut self, value: u16) {
        self.channel_count = value;
    }

    #[inline]
    #[must_use]
    pub fn channel_size(&self) -> Information {
        self.channel_size
    }

    #[inline]
    pub fn set_channel_size(&mut self, value: Information) {
        self.channel_size = value;
    }

    #[inline]
    #[must_use]
    pub fn frequency(&self) -> Frequency {
        self.frequency
    }

    #[inline]
    pub fn set_frequency(&mut self, value: Frequency) {
        self.frequency = value;
    }

    #[inline]
    #[must_use]
    pub fn volume(&self) -> i16 {
        self.volume
    }

    #[inline]
    #[must_use]
    pub fn sample_size(&self) -> Information {
        u32::from(self.channel_count) * self.channel_size
    }

    /// The rate the audio device drains the sound buffer: one sample's worth of bytes for every
    /// cycle of the sample rate.
    #[inline]
    #[must_use]
    pub fn sample_rate(&self) -> InformationRate {
        (self.sample_size() * self.frequency()).into()
    }

    #[inline]
    #[must_use]
    pub fn buffer_length(&self) -> Option<Information> {
        self.buffer_length
    }

    #[inline]
    pub fn set_buffer_length(&mut self, value: Information) {
        self.buffer_length = Some(value);
    }

    #[inline]
    pub fn clear_buffer_length(&mut self) {
        self.buffer_length = None;
    }

    #[inline]
    #[must_use]
    pub fn play_cursor(&self) -> Option<usize> {
        self.play_cursor
    }

    #[inline]
    pub fn set_play_cursor(&mut self, value: usize) {
        self.play_cursor = Some(value);
    }

    #[inline]
    pub fn clear_play_cursor(&mut self) {
        self.play_cursor = None;
    }

    #[inline]
    #[must_use]
    pub fn write_cursor(&self) -> Option<usize> {
        self.write_cursor
    }

    #[inline]
    pub fn set_write_cursor(&mut self, value: usize) {
        self.write_cursor = Some(value);
    }

    #[inline]
    pub fn clear_write_cursor(&mut self) {
        self.write_cursor = None;
    }

    #[must_use]
    pub fn is_new_sound_buffer_needed(old: &AudioState, new: &AudioState) -> bool {
        old.channel_size != new.channel_size
            || old.channel_count != new.channel_count
            || old.frequency != new.frequency
    }
}
