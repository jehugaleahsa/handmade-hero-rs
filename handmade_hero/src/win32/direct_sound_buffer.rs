use super::direct_sound::DirectSound;
use super::direct_sound_buffer_lock_guard::DirectSoundBufferLockGuard;
use core::slice;
use std::ffi::c_void;
use std::marker::PhantomData;
use uom::si::information::byte;
use uom::si::u32::Information;
use windows::Win32::Media::Audio::DirectSound::{DSBPLAY_LOOPING, IDirectSoundBuffer};
use windows::core::Result;

#[derive(Debug)]
pub struct DirectSoundBuffer<'ds> {
    _primary_buffer: IDirectSoundBuffer,
    pub(crate) buffer: IDirectSoundBuffer,
    length: Information,
    direct_sound: PhantomData<&'ds DirectSound>,
}

impl DirectSoundBuffer<'_> {
    #[inline]
    #[must_use]
    pub(crate) fn new(
        primary_buffer: IDirectSoundBuffer,
        buffer: IDirectSoundBuffer,
        length: Information,
    ) -> Self {
        Self {
            _primary_buffer: primary_buffer,
            buffer,
            length,
            direct_sound: PhantomData {},
        }
    }

    #[inline]
    #[must_use]
    pub fn length(&self) -> Information {
        self.length
    }

    #[inline]
    pub fn play_looping(&mut self) -> Result<()> {
        unsafe { self.buffer.Play(0, 0, DSBPLAY_LOOPING) }
    }

    #[inline]
    pub fn stop(&mut self) -> Result<()> {
        unsafe { self.buffer.Stop() }
    }

    pub fn get_cursors(&self) -> Result<(u32, u32)> {
        let mut play_cursor = 0u32;
        let mut write_cursor = 0u32;
        unsafe {
            self.buffer
                .GetCurrentPosition(Some(&raw mut play_cursor), Some(&raw mut write_cursor))?;
        }
        Ok((play_cursor, write_cursor))
    }

    pub(super) fn clear(&mut self) -> Result<()> {
        let mut regions = self.lock_regions(0, self.length)?;
        regions.region1.as_bytes_mut().fill(0);
        regions.region2.as_bytes_mut().fill(0);
        self.unlock_regions(&regions)?;
        Ok(())
    }

    #[inline]
    pub fn lock(
        &mut self,
        write_offset: u32,
        write_size: Information,
    ) -> Result<DirectSoundBufferLockGuard<'_>> {
        DirectSoundBufferLockGuard::create(self, write_offset, write_size)
    }

    pub(super) fn lock_regions(
        &self,
        write_offset: u32,
        write_size: Information,
    ) -> Result<LockRegions> {
        let mut regions = LockRegions::default();
        let first_region = &mut regions.region1;
        let second_region = &mut regions.region2;
        unsafe {
            self.buffer.Lock(
                write_offset,
                write_size.get::<byte>(),
                &raw mut first_region.offset,
                &raw mut first_region.size,
                Some(&raw mut second_region.offset),
                Some(&raw mut second_region.size),
                0,
            )?;
        }
        Ok(regions)
    }

    pub(super) fn unlock_regions(&self, regions: &LockRegions) -> Result<()> {
        unsafe {
            self.buffer.Unlock(
                regions.region1.offset,
                regions.region1.size,
                Some(regions.region2.offset),
                regions.region2.size,
            )
        }
    }
}

#[derive(Debug, Default)]
pub(super) struct LockRegion {
    pub offset: *mut c_void,
    pub size: u32,
}

impl LockRegion {
    /// Views the locked region as writable bytes.
    ///
    /// The pointer comes from `DirectSound`, so this is the one place the platform layer trusts
    /// memory it didn't allocate.
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        if self.offset.is_null() || self.size == 0 {
            return &mut [];
        }
        let Ok(size) = usize::try_from(self.size) else {
            return &mut [];
        };
        // SAFETY: DirectSound's `Lock` handed back `offset` and `size` as a pair describing one
        // contiguous region of the sound buffer, and that region stays valid and exclusively
        // ours until `Unlock`, which the owning guard calls in `Drop`. The slice borrows `self`
        // mutably, so the region can't be viewed twice at once.
        unsafe { slice::from_raw_parts_mut(self.offset.cast::<u8>(), size) }
    }
}

#[derive(Debug, Default)]
pub(super) struct LockRegions {
    pub region1: LockRegion,
    pub region2: LockRegion,
}
