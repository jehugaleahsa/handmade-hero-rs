use std::slice;
use uom::si::information::byte;

use crate::{narrow_unsigned, sample::Sample, units::si::information::Information};

#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
pub struct StereoSample {
    left: i16,
    right: i16,
}

// `as_bytes` reads every byte of every sample, which is only sound if none of those bytes are
// padding. `repr(C)` lays the two `i16` fields out back to back, so the struct is exactly two
// `i16` wide. This fails the build if a future edit changes that.
const _: () = assert!(
    size_of::<StereoSample>() == 2 * size_of::<i16>(),
    "StereoSample must have no padding for `as_bytes` to be sound"
);

impl StereoSample {
    const CHANNEL_COUNT: u16 = 2;
    const CHANNEL_SIZE_IN_BYTES: u32 = narrow_unsigned!(size_of::<i16>() => u32);

    #[inline]
    #[must_use]
    pub fn from_left_right(left: i16, right: i16) -> Self {
        Self { left, right }
    }

    /// Views the samples as the raw bytes a sound device consumes.
    #[inline]
    #[must_use]
    pub fn as_bytes(samples: &[Self]) -> &[u8] {
        let byte_count = size_of_val(samples);
        let pointer = samples.as_ptr().cast::<u8>();
        // SAFETY: `pointer` comes from a live slice and `byte_count` is exactly that slice's
        // length times the size of one sample, so the range is in bounds. Every byte in the
        // range is initialized because the struct has no padding (checked at compile time above);
        // reading padding as `u8` would be undefined behavior.
        unsafe { slice::from_raw_parts(pointer, byte_count) }
    }
}

impl Sample for StereoSample {
    #[inline]
    fn channel_count(&self) -> u16 {
        Self::CHANNEL_COUNT
    }

    #[inline]
    fn channel_size(&self) -> Information {
        Information::new::<byte>(Self::CHANNEL_SIZE_IN_BYTES)
    }
}

#[cfg(test)]
mod tests {
    use super::StereoSample;

    #[test]
    fn test_as_bytes_matches_field_layout() {
        let samples = [
            StereoSample::from_left_right(1, -1),
            StereoSample::from_left_right(i16::MIN, i16::MAX),
        ];
        let bytes = StereoSample::as_bytes(&samples);

        let mut expected = Vec::new();
        for sample in &samples {
            expected.extend_from_slice(&sample.left.to_ne_bytes());
            expected.extend_from_slice(&sample.right.to_ne_bytes());
        }
        assert_eq!(bytes, expected.as_slice());
    }

    #[test]
    fn test_as_bytes_of_empty_slice_is_empty() {
        assert!(StereoSample::as_bytes(&[]).is_empty());
    }
}
