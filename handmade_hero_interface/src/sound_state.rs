use crate::units::si::frequency::Frequency;
use crate::units::si::information::Information;
use crate::units::si::information_rate::InformationRate;
use serde::{Deserialize, Serialize};
use uom::si::frequency::hertz;

const SAMPLES_PER_SECOND: u32 = 48_000u32;
const DEFAULT_VOLUME: i16 = 500;

#[derive(Debug, Serialize, Deserialize)]
pub struct SoundState {
    channel_count: u16,
    channel_size: Information,
    /// The pitch of the experimental tone generator.
    hertz: u32,
    /// The phase of the experimental tone generator.
    theta: f32,
    volume: i16,
}

impl SoundState {
    #[inline]
    #[must_use]
    pub fn new(channel_count: u16, channel_size: Information) -> Self {
        Self {
            channel_count,
            channel_size,
            hertz: 256,
            theta: 0f32,
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
        (self.sample_size() * self.samples_per_second()).into()
    }
}
