use crate::direct_sound_buffer::DirectSoundBuffer;
use std::ffi::c_void;
use std::ptr::null_mut;
use windows::core::Result;

pub struct DirectSoundBufferLockGuard<'ds> {
    buffer: &'ds DirectSoundBuffer<'ds>,
    region1: *mut c_void,
    region1_size: u32,
    region2: *mut c_void,
    region2_size: u32,
}

impl<'ds> DirectSoundBufferLockGuard<'ds> {
    pub(crate) fn create(
        buffer: &'ds DirectSoundBuffer<'ds>,
        write_offset: u32,
        bytes_to_write: u32,
    ) -> Result<Self> {
        let mut region1 = null_mut();
        let mut region1_size = 0;
        let mut region2 = null_mut();
        let mut region2_size = 0;
        unsafe {
            buffer.buffer.Lock(
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
