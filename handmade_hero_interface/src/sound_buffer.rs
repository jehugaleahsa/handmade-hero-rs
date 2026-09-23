use std::slice;

use uom::si::information::byte;

use crate::{sample::Sample, units::si::information::Information};

/// Sample storage the platform layer owns without knowing the sample type.
///
/// The platform sizes it in bytes from the audio format, hands the game a [`SoundBufferWindow`]
/// over the bytes it wants filled each frame, and reads those bytes back to copy to the device.
/// Only the game decides what type the bytes are, by asking the window for a typed view.
///
/// The storage is a `Vec<u64>` rather than a `Vec<u8>`. A `u8` allocation only promises
/// alignment of one, so a typed view over it would fail whenever the allocator happened to hand
/// back an odd address. A `u64` allocation is always eight-byte aligned, so any sample type with
/// alignment up to eight views cleanly from the first byte. Viewing `u64` as bytes is trivially
/// sound: no padding, every bit pattern valid, alignment of one.
#[derive(Debug, Default)]
pub struct SoundBuffer {
    data: Option<Vec<u64>>,
}

impl SoundBuffer {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self { data: None }
    }

    /// Allocates the storage on first use and replaces it with a larger allocation if a later
    /// call asks for more.
    pub fn ensure_capacity(&mut self, capacity: Information) {
        let Ok(bytes) = usize::try_from(capacity.get::<byte>()) else {
            return; // 16-bit OS?
        };
        let words = bytes.div_ceil(size_of::<u64>());
        if let Some(ref mut existing) = self.data {
            if existing.len() < words {
                existing.resize(words, 0u64);
            }
        } else {
            self.data = Some(vec![0u64; words]);
        }
    }

    /// The first `length` bytes, ready for the game to fill, tagged with the sample size the
    /// audio format promises so the game's typed view can be checked against it.
    pub fn window(
        &mut self,
        length: Information,
        sample_size: Information,
    ) -> Option<SoundBufferWindow<'_>> {
        let length_bytes = usize::try_from(length.get::<byte>()).ok()?;
        let bytes = self.bytes_mut()?.get_mut(..length_bytes)?;
        let window = SoundBufferWindow {
            bytes,
            size: length,
            sample_size,
        };
        Some(window)
    }

    /// The first `length` bytes, for copying to the device after the game has filled them.
    #[must_use]
    pub fn bytes(&self, length: Information) -> Option<&[u8]> {
        let length = usize::try_from(length.get::<byte>()).ok()?;
        self.bytes_all()?.get(..length)
    }

    fn bytes_all(&self) -> Option<&[u8]> {
        let words = self.data.as_ref()?;
        let byte_count = words.len() * size_of::<u64>();
        // SAFETY: `words` is a live slice and `byte_count` is exactly its size in bytes, so the
        // range is in bounds. `u8` has alignment 1 and every bit pattern is a valid `u8`. `u64`
        // has no padding, so every byte is initialized. The output reborrows `words` for the
        // same lifetime, so it can't be written while this view exists.
        let raw = unsafe { slice::from_raw_parts(words.as_ptr().cast::<u8>(), byte_count) };
        Some(raw)
    }

    fn bytes_mut(&mut self) -> Option<&mut [u8]> {
        let words = self.data.as_mut()?;
        let byte_count = words.len() * size_of::<u64>();
        // SAFETY: as in `bytes_all`, plus the borrow is exclusive: the output reborrows `words`
        // mutably for the same lifetime, so no other view of the memory can exist alongside it.
        let raw = unsafe { slice::from_raw_parts_mut(words.as_mut_ptr().cast::<u8>(), byte_count) };
        Some(raw)
    }
}

/// The bytes the game fills with one frame of audio.
#[derive(Debug)]
pub struct SoundBufferWindow<'a> {
    bytes: &'a mut [u8],
    /// The slice's length as the platform's byte unit. Stored rather than computed.
    size: Information,
    sample_size: Information,
}

