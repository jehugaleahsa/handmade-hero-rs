use crate::units::si::frequency::Frequency;
use crate::units::si::information::Information;
use crate::units::si::information_rate::InformationRate;
use serde::{Deserialize, Serialize};
use uom::si::frequency::hertz;

const SAMPLES_PER_SECOND: u32 = 48_000u32;
const DEFAULT_VOLUME: i16 = 3_000;

#[derive(Debug, Serialize, Deserialize)]
pub struct SoundState {
    channel_count: u16,
    channel_size: Information,
    theta: u32,
    volume: i16,
}

impl SoundState {
    #[inline]
    #[must_use]
    pub fn new(channel_count: u16, channel_size: Information) -> Self {
        Self {
            channel_count,
            channel_size,
            theta: 0,
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
    pub fn theta(&self) -> u32 {
        self.theta
    }

    #[inline]
    pub fn set_theta(&mut self, value: u32) {
        self.theta = value;
    }
}
