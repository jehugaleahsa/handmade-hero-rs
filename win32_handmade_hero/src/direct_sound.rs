use crate::direct_sound_buffer::DirectSoundBuffer;
use handmade_hero_interface::narrow_unsigned;
use handmade_hero_interface::units::si::frequency::Frequency;
use handmade_hero_interface::units::si::information::Information;
use uom::si::frequency::hertz;
use uom::si::information::{bit, byte};
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
        sample_rate: Frequency,
        channel_size: Information,
        channel_count: u16,
        duration: uom::si::u32::Time,
    ) -> Result<DirectSoundBuffer<'_>> {
        let primary_buffer_description = Self::create_primary_buffer_description();
        let primary_buffer = self.create_sound_buffer(&primary_buffer_description)?;
        let mut format = Self::create_buffer_format(sample_rate, channel_size, channel_count);
        unsafe {
            primary_buffer.SetFormat(&raw const format)?;
        }

        let buffer_size = Self::buffer_size(sample_rate, channel_size, channel_count, duration);
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

    fn create_buffer_format(
        sample_rate: Frequency,
        channel_size: Information,
        channel_count: u16,
    ) -> WAVEFORMATEX {
        let sample_rate_hz = sample_rate.get::<hertz>();
        let bits_per_channel = channel_size.get::<bit>();
        #[expect(clippy::cast_possible_truncation)]
        let bits_per_channel = bits_per_channel as u16;
        let bytes_per_sample = channel_size.get::<byte>() * u32::from(channel_count);
        #[expect(clippy::cast_possible_truncation)]
        let bytes_per_sample = bytes_per_sample as u16;
        let average_bytes_per_second = sample_rate_hz * u32::from(bytes_per_sample);
        WAVEFORMATEX {
            wFormatTag: narrow_unsigned!(WAVE_FORMAT_PCM => u16),
            nChannels: channel_count,
            nSamplesPerSec: sample_rate_hz,
            wBitsPerSample: bits_per_channel,
            nBlockAlign: bytes_per_sample,
            nAvgBytesPerSec: average_bytes_per_second,
            ..Default::default()
        }
    }

    fn buffer_size(
        sample_rate: Frequency,
        channel_size: Information,
        channel_count: u16,
        duration: uom::si::u32::Time,
    ) -> Information {
        let sample_size = channel_size * u32::from(channel_count);
        (sample_rate * duration * sample_size).into()
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
