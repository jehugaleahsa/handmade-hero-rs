use crate::controller_state::ControllerState;
use crate::input_state::InputState;
use crate::pixel::Pixel;
use crate::stereo_sample::StereoSample;
use std::f32::consts::PI;
use std::ops::Neg;

const BITS_PER_SAMPLE: u16 = 16;
pub const SAMPLES_PER_SECOND: u32 = 48_000u32;
pub const DEFAULT_VOLUME: i16 = 500;
#[allow(clippy::cast_possible_truncation)]
pub const BYTES_PER_SAMPLE: u32 = size_of::<StereoSample>() as u32;

#[derive(Debug)]
pub struct Application {
    x_offset: u16,
    y_offset: u16,
    bitmap_width: u16,
    bitmap_height: u16,
    sound_hertz: u32,
    sound_theta: f32,
    sound_latency: u32,
    sound_samples_per_seconds: u32,
    sound_bytes_per_sample: u32,
    sound_bits_per_sample: u16,
    sound_channel_count: u16,
    sound_volume: i16,
}

impl Application {
    pub fn new() -> Self {
        Self {
            x_offset: 0,
            y_offset: 0,
            bitmap_width: 0,
            bitmap_height: 0,
            sound_hertz: 256,
            sound_theta: 0f32,
            sound_latency: SAMPLES_PER_SECOND / 15,
            sound_samples_per_seconds: SAMPLES_PER_SECOND,
            sound_bytes_per_sample: BYTES_PER_SAMPLE,
            sound_bits_per_sample: BITS_PER_SAMPLE,
            sound_channel_count: StereoSample::CHANNEL_COUNT,
            sound_volume: DEFAULT_VOLUME,
        }
    }

    pub fn render(&self, bitmap_buffer: &mut [Pixel]) {
        let width = self.bitmap_width;
        let height = self.bitmap_height;
        let mut index = 0;
        for y in 0..height {
            for x in 0..width {
                let color = Pixel::from_rgb(
                    0,
                    (y.wrapping_add(self.y_offset) & 0xFF) as u8,
                    (x.wrapping_add(self.x_offset) & 0xFF) as u8,
                );
                bitmap_buffer[index] = color;
                index += 1;
            }
        }
    }

    pub fn write_sound(&mut self, sound_buffer: &mut [StereoSample]) {
        let time_delta = 2.0f32 * PI / self.calculate_wave_period();

        for sample in sound_buffer {
            let sine_value = self.sound_theta.sin();
            let volume = f32::from(self.sound_volume);
            #[allow(clippy::cast_possible_truncation)]
            let sample_value = (sine_value * volume) as i16;
            *sample = StereoSample::from_left_right(sample_value, sample_value);
            self.sound_theta += time_delta;
        }
    }

    pub fn resize_bitmap(&mut self, width: u16, height: u16) {
        self.bitmap_width = width;
        self.bitmap_height = height;
    }

    pub fn handle_input(&mut self, input_state: &InputState) {
        self.handle_keyboard_input(input_state.keyboard());
        for controller in input_state.controllers() {
            self.handle_controller_input(controller);
        }
    }

    fn handle_keyboard_input(&mut self, keyboard: &ControllerState) {
        if !keyboard.enabled() {
            return;
        }

        let shift_left = keyboard.left().half_transition_count();
        let shift_right = keyboard.right().half_transition_count();
        let shift_x = shift_right.cast_signed() - shift_left.cast_signed();
        self.shift_x(shift_x.neg().saturating_mul(5));

        let shift_up = keyboard.up().half_transition_count();
        let shift_down = keyboard.down().half_transition_count();
        let shift_y = shift_up.cast_signed() - shift_down.cast_signed();
        self.shift_y(shift_y.saturating_mul(5));
    }

    fn handle_controller_input(&mut self, controller: &ControllerState) {
        if !controller.enabled() {
            return;
        }
        self.shift_x_using_controller(controller);
        self.shift_y_using_controller(controller);
        self.set_hertz_using_controller(controller);
    }

    fn shift_x_using_controller(&mut self, controller: &ControllerState) {
        let shift_x_ratio = if controller.left().ended_down() {
            1.0f32
        } else if controller.right().ended_down() {
            -1.0f32
        } else {
            -(controller.left_joystick().x() + controller.right_joystick().x())
        };
        #[allow(clippy::cast_possible_truncation)]
        let shift_x = (shift_x_ratio * 5f32) as i16;
        self.shift_x(shift_x);
    }

    fn shift_y_using_controller(&mut self, controller: &ControllerState) {
        let shift_y_ratio = if controller.up().ended_down() {
            1.0f32
        } else if controller.down().ended_down() {
            -1.0f32
        } else {
            controller.left_joystick().y() + controller.right_joystick().y()
        };
        #[allow(clippy::cast_possible_truncation)]
        let shift_y = (shift_y_ratio * 5f32) as i16;
        self.shift_y(shift_y);
    }

    fn set_hertz_using_controller(&mut self, controller: &ControllerState) {
        let left_thumb_y_ratio = if controller.up().ended_down() {
            1.0f32
        } else if controller.down().ended_down() {
            -1.0f32
        } else {
            controller.left_joystick().y()
        };
        let right_thumb_y_ratio = controller.right_joystick().y();
        let thumb_y_ratio = f32::midpoint(left_thumb_y_ratio, right_thumb_y_ratio);
        #[allow(clippy::cast_sign_loss)]
        #[allow(clippy::cast_possible_truncation)]
        let hertz = (512.0f32 + (256.0f32 * thumb_y_ratio)) as u32;
        self.sound_hertz = hertz;
    }

    #[inline]
    #[must_use]
    pub fn bitmap_width(&self) -> u16 {
        self.bitmap_width
    }

    #[inline]
    #[must_use]
    pub fn bitmap_height(&self) -> u16 {
        self.bitmap_height
    }

    #[inline]
    pub fn shift_x(&mut self, shift: i16) {
        Self::shift(&mut self.x_offset, shift);
    }

    #[inline]
    pub fn shift_y(&mut self, shift: i16) {
        Self::shift(&mut self.y_offset, shift);
    }

    fn shift(offset: &mut u16, shift: i16) {
        // We need to handle if a negative value is the minimal possible value.
        *offset = if shift.is_negative() {
            offset.wrapping_sub(shift.unsigned_abs())
        } else {
            offset.wrapping_add(shift.unsigned_abs())
        }
    }

    #[inline]
    #[must_use]
    pub fn sound_channel_count(&self) -> u16 {
        self.sound_channel_count
    }

    #[inline]
    #[must_use]
    pub fn sound_samples_per_second(&self) -> u32 {
        self.sound_samples_per_seconds
    }

    #[inline]
    #[must_use]
    pub fn sound_bits_per_sample(&self) -> u16 {
        self.sound_bits_per_sample
    }

    #[inline]
    #[must_use]
    pub fn sound_bytes_per_sample(&self) -> u32 {
        self.sound_bytes_per_sample
    }

    #[inline]
    #[must_use]
    pub fn sound_latency(&self) -> u32 {
        self.sound_latency
    }

    #[inline]
    pub fn set_sound_latency(&mut self, value: u32) {
        self.sound_latency = value;
    }

    #[inline]
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn calculate_wave_period(&self) -> f32 {
        self.sound_samples_per_seconds as f32 / self.sound_hertz as f32
    }

    #[inline]
    #[must_use]
    pub fn sound_buffer_size(&self) -> u32 {
        self.sound_samples_per_seconds * self.sound_bytes_per_sample
    }
}
