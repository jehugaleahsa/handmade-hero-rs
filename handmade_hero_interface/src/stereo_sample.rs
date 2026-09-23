use uom::si::information::byte;

use crate::{narrow_unsigned, sample::Sample, units::si::information::Information};

#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
pub struct StereoSample {
    left: i16,
    right: i16,
}

// `Sample` promises the type has no padding bytes. `repr(C)` lays the two `i16` fields out back
// to back, so the struct is exactly two `i16` wide. This fails the build if a future edit
// changes that.
const _: () = assert!(
    size_of::<StereoSample>() == 2 * size_of::<i16>(),
    "StereoSample must have no padding to implement `Sample` soundly"
);

impl StereoSample {
    const CHANNEL_COUNT: u16 = 2;
    const CHANNEL_SIZE_IN_BYTES: u32 = narrow_unsigned!(size_of::<i16>() => u32);

    #[inline]
    #[must_use]
    pub fn from_left_right(left: i16, right: i16) -> Self {
        Self { left, right }
    }
}

// SAFETY: Both fields are `i16`, for which every bit pattern is valid, and the assertion above
// rules out padding.
unsafe impl Sample for StereoSample {
    #[inline]
    fn channel_count(&self) -> u16 {
        Self::CHANNEL_COUNT
    }

    #[inline]
    fn channel_size(&self) -> Information {
        Information::new::<byte>(Self::CHANNEL_SIZE_IN_BYTES)
    }
}
