use crate::pixel::Pixel;
use std::ffi::c_void;

const BITS_PER_SAMPLE: u16 = 16;
const STEREO_CHANNEL_COUNT: u16 = 2; // Stereo
pub const SAMPLES_PER_SECOND: u32 = 48_000u32;
pub const DEFAULT_VOLUME: i16 = 3_000;
#[allow(clippy::cast_possible_truncation)]
pub const SOUND_BUFFER_SIZE: u32 =
    SAMPLES_PER_SECOND * size_of::<u16>() as u32 * STEREO_CHANNEL_COUNT as u32;
#[allow(clippy::cast_possible_truncation)]
pub const BYTES_PER_SAMPLE: u32 = (size_of::<i16>() * 2) as u32;

#[derive(Debug)]
struct SoundOutputState {
    hertz: u32,
    index: u32,
    theta: f32,
    wave_period: u32,
    latency: u32,
    samples_per_seconds: u32,
    bytes_per_sample: u32,
    bits_per_sample: u16,
    channel_count: u16,
    volume: i16,
}

#[derive(Debug)]
pub struct Application {
    x_offset: u32,
    y_offset: u32,
    bitmap_width: u32,
    bitmap_height: u32,
    sound_output_state: SoundOutputState,
}

impl Application {
    pub fn new() -> Self {
        Self {
            x_offset: 0,
            y_offset: 0,
            bitmap_width: 0,
            bitmap_height: 0,
            sound_output_state: SoundOutputState {
                hertz: 256,
                index: 0,
                theta: 0f32,
                wave_period: SAMPLES_PER_SECOND / 256,
                latency: SAMPLES_PER_SECOND / 15,
                samples_per_seconds: SAMPLES_PER_SECOND,
                bytes_per_sample: BYTES_PER_SAMPLE,
                bits_per_sample: BITS_PER_SAMPLE,
                channel_count: STEREO_CHANNEL_COUNT,
                volume: DEFAULT_VOLUME,
            },
        }
    }

    pub fn render(&self, bitmap_buffer: *mut c_void) {
        assert!(!bitmap_buffer.is_null());
        let width = self.bitmap_width;
        let height = self.bitmap_height;
        let mut pixel = bitmap_buffer.cast::<u32>();
        for y in 0..height {
            for x in 0..width {
                let color = Pixel::from_rgb(
                    0,
                    (y.wrapping_add(self.y_offset) & 0xFF) as u8,
                    (x.wrapping_add(self.x_offset) & 0xFF) as u8,
                );
                unsafe { *pixel = u32::from(color) };
                pixel = unsafe { pixel.add(1) };
            }
        }
    }

    pub fn resize_bitmap(&mut self, width: u32, height: u32) {
        self.bitmap_width = width;
        self.bitmap_height = height;
    }

    #[inline]
    #[must_use]
    pub fn bitmap_width(&self) -> u32 {
        self.bitmap_width
    }

    #[inline]
    #[must_use]
    pub fn bitmap_height(&self) -> u32 {
        self.bitmap_height
    }

    #[allow(clippy::cast_sign_loss)]
    pub fn shift_x(&mut self, shift: i16) {
        if shift.is_negative() {
            self.x_offset = self.x_offset.wrapping_sub(-shift as u32);
        } else {
            self.x_offset = self.x_offset.wrapping_add(shift as u32);
        }
    }

    #[allow(clippy::cast_sign_loss)]
    pub fn shift_y(&mut self, shift: i16) {
        if shift.is_negative() {
            self.y_offset = self.y_offset.wrapping_sub(-shift as u32);
        } else {
            self.y_offset = self.y_offset.wrapping_add(shift as u32);
        }
    }

    #[inline]
    #[must_use]
    pub fn sound_index(&self) -> u32 {
        self.sound_output_state.index
    }

    #[inline]
    pub fn increase_sound_index(&mut self, delta: u32) {
        self.sound_output_state.index = self.sound_output_state.index.wrapping_add(delta);
    }

    #[inline]
    #[must_use]
    pub fn sound_channel_count(&self) -> u16 {
        self.sound_output_state.channel_count
    }

    #[inline]
    #[must_use]
    pub fn sound_samples_per_second(&self) -> u32 {
        self.sound_output_state.samples_per_seconds
    }

    #[inline]
    #[must_use]
    pub fn sound_bits_per_sample(&self) -> u16 {
        self.sound_output_state.bits_per_sample
    }

    #[inline]
    #[must_use]
    pub fn sound_bytes_per_sample(&self) -> u32 {
        self.sound_output_state.bytes_per_sample
    }

    #[inline]
    #[must_use]
    pub fn sound_latency(&self) -> u32 {
        self.sound_output_state.latency
    }

    #[inline]
    pub fn set_sound_hertz(&mut self, hertz: u32) {
        self.sound_output_state.hertz = hertz;
        let wave_period = self.sound_output_state.samples_per_seconds / hertz;
        self.sound_output_state.wave_period = wave_period;
    }

    #[inline]
    #[must_use]
    pub fn sound_wave_period(&self) -> u32 {
        self.sound_output_state.wave_period
    }

    #[inline]
    #[must_use]
    pub fn sound_theta(&self) -> f32 {
        self.sound_output_state.theta
    }

    #[inline]
    pub fn increase_sound_theta(&mut self, delta: f32) {
        self.sound_output_state.theta += delta;
    }

    #[inline]
    #[must_use]
    pub fn sound_volume(&self) -> i16 {
        self.sound_output_state.volume
    }

    #[inline]
    #[must_use]
    pub fn sound_buffer_size(&self) -> u32 {
        SOUND_BUFFER_SIZE
    }
}
