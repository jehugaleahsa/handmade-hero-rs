use crate::direct_sound::DirectSound;
use crate::direct_sound_buffer_lock_guard::DirectSoundBufferLockGuard;
use std::marker::PhantomData;
use windows::core::Result;
use windows::Win32::Media::Audio::DirectSound::{IDirectSoundBuffer, DSBPLAY_LOOPING};

#[derive(Debug)]
pub struct DirectSoundBuffer<'ds> {
    pub(crate) buffer: IDirectSoundBuffer,
    direct_sound: PhantomData<&'ds DirectSound>,
}

impl<'ds> DirectSoundBuffer<'ds> {
    pub(crate) fn new(buffer: IDirectSoundBuffer) -> Self {
        Self {
            buffer,
            direct_sound: PhantomData {},
        }
    }

    pub fn play_looping(&self) -> Result<()> {
        unsafe { self.buffer.Play(0, 0, DSBPLAY_LOOPING) }
    }

    pub fn get_play_cursor(&self) -> Result<u32> {
        let mut play_cursor = 0u32;
        let mut write_cursor = 0u32;
        unsafe {
            self.buffer
                .GetCurrentPosition(Some(&raw mut play_cursor), Some(&raw mut write_cursor))?;
        }
        Ok(play_cursor)
    }

    pub fn lock(
        &self,
        write_offset: u32,
        write_length: u32,
    ) -> Result<DirectSoundBufferLockGuard<'_>> {
        DirectSoundBufferLockGuard::create(self, write_offset, write_length)
    }
}
