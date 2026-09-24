use super::direct_sound::DirectSound;
use super::direct_sound_buffer::DirectSoundBuffer;
use handmade_hero_interface::audio_format::AudioFormat;
use handmade_hero_interface::units::si::information::Information;
use uom::si::time::second;
use windows::core::Result as Win32Result;

/// The sound safety margin is this fraction of a frame of audio.
const SOUND_SAFETY_MARGIN_DIVISOR: u32 = 3;

/// The device buffer the platform fills, plus everything whose lifetime is tied to it.
#[derive(Debug)]
pub struct Win32SoundOutput<'ds> {
    buffer: DirectSoundBuffer<'ds>,
    frame_size: Information,
    write_offset: Option<u32>,
}

impl<'ds> Win32SoundOutput<'ds> {
    /// Opens a one-second buffer for `format` and starts it looping.
    pub fn start(
        direct_sound: &'ds DirectSound,
        format: AudioFormat,
        frame_size: Information,
    ) -> Win32Result<Self> {
        let duration = uom::si::u32::Time::new::<second>(1);
        let mut buffer = direct_sound.create_buffer(
            format.frequency(),
            format.channel_size(),
            format.channel_count(),
            duration,
        )?;
        buffer.play_looping()?;
        let output = Self {
            buffer,
            frame_size,
            write_offset: None,
        };
        Ok(output)
    }

    pub fn stop(&mut self) {
        self.buffer.stop().unwrap_or_default(); // Ignore errors
    }

    #[inline]
    #[must_use]
    pub fn buffer(&self) -> &DirectSoundBuffer<'ds> {
        &self.buffer
    }

    #[inline]
    pub fn buffer_mut(&mut self) -> &mut DirectSoundBuffer<'ds> {
        &mut self.buffer
    }

    #[inline]
    #[must_use]
    pub fn frame_size(&self) -> Information {
        self.frame_size
    }

    #[inline]
    #[must_use]
    pub fn safety_margin(&self) -> Information {
        self.frame_size / SOUND_SAFETY_MARGIN_DIVISOR
    }

    #[inline]
    #[must_use]
    pub fn write_offset(&self) -> Option<u32> {
        self.write_offset
    }

    #[inline]
    pub fn set_write_offset(&mut self, value: u32) {
        self.write_offset = Some(value);
    }

    /// Seeds the write offset from the device's write cursor if nothing has been written yet.
    ///
    /// Called at the end of a frame. After the buffer has played for a frame, the device knows
    /// how far its write cursor leads its play cursor.
    pub fn seed_write_offset(&mut self) {
        if self.write_offset.is_none()
            && let Ok((_, write_cursor)) = self.buffer.get_cursors()
        {
            self.write_offset = Some(write_cursor);
        }
    }
}
