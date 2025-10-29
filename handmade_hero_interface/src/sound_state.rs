use crate::stereo_sample::StereoSample;
use bincode::{Decode, Encode};

const BITS_PER_SAMPLE: u16 = 16;
pub const SAMPLES_PER_SECOND: u32 = 48_000u32;
pub const DEFAULT_VOLUME: i16 = 500;
#[allow(clippy::cast_possible_truncation)]
pub const BYTES_PER_SAMPLE: u32 = size_of::<StereoSample>() as u32;

#[derive(Debug, Encode, Decode)]
pub struct SoundState {
    hertz: u32,
    theta: f32,
    samples_per_seconds: u32,
    bytes_per_sample: u32,
    bits_per_sample: u16,
    channel_count: u16,
    volume: i16,
}

impl SoundState {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            hertz: 256,
            theta: 0f32,
            samples_per_seconds: SAMPLES_PER_SECOND,
            bytes_per_sample: BYTES_PER_SAMPLE,
            bits_per_sample: BITS_PER_SAMPLE,
            channel_count: StereoSample::CHANNEL_COUNT,
            volume: DEFAULT_VOLUME,
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
    pub fn samples_per_second(&self) -> u32 {
        self.samples_per_seconds
    }

    #[inline]
    #[must_use]
    pub fn bits_per_sample(&self) -> u16 {
        self.bits_per_sample
    }

    #[inline]
    #[must_use]
    pub fn bytes_per_sample(&self) -> u32 {
        self.bytes_per_sample
    }

    #[inline]
    #[must_use]
    pub fn buffer_size(&self) -> u32 {
        self.samples_per_seconds * self.bytes_per_sample
    }
}

impl Default for SoundState {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
