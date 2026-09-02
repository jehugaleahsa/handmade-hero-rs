use uom::si::information::byte;

use crate::{narrow_unsigned, sample::Sample, units::si::information::Information};

#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
pub struct StereoSample {
    left: i16,
    right: i16,
}

impl StereoSample {
    const CHANNEL_COUNT: u16 = 2;
    const CHANNEL_SIZE_IN_BYTES: u32 = narrow_unsigned!(size_of::<i16>() => u32);

    #[inline]
    #[must_use]
    pub fn from_left_right(left: i16, right: i16) -> Self {
        Self { left, right }
    }
}

impl Sample for StereoSample {
    fn channel_count(&self) -> u16 {
        Self::CHANNEL_COUNT
    }

    fn channel_size(&self) -> Information {
        Information::new::<byte>(Self::CHANNEL_SIZE_IN_BYTES)
    }
}
