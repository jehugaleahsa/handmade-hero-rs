use std::ffi::c_void;
use std::ptr::null_mut;
use windows::core::Result;
use windows::Win32::Media::Audio::DirectSound::IDirectSoundBuffer;

pub struct DirectSoundBufferLockGuard<'buffer> {
    buffer: &'buffer IDirectSoundBuffer,
    region1: *mut c_void,
    region1_size: u32,
    region2: *mut c_void,
    region2_size: u32,
}

impl<'buffer> DirectSoundBufferLockGuard<'buffer> {
    fn create(
        buffer: &'buffer IDirectSoundBuffer,
        write_offset: u32,
        bytes_to_write: u32,
    ) -> Result<Self> {
        let mut region1 = null_mut();
        let mut region1_size = 0;
        let mut region2 = null_mut();
        let mut region2_size = 0;
        unsafe {
            buffer.Lock(
                write_offset,
                bytes_to_write,
                &raw mut region1,
                &raw mut region1_size,
                Some(&raw mut region2),
                Some(&raw mut region2_size),
                0,
            )?;
        }
        Ok(Self {
            buffer,
            region1,
            region1_size,
            region2,
            region2_size,
        })
    }

    #[must_use]
    #[inline]
    pub fn region1(&self) -> *mut c_void {
        self.region1
    }

    #[must_use]
    #[inline]
    pub fn region1_size(&self) -> u32 {
        self.region1_size
    }

    #[must_use]
    #[inline]
    pub fn region2(&self) -> *mut c_void {
        self.region2
    }

    #[must_use]
    #[inline]
    pub fn region2_size(&self) -> u32 {
        self.region2_size
    }
}

impl Drop for DirectSoundBufferLockGuard<'_> {
    fn drop(&mut self) {
        unsafe {
            self.buffer
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

pub struct DirectSoundBufferLock<'buffer> {
    buffer: &'buffer IDirectSoundBuffer,
}

impl<'buffer> DirectSoundBufferLock<'buffer> {
    #[must_use]
    pub fn new(buffer: &'buffer IDirectSoundBuffer) -> Self {
        Self { buffer }
    }

    pub fn lock(
        &self,
        write_offset: u32,
        bytes_to_write: u32,
    ) -> Result<DirectSoundBufferLockGuard<'buffer>> {
        DirectSoundBufferLockGuard::create(self.buffer, write_offset, bytes_to_write)
    }
}
