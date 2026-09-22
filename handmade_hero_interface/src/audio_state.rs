use serde::{Deserialize, Serialize};
use uom::si::frequency::hertz;

use crate::units::si::{
    frequency::Frequency, information::Information, information_rate::InformationRate,
};

const SAMPLES_PER_SECOND: u32 = 48_000u32;
const DEFAULT_VOLUME: i16 = 3_000;

#[derive(Debug, Serialize, Deserialize)]
pub struct AudioState {
    channel_count: u16,
    channel_size: Information,
    volume: i16,
    buffer_size: Option<usize>,
    play_cursor: Option<usize>,
    write_cursor: Option<usize>,
}

impl AudioState {
    #[inline]
    #[must_use]
    pub fn new(channel_count: u16, channel_size: Information) -> Self {
        Self {
            channel_count,
            channel_size,
            volume: DEFAULT_VOLUME,
            buffer_size: None,
            play_cursor: None,
            write_cursor: None,
        }
    }

    #[inline]
    #[must_use]
    pub fn volume(&self) -> i16 {
        self.volume
    }

    #[inline]
    #[must_use]
    pub fn channel_count(&self) -> u16 {
        self.channel_count
    }

    #[inline]
    #[must_use]
    pub fn frequency(&self) -> Frequency {
        Frequency::new::<hertz>(SAMPLES_PER_SECOND)
    }

    #[inline]
    #[must_use]
    pub fn sample_size(&self) -> Information {
        u32::from(self.channel_count) * self.channel_size
    }

    #[inline]
    #[must_use]
    pub fn channel_size(&self) -> Information {
        self.channel_size
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
    pub fn buffer_sample_count(&self) -> Option<usize> {
        self.buffer_size
    }

    #[inline]
    pub fn set_buffer_sample_count(&mut self, value: usize) {
        self.buffer_size = Some(value);
    }

    #[inline]
    pub fn clear_buffer_size(&mut self) {
        self.buffer_size = None;
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
}
