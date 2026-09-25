use serde::{Deserialize, Serialize};
use uom::si::frequency::hertz;

use crate::{
    audio_format::AudioFormat,
    units::si::{frequency::Frequency, information::Information},
};

const DEFAULT_SAMPLES_PER_SECOND: u32 = 48_000u32;
const DEFAULT_VOLUME: i16 = 3_000;

#[derive(Debug, Serialize, Deserialize)]
pub struct AudioState {
    channel_count: u16,
    channel_size: Information,
    frequency: Frequency,
    volume: i16,
    buffer_length: Option<Information>,
    flip_play_cursor: Option<usize>,
    flip_write_cursor: Option<usize>,
    output_play_cursor: Option<usize>,
    output_write_cursor: Option<usize>,
    output_write_offset: Option<usize>,
    output_write_length: Option<Information>,
    expected_frame_boundary: Option<usize>,
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
            flip_play_cursor: None,
            flip_write_cursor: None,
            output_play_cursor: None,
            output_write_cursor: None,
            output_write_offset: None,
            output_write_length: None,
            expected_frame_boundary: None,
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
    pub fn format(&self) -> AudioFormat {
        AudioFormat::new(self.channel_count, self.channel_size, self.frequency)
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
    pub fn flip_play_cursor(&self) -> Option<usize> {
        self.flip_play_cursor
    }

    #[inline]
    pub fn set_flip_play_cursor(&mut self, value: usize) {
        self.flip_play_cursor = Some(value);
    }

    #[inline]
    pub fn clear_flip_play_cursor(&mut self) {
        self.flip_play_cursor = None;
    }

    #[inline]
    #[must_use]
    pub fn flip_write_cursor(&self) -> Option<usize> {
        self.flip_write_cursor
    }

    #[inline]
    pub fn set_flip_write_cursor(&mut self, value: usize) {
        self.flip_write_cursor = Some(value);
    }

    #[inline]
    pub fn clear_flip_write_cursor(&mut self) {
        self.flip_write_cursor = None;
    }

    #[inline]
    #[must_use]
    pub fn output_play_cursor(&self) -> Option<usize> {
        self.output_play_cursor
    }

    #[inline]
    pub fn set_output_play_cursor(&mut self, value: usize) {
        self.output_play_cursor = Some(value);
    }

    #[inline]
    pub fn clear_output_play_cursor(&mut self) {
        self.output_play_cursor = None;
    }

    #[inline]
    #[must_use]
    pub fn output_write_cursor(&self) -> Option<usize> {
        self.output_write_cursor
    }

    #[inline]
    pub fn set_output_write_cursor(&mut self, value: usize) {
        self.output_write_cursor = Some(value);
    }

    #[inline]
    pub fn clear_output_write_cursor(&mut self) {
        self.output_write_cursor = None;
    }

    #[inline]
    #[must_use]
    pub fn output_write_offset(&self) -> Option<usize> {
        self.output_write_offset
    }

    #[inline]
    pub fn set_output_write_offset(&mut self, value: usize) {
        self.output_write_offset = Some(value);
    }

    #[inline]
    pub fn clear_output_write_offset(&mut self) {
        self.output_write_offset = None;
    }

    #[inline]
    #[must_use]
    pub fn output_write_length(&self) -> Option<Information> {
        self.output_write_length
    }

    #[inline]
    pub fn set_output_write_length(&mut self, value: Information) {
        self.output_write_length = Some(value);
    }

    #[inline]
    pub fn clear_output_write_length(&mut self) {
        self.output_write_length = None;
    }

    #[inline]
    #[must_use]
    pub fn expected_frame_boundary(&self) -> Option<usize> {
        self.expected_frame_boundary
    }

    #[inline]
    pub fn set_expected_frame_boundary(&mut self, value: usize) {
        self.expected_frame_boundary = Some(value);
    }

    #[inline]
    pub fn clear_expected_frame_boundary(&mut self) {
        self.expected_frame_boundary = None;
    }
}
