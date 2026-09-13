use super::direct_sound_buffer::{DirectSoundBuffer, LockRegions};
use handmade_hero_interface::sample::Sample;
use std::marker::PhantomData;
use uom::si::u32::Information;
use windows::core::Result;

pub struct DirectSoundBufferLockGuard<'ds, T> {
    buffer: &'ds DirectSoundBuffer<'ds>,
    regions: LockRegions,
    _phantom: PhantomData<T>,
}

impl<'ds, T> DirectSoundBufferLockGuard<'ds, T> {
    pub(crate) fn create(
        buffer: &'ds DirectSoundBuffer<'ds>,
        write_offset: u32,
        write_size: Information,
    ) -> Result<Self> {
        let regions = buffer.lock_regions(write_offset, write_size)?;
        let guard = Self {
            buffer,
            regions,
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
        let region1 = self.regions.region1.as_slice_mut();
        Self::copy_sound_buffer(region1, source, 0);

        let region2 = self.regions.region2.as_slice_mut();
        Self::copy_sound_buffer(region2, source, region1.len());
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
        self.buffer.unlock_regions(&self.regions).unwrap_or(()); // Ignore any errors
    }
}
