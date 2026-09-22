use super::direct_sound_buffer::{DirectSoundBuffer, LockRegions};
use uom::si::u32::Information;
use windows::core::Result;

pub struct DirectSoundBufferLockGuard<'ds> {
    buffer: &'ds DirectSoundBuffer<'ds>,
    regions: LockRegions,
}

impl<'ds> DirectSoundBufferLockGuard<'ds> {
    pub(crate) fn create(
        buffer: &'ds DirectSoundBuffer<'ds>,
        write_offset: u32,
        write_size: Information,
    ) -> Result<Self> {
        let regions = buffer.lock_regions(write_offset, write_size)?;
        let guard = Self { buffer, regions };
        Ok(guard)
    }
}

impl DirectSoundBufferLockGuard<'_> {
    /// Copies raw sample bytes into the locked region, spilling into the second region when the
    /// lock wrapped around the end of the ring buffer.
    pub fn copy_from(&mut self, source: &[u8]) {
        let region1 = self.regions.region1.as_bytes_mut();
        Self::copy_sound_buffer(region1, source, 0);
        let region1_len = region1.len();

        let region2 = self.regions.region2.as_bytes_mut();
        Self::copy_sound_buffer(region2, source, region1_len);
    }

    fn copy_sound_buffer(destination: &mut [u8], source: &[u8], source_offset: usize) {
        let source_end = source_offset.saturating_add(destination.len());
        let source_slice = &source[source_offset..source_end];
        debug_assert_eq!(source_slice.len(), destination.len());
        destination.copy_from_slice(source_slice);
    }
}

impl Drop for DirectSoundBufferLockGuard<'_> {
    fn drop(&mut self) {
        self.buffer.unlock_regions(&self.regions).unwrap_or(()); // Ignore any errors
    }
}
