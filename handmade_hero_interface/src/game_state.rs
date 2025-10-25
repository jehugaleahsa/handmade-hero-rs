use crate::stereo_sample::StereoSample;
use bincode::{Decode, Encode};

const BITS_PER_SAMPLE: u16 = 16;
pub const SAMPLES_PER_SECOND: u32 = 48_000u32;
pub const DEFAULT_VOLUME: i16 = 500;
#[allow(clippy::cast_possible_truncation)]
pub const BYTES_PER_SAMPLE: u32 = size_of::<StereoSample>() as u32;

#[derive(Debug, Encode, Decode)]
pub struct GameState {
    sound_hertz: u32,
    sound_theta: f32,
    sound_samples_per_seconds: u32,
    sound_bytes_per_sample: u32,
    sound_bits_per_sample: u16,
    sound_channel_count: u16,
    sound_volume: i16,
    width: u16,
    height: u16,
    rendered: bool,
}

impl GameState {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            sound_hertz: 256,
            sound_theta: 0f32,
            sound_samples_per_seconds: SAMPLES_PER_SECOND,
            sound_bytes_per_sample: BYTES_PER_SAMPLE,
            sound_bits_per_sample: BITS_PER_SAMPLE,
            sound_channel_count: StereoSample::CHANNEL_COUNT,
            sound_volume: DEFAULT_VOLUME,
            width: 0,
            height: 0,
            rendered: false,
        }
    }

    #[inline]
    #[must_use]
    pub fn sound_volume(&self) -> i16 {
        self.sound_volume
    }

    #[inline]
    #[must_use]
    pub fn sound_channel_count(&self) -> u16 {
        self.sound_channel_count
    }

    #[inline]
    #[must_use]
    pub fn sound_samples_per_second(&self) -> u32 {
        self.sound_samples_per_seconds
    }

    #[inline]
    #[must_use]
    pub fn sound_bits_per_sample(&self) -> u16 {
        self.sound_bits_per_sample
    }

    #[inline]
    #[must_use]
    pub fn sound_bytes_per_sample(&self) -> u32 {
        self.sound_bytes_per_sample
    }

    #[inline]
    #[must_use]
    pub fn sound_buffer_size(&self) -> u32 {
        self.sound_samples_per_seconds * self.sound_bytes_per_sample
    }

    #[inline]
    #[must_use]
    pub fn width(&self) -> u16 {
        self.width
    }

    #[inline]
    pub fn set_width(&mut self, value: u16) {
        self.width = value;
    }

    #[inline]
    #[must_use]
    pub fn height(&self) -> u16 {
        self.height
    }

    #[inline]
    pub fn set_height(&mut self, value: u16) {
        self.height = value;
    }

    #[inline]
    #[must_use]
    pub fn rendered(&self) -> bool {
        self.rendered
    }

    #[inline]
    pub fn set_rendered(&mut self) {
        self.rendered = true;
    }
}

impl Default for GameState {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
