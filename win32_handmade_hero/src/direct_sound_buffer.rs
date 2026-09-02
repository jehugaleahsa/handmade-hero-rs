use crate::direct_sound::DirectSound;
use crate::direct_sound_buffer_lock_guard::DirectSoundBufferLockGuard;
use std::marker::PhantomData;
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
    pub fn play_looping(&self) -> Result<()> {
        unsafe { self.buffer.Play(0, 0, DSBPLAY_LOOPING) }
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

    pub fn lock(
        &self,
        write_offset: u32,
        write_length: u32,
    ) -> Result<DirectSoundBufferLockGuard<'_>> {
        DirectSoundBufferLockGuard::create(self, write_offset, write_length)
    }
}
