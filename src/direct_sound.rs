use crate::direct_sound_buffer::DirectSoundBuffer;
use windows::core::{Error, Result};
use windows::Win32::Foundation::HWND;
use windows::Win32::Media::Audio::DirectSound::{
    DirectSoundCreate, IDirectSound, DSBCAPS_PRIMARYBUFFER, DSBUFFERDESC, DSSCL_PRIORITY,
};
use windows::Win32::Media::Audio::{WAVEFORMATEX, WAVE_FORMAT_PCM};

const NUMBER_OF_CHANNELS: u16 = 2; // Stereo
const BITS_PER_SAMPLE: u16 = 16;
#[allow(clippy::cast_possible_truncation)]
pub(crate) const BYTES_PER_SAMPLE: u32 = (size_of::<i16>() * 2) as u32;
pub(crate) const SAMPLES_PER_SECOND: u32 = 48_000u32;

pub(crate) const VOLUME: i16 = 3_000;
#[allow(clippy::cast_possible_truncation)]
pub(crate) const SOUND_BUFFER_SIZE: u32 =
    SAMPLES_PER_SECOND * size_of::<u16>() as u32 * NUMBER_OF_CHANNELS as u32;

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

    pub fn create_buffer(&self, buffer_size: u32) -> Result<DirectSoundBuffer<'_>> {
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
        let mut format = Self::create_buffer_format();
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

    fn create_buffer_format() -> WAVEFORMATEX {
        const BLOCK_ALIGN: u16 = NUMBER_OF_CHANNELS * BITS_PER_SAMPLE / 8;
        #[allow(clippy::cast_possible_truncation)]
        WAVEFORMATEX {
            wFormatTag: WAVE_FORMAT_PCM as u16,
            nChannels: NUMBER_OF_CHANNELS,
            nSamplesPerSec: SAMPLES_PER_SECOND,
            wBitsPerSample: BITS_PER_SAMPLE,
            nBlockAlign: BLOCK_ALIGN,
            nAvgBytesPerSec: SAMPLES_PER_SECOND * u32::from(BLOCK_ALIGN),
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
