use crate::direct_sound_buffer::DirectSoundBuffer;
use handmade_hero_interface::narrow_unsigned;
use handmade_hero_interface::sample::Sample;
use std::ffi::c_void;
use std::marker::PhantomData;
use std::ptr::null_mut;
use std::slice;
use uom::si::information::byte;
use uom::si::u32::Information;
use windows::core::Result;

pub struct DirectSoundBufferLockGuard<'ds, T> {
    buffer: &'ds DirectSoundBuffer<'ds>,
    region1: *mut c_void,
    region1_size: u32,
    region2: *mut c_void,
    region2_size: u32,
    _phantom: PhantomData<T>,
}

impl<'ds, T> DirectSoundBufferLockGuard<'ds, T> {
    pub(crate) fn create(
        buffer: &'ds DirectSoundBuffer<'ds>,
        write_offset: u32,
        write_size: Information,
    ) -> Result<Self> {
        let mut region1 = null_mut();
        let mut region1_size = 0;
        let mut region2 = null_mut();
        let mut region2_size = 0;
        unsafe {
            buffer.buffer.Lock(
                write_offset,
                write_size.get::<byte>(),
                &raw mut region1,
                &raw mut region1_size,
                Some(&raw mut region2),
                Some(&raw mut region2_size),
                0,
            )?;
        }
        let guard = Self {
            buffer,
            region1,
            region1_size,
            region2,
            region2_size,
            _phantom: PhantomData,
        };
        Ok(guard)
    }
}

impl<T> DirectSoundBufferLockGuard<'_, T>
where
    T: Sample,
{
    pub fn copy_from(&mut self, source: &[T]) {
        let region1 = Self::region_slice_mut(self.region1, self.region1_size);
        Self::copy_sound_buffer(region1, source, 0);

        let region2 = Self::region_slice_mut(self.region2, self.region2_size);
        Self::copy_sound_buffer(region2, source, region1.len());
    }

    fn region_slice_mut<'a>(region: *mut c_void, size: u32) -> &'a mut [T] {
        if region.is_null() || size == 0 {
            return &mut [];
        }
        let sample_count = size / narrow_unsigned!(size_of::<T>() => u32);
        let Ok(sample_count) = usize::try_from(sample_count) else {
            return &mut [];
        };
        let sample_pointer = region.cast::<T>();
        unsafe { slice::from_raw_parts_mut(sample_pointer, sample_count) }
    }

    fn copy_sound_buffer(destination: &mut [T], source: &[T], source_offset: usize) {
        let source_end = source_offset.saturating_add(destination.len());
        let source_slice = &source[source_offset..source_end];
        debug_assert_eq!(source_slice.len(), destination.len());
        destination.copy_from_slice(source_slice);
    }
}

impl<T> Drop for DirectSoundBufferLockGuard<'_, T> {
    fn drop(&mut self) {
        unsafe {
            self.buffer
                .buffer
                .Unlock(
                    self.region1,
                    self.region1_size,
                    Some(self.region2),
                    self.region2_size,
                )
                .unwrap_or(()); // Ignore any errors
        };
    }
}
