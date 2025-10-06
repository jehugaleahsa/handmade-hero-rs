use crate::controller_state::ControllerState;
use crate::input_state::InputState;
use crate::stereo_sample::StereoSample;
use std::f32::consts::PI;
use std::ops::Neg;

const BITS_PER_SAMPLE: u16 = 16;
pub const SAMPLES_PER_SECOND: u32 = 48_000u32;
pub const DEFAULT_VOLUME: i16 = 500;
#[allow(clippy::cast_possible_truncation)]
pub const BYTES_PER_SAMPLE: u32 = size_of::<StereoSample>() as u32;

#[derive(Debug)]
pub struct ApplicationState {
    x_offset: u16,
    y_offset: u16,
    sound_hertz: u32,
    sound_theta: f32,
    sound_samples_per_seconds: u32,
    sound_bytes_per_sample: u32,
    sound_bits_per_sample: u16,
    sound_channel_count: u16,
    sound_volume: i16,
    player_x: u16,
    player_y: u16,
    width: u16,
    height: u16,
    jump_time: f32,
}

impl ApplicationState {
    const FULL_CIRCLE: f32 = 2.0f32 * PI;
    pub const PLAYER_WIDTH: u16 = 10;
    pub const PLAYER_HEIGHT: u16 = 10;

    #[must_use]
    pub fn new() -> Self {
        Self {
            x_offset: 0,
            y_offset: 0,
            sound_hertz: 256,
            sound_theta: 0f32,
            sound_samples_per_seconds: SAMPLES_PER_SECOND,
            sound_bytes_per_sample: BYTES_PER_SAMPLE,
            sound_bits_per_sample: BITS_PER_SAMPLE,
            sound_channel_count: StereoSample::CHANNEL_COUNT,
            sound_volume: DEFAULT_VOLUME,
            player_x: 0,
            player_y: 0,
            width: 0,
            height: 0,
            jump_time: 0f32,
        }
    }

    #[inline]
    #[must_use]
    pub fn y_offset(&self) -> u16 {
        self.y_offset
    }

    #[inline]
    #[must_use]
    pub fn x_offset(&self) -> u16 {
        self.x_offset
    }

    #[inline]
    #[must_use]
    pub fn time_delta(&self) -> f32 {
        Self::FULL_CIRCLE / self.calculate_wave_period()
    }

    #[inline]
    #[must_use]
    pub fn sound_theta(&self) -> f32 {
        self.sound_theta
    }

    #[inline]
    pub fn advance_sound_theta(&mut self, amount: f32) {
        self.sound_theta += amount;
        if self.sound_theta >= Self::FULL_CIRCLE {
            self.sound_theta -= Self::FULL_CIRCLE;
        }
    }

    #[inline]
    #[must_use]
    pub fn sound_volume(&self) -> i16 {
        self.sound_volume
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

        if controller.a().ended_down() && self.jump_time == 0f32 {
            self.jump();
        } else if self.jump_time > 0f32 {
            self.jump_time = 0f32.max(self.jump_time - 0.033f32);
        }
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
        self.shift_x(shift_x.neg());
        self.set_player_x(
            self.player_x()
                .cast_signed()
                .wrapping_sub(shift_x)
                .unsigned_abs(),
        );
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
        self.shift_y(shift_y.neg());
        self.set_player_y(
            self.player_y()
                .cast_signed()
                .wrapping_sub(shift_y)
                .unsigned_abs(),
        );
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
    #[allow(clippy::cast_precision_loss)]
    pub fn calculate_wave_period(&self) -> f32 {
        self.sound_samples_per_seconds as f32 / self.sound_hertz as f32
    }

    #[inline]
    #[must_use]
    pub fn sound_buffer_size(&self) -> u32 {
        self.sound_samples_per_seconds * self.sound_bytes_per_sample
    }

    #[inline]
    #[must_use]
    pub fn player_x(&self) -> u16 {
        self.player_x
    }

    #[inline]
    pub fn set_player_x(&mut self, value: u16) {
        self.player_x = value.min(self.width - Self::PLAYER_WIDTH);
    }

    #[inline]
    #[must_use]
    pub fn player_y(&self) -> u16 {
        self.player_y
    }

    #[inline]
    pub fn set_player_y(&mut self, value: u16) {
        self.player_y = value.min(self.height - Self::PLAYER_HEIGHT);
    }

    #[inline]
    #[must_use]
    pub fn width(&self) -> u16 {
        self.width
    }

    #[inline]
    pub fn set_width(&mut self, value: u16) {
        self.width = value;
    }

    #[inline]
    #[must_use]
    pub fn height(&self) -> u16 {
        self.height
    }

    #[inline]
    pub fn set_height(&mut self, value: u16) {
        self.height = value;
    }

    #[inline]
    pub fn jump(&mut self) {
        if self.jump_time == 0f32 {
            self.jump_time += 1.0f32;
        }
    }

    #[inline]
    #[must_use]
    pub fn jump_time(&self) -> f32 {
        self.jump_time
    }
}

impl Default for ApplicationState {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
