use crate::stereo_sample::StereoSample;
use crate::units::si::frequency::Frequency;
use crate::units::si::information::Information;
use crate::units::si::information_rate::InformationRate;
use serde::{Deserialize, Serialize};
use uom::si::frequency::hertz;
use uom::si::information::byte;
use uom::si::time::second;

const BITS_PER_SAMPLE: u16 = 16;
const SAMPLES_PER_SECOND: u32 = 48_000u32;
const DEFAULT_VOLUME: i16 = 500;
#[expect(clippy::cast_possible_truncation)]
const BYTES_PER_SAMPLE: u32 = size_of::<StereoSample>() as u32;

#[derive(Debug, Serialize, Deserialize)]
pub struct SoundState {
    /// The pitch of the experimental tone generator.
    hertz: u32,
    /// The phase of the experimental tone generator.
    theta: f32,
    samples_per_second: Frequency,
    bytes_per_sample: Information,
    bits_per_sample: u16,
    channel_count: u16,
    volume: i16,
}

impl SoundState {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        let samples_per_second = Frequency::new::<hertz>(SAMPLES_PER_SECOND);
        let bytes_per_sample = Information::new::<byte>(BYTES_PER_SAMPLE);
        Self {
            hertz: 256,
            theta: 0f32,
            samples_per_second,
            bytes_per_sample,
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
    pub fn samples_per_second(&self) -> Frequency {
        self.samples_per_second
    }

    #[inline]
    #[must_use]
    pub fn bits_per_sample(&self) -> u16 {
        self.bits_per_sample
    }

    #[inline]
    #[must_use]
    pub fn bytes_per_sample(&self) -> Information {
        self.bytes_per_sample
    }

    /// The rate the audio device drains the sound buffer: one sample's worth of bytes for every
    /// cycle of the sample rate.
    #[inline]
    #[must_use]
    pub fn bytes_per_second(&self) -> InformationRate {
        (self.bytes_per_sample * self.samples_per_second).into()
    }

    /// The sound buffer holds a single second of audio.
    #[inline]
    #[must_use]
    pub fn buffer_size(&self) -> Information {
        let buffer_duration = uom::si::u32::Time::new::<second>(1);
        (self.bytes_per_second() * buffer_duration).into()
    }
}

impl Default for SoundState {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
