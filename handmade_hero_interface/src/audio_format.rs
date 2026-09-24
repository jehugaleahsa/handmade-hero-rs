use crate::units::si::{
    frequency::Frequency, information::Information, information_rate::InformationRate,
};

/// The shape of a PCM stream: how many channels, how wide each channel is, and how many samples
/// play per second.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AudioFormat {
    channel_count: u16,
    channel_size: Information,
    frequency: Frequency,
}

impl AudioFormat {
    #[inline]
    #[must_use]
    pub fn new(channel_count: u16, channel_size: Information, frequency: Frequency) -> Self {
        Self {
            channel_count,
            channel_size,
            frequency,
        }
    }

    #[inline]
    #[must_use]
    pub fn channel_count(&self) -> u16 {
        self.channel_count
    }

    #[inline]
    #[must_use]
    pub fn channel_size(&self) -> Information {
        self.channel_size
    }

    #[inline]
    #[must_use]
    pub fn frequency(&self) -> Frequency {
        self.frequency
    }

    /// Bytes in one sample across every channel.
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
        (self.sample_size() * self.frequency).into()
    }
}
