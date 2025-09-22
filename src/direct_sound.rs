use crate::direct_sound_buffer::DirectSoundBuffer;
use windows::core::{Error, Result};
use windows::Win32::Foundation::HWND;
use windows::Win32::Media::Audio::DirectSound::{
    DirectSoundCreate, IDirectSound, DSBCAPS_PRIMARYBUFFER, DSBUFFERDESC, DSSCL_PRIORITY,
};
use windows::Win32::Media::Audio::{WAVEFORMATEX, WAVE_FORMAT_PCM};

#[derive(Debug)]
pub struct DirectSound {
    direct_sound: IDirectSound,
}

impl DirectSound {
    pub fn initialize(window_handle: HWND) -> Result<Self> {
        let mut direct_sound = None;
        unsafe { DirectSoundCreate(None, &raw mut direct_sound, None)? };
        let Some(direct_sound) = direct_sound else {
            return Err(Error::from_win32());
        };
        unsafe { direct_sound.SetCooperativeLevel(window_handle, DSSCL_PRIORITY)? };

        Ok(Self { direct_sound })
    }

    pub fn create_buffer(
        &self,
        channel_count: u16,
        samples_per_second: u32,
        sample_bits: u16,
        buffer_size: u32,
    ) -> Result<DirectSoundBuffer<'_>> {
        let primary_buffer_description = Self::create_primary_buffer_description();
        let mut primary_buffer = None;
        unsafe {
            self.direct_sound.CreateSoundBuffer(
                &raw const primary_buffer_description,
                &raw mut primary_buffer,
                None,
            )?;
        }
        let Some(ref primary_buffer) = primary_buffer else {
            return Err(Error::from_win32());
        };
        let mut format = Self::create_buffer_format(channel_count, samples_per_second, sample_bits);
        unsafe {
            primary_buffer.SetFormat(&raw const format)?;
        }

        let secondary_buffer_description =
            Self::create_secondary_buffer_description(buffer_size, &mut format);
        let mut secondary_buffer = None;
        unsafe {
            self.direct_sound.CreateSoundBuffer(
                &raw const secondary_buffer_description,
                &raw mut secondary_buffer,
                None,
            )?;
        }
        let Some(secondary_buffer) = secondary_buffer else {
            return Err(Error::from_win32());
        };

        Ok(DirectSoundBuffer::new(secondary_buffer, buffer_size))
    }

    fn create_primary_buffer_description() -> DSBUFFERDESC {
        let mut description = DSBUFFERDESC::default();
        #[allow(clippy::cast_possible_truncation)]
        let description_size = size_of::<DSBUFFERDESC>() as u32;
        description.dwSize = description_size;
        description.dwFlags = DSBCAPS_PRIMARYBUFFER;
        // NOTE: The buffer size for the primary buffer should be 0.
        description
    }

    fn create_buffer_format(
        channel_count: u16,
        samples_per_second: u32,
        bits_per_sample: u16,
    ) -> WAVEFORMATEX {
        let block_align = channel_count * bits_per_sample / 8;
        let average_bytes_per_second = samples_per_second * u32::from(block_align);
        #[allow(clippy::cast_possible_truncation)]
        let format = WAVE_FORMAT_PCM as u16;
        WAVEFORMATEX {
            wFormatTag: format,
            nChannels: channel_count,
            nSamplesPerSec: samples_per_second,
            wBitsPerSample: bits_per_sample,
            nBlockAlign: block_align,
            nAvgBytesPerSec: average_bytes_per_second,
            ..Default::default()
        }
    }

    fn create_secondary_buffer_description(
        buffer_size: u32,
        format: &mut WAVEFORMATEX,
    ) -> DSBUFFERDESC {
        let mut description = DSBUFFERDESC::default();
        #[allow(clippy::cast_possible_truncation)]
        let description_size = size_of::<DSBUFFERDESC>() as u32;
        description.dwSize = description_size;
        description.dwBufferBytes = buffer_size;
        description.lpwfxFormat = format;
        description
    }
}
