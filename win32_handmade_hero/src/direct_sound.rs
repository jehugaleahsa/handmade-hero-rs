use crate::direct_sound_buffer::DirectSoundBuffer;
use handmade_hero_interface::narrow_unsigned;
use handmade_hero_interface::stereo_sample::StereoSample;
use handmade_hero_interface::units::si::frequency::Frequency;
use handmade_hero_interface::units::si::information::Information;
use uom::si::frequency::hertz;
use uom::si::information::byte;
use windows::Win32::Foundation::HWND;
use windows::Win32::Media::Audio::DirectSound::{
    DSBCAPS_PRIMARYBUFFER, DSBUFFERDESC, DSSCL_PRIORITY, DirectSoundCreate, IDirectSound,
    IDirectSoundBuffer,
};
use windows::Win32::Media::Audio::{WAVE_FORMAT_PCM, WAVEFORMATEX};
use windows::core::{Error, Result};

#[derive(Debug)]
pub struct DirectSound {
    direct_sound: IDirectSound,
}

impl DirectSound {
    pub fn initialize(window_handle: HWND) -> Result<Self> {
        let direct_sound = Self::create_direct_sound()?;
        unsafe { direct_sound.SetCooperativeLevel(window_handle, DSSCL_PRIORITY)? };
        let result = Self { direct_sound };
        Ok(result)
    }

    fn create_direct_sound() -> Result<IDirectSound> {
        let mut direct_sound = None;
        unsafe { DirectSoundCreate(None, &raw mut direct_sound, None)? };
        direct_sound.ok_or_else(Error::from_thread)
    }

    pub fn create_buffer(
        &self,
        samples_per_second: Frequency,
        buffer_size: Information,
    ) -> Result<DirectSoundBuffer<'_>> {
        let primary_buffer_description = Self::create_primary_buffer_description();
        let primary_buffer = self.create_sound_buffer(&primary_buffer_description)?;
        let mut format = Self::create_buffer_format(samples_per_second);
        unsafe {
            primary_buffer.SetFormat(&raw const format)?;
        }

        let secondary_buffer_description =
            Self::create_secondary_buffer_description(buffer_size, &mut format);
        let secondary_buffer = self.create_sound_buffer(&secondary_buffer_description)?;

        let buffer = DirectSoundBuffer::new(primary_buffer, secondary_buffer, buffer_size);
        Ok(buffer)
    }

    fn create_primary_buffer_description() -> DSBUFFERDESC {
        // NOTE: The buffer size for the primary buffer should be 0!
        DSBUFFERDESC {
            dwSize: narrow_unsigned!(size_of::<DSBUFFERDESC>() => u32),
            dwFlags: DSBCAPS_PRIMARYBUFFER,
            ..DSBUFFERDESC::default()
        }
    }

    fn create_buffer_format(samples_per_second: Frequency) -> WAVEFORMATEX {
        const BITS_PER_BYTE: u16 = 8;

        let samples_per_second_hz = samples_per_second.get::<hertz>();
        let block_align = narrow_unsigned!(size_of::<StereoSample>() => u16);
        let bits_per_sample = block_align / StereoSample::CHANNEL_COUNT * BITS_PER_BYTE;
        let average_bytes_per_second = samples_per_second_hz * u32::from(block_align);
        WAVEFORMATEX {
            wFormatTag: narrow_unsigned!(WAVE_FORMAT_PCM => u16),
            nChannels: StereoSample::CHANNEL_COUNT,
            nSamplesPerSec: samples_per_second_hz,
            wBitsPerSample: bits_per_sample,
            nBlockAlign: block_align,
            nAvgBytesPerSec: average_bytes_per_second,
            ..Default::default()
        }
    }

    fn create_secondary_buffer_description(
        buffer_size: Information,
        format: &mut WAVEFORMATEX,
    ) -> DSBUFFERDESC {
        DSBUFFERDESC {
            dwSize: narrow_unsigned!(size_of::<DSBUFFERDESC>() => u32),
            dwBufferBytes: buffer_size.get::<byte>(),
            lpwfxFormat: format,
            ..DSBUFFERDESC::default()
        }
    }

    fn create_sound_buffer(&self, description: &DSBUFFERDESC) -> Result<IDirectSoundBuffer> {
        let mut buffer = None;
        unsafe {
            self.direct_sound
                .CreateSoundBuffer(&raw const *description, &raw mut buffer, None)?;
        }
        buffer.ok_or_else(Error::from_thread)
    }
}