impl SoundBufferWindow<'_> {
    /// How many bytes the window covers.
    #[inline]
    #[must_use]
    pub fn size(&self) -> Information {
        self.size
    }

    /// Views the window as samples of `T`.
    ///
    /// Returns `None` when `T` doesn't fit the window: its size differs from the sample size the
    /// audio format promised, the window's length isn't a whole number of samples, or the
    /// window's start isn't aligned for `T`. The first catches a game asking for mono samples
    /// out of a stereo buffer. The second can't happen once the platform aligns its write size.
    /// The third can't happen while the storage is `u64`-backed and `T`'s alignment is at most
    /// eight, but checking it is what makes the safety argument below independent of how the
    /// storage happens to be allocated.
    pub fn as_samples_mut<T: Sample>(&mut self) -> Option<&mut [T]> {
        let sample_size = usize::try_from(self.sample_size.get::<byte>()).ok()?;
        if size_of::<T>() == 0 || size_of::<T>() != sample_size {
            return None;
        }
        if !self.bytes.len().is_multiple_of(size_of::<T>()) {
            return None;
        }
        let pointer = self.bytes.as_mut_ptr();
        if !pointer.addr().is_multiple_of(align_of::<T>()) {
            return None;
        }
        let count = self.bytes.len() / size_of::<T>();
        // SAFETY: `pointer` comes from a live, exclusively borrowed slice and `count` times the
        // size of `T` equals that slice's length, so the range is in bounds. The check above
        // guarantees `pointer` is aligned for `T`. Every byte is initialized: the storage was
        // zeroed on allocation, and `Sample` promises `T` has no padding, so no write through a
        // typed view can leave a byte uninitialized. `Sample` also promises every bit pattern is
        // a valid `T`, so reading the existing bytes as `T` is valid. The output reborrows
        // `self.bytes` mutably for the same lifetime, so the byte view and the typed view can't
        // coexist.
        let raw = unsafe { slice::from_raw_parts_mut(pointer.cast::<T>(), count) };
        Some(raw)
    }
}

#[cfg(test)]
mod tests {
    use super::SoundBuffer;
    use crate::{
        narrow_unsigned, stereo_sample::StereoSample, units::si::information::Information,
    };
    use uom::si::information::byte;

    const CHANNEL_SIZE: u32 = narrow_unsigned!(size_of::<i16>() => u32);

    fn bytes(count: u32) -> Information {
        Information::new::<byte>(count)
    }

    fn stereo_size() -> Information {
        bytes(2 * CHANNEL_SIZE)
    }

    #[test]
    fn test_window_before_allocation_is_none() {
        let mut buffer = SoundBuffer::new();
        assert!(buffer.window(bytes(4), stereo_size()).is_none());
        assert!(buffer.bytes(bytes(4)).is_none());
    }

    #[test]
    fn test_window_past_capacity_is_none() {
        let mut buffer = SoundBuffer::new();
        buffer.ensure_capacity(bytes(8));
        assert!(buffer.window(bytes(8), stereo_size()).is_some());
        assert!(buffer.window(bytes(9), stereo_size()).is_none());
    }

    #[test]
    fn test_typed_view_round_trips_through_bytes() {
        let mut buffer = SoundBuffer::new();
        buffer.ensure_capacity(bytes(64));

        let samples = [
            StereoSample::from_left_right(1, -1),
            StereoSample::from_left_right(i16::MIN, i16::MAX),
        ];
        {
            let mut window = buffer
                .window(bytes(8), stereo_size())
                .expect("window fits the allocation");
            let view = window
                .as_samples_mut::<StereoSample>()
                .expect("stereo matches the sample size");
            assert_eq!(view.len(), samples.len());
            view.copy_from_slice(&samples);
        }

        let mut expected = Vec::new();
        for (left, right) in [(1i16, -1i16), (i16::MIN, i16::MAX)] {
            expected.extend_from_slice(&left.to_ne_bytes());
            expected.extend_from_slice(&right.to_ne_bytes());
        }
        let actual = buffer.bytes(bytes(8)).expect("bytes fit the allocation");
        assert_eq!(actual, expected.as_slice());
    }

    #[test]
    fn test_typed_view_rejects_mismatched_sample_size() {
        let mut buffer = SoundBuffer::new();
        buffer.ensure_capacity(bytes(8));
        let mono_size = bytes(CHANNEL_SIZE);
        let mut window = buffer
            .window(bytes(8), mono_size)
            .expect("window fits the allocation");
        assert!(window.as_samples_mut::<StereoSample>().is_none());
    }

    #[test]
    fn test_typed_view_rejects_partial_sample() {
        let mut buffer = SoundBuffer::new();
        buffer.ensure_capacity(bytes(8));
        let mut window = buffer
            .window(bytes(6), stereo_size())
            .expect("window fits the allocation");
        assert!(window.as_samples_mut::<StereoSample>().is_none());
    }

    #[test]
    fn test_ensure_capacity_keeps_a_large_enough_allocation() {
        let mut buffer = SoundBuffer::new();
        buffer.ensure_capacity(bytes(16));
        {
            let mut window = buffer
                .window(bytes(4), stereo_size())
                .expect("window fits the allocation");
            let view = window
                .as_samples_mut::<StereoSample>()
                .expect("stereo matches the sample size");
            view[0] = StereoSample::from_left_right(7, 7);
        }
        buffer.ensure_capacity(bytes(8));
        let actual = buffer.bytes(bytes(4)).expect("bytes fit the allocation");
        let mut expected = Vec::new();
        expected.extend_from_slice(&7i16.to_ne_bytes());
        expected.extend_from_slice(&7i16.to_ne_bytes());
        assert_eq!(actual, expected.as_slice());
    }
}
