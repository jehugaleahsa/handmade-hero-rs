use crate::button::Button;
use crate::button_state::ButtonState;
use crate::joystick_state::JoystickState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct ControllerState {
    a: ButtonState,
    b: ButtonState,
    x: ButtonState,
    y: ButtonState,
    left_shoulder: ButtonState,
    right_shoulder: ButtonState,
    up: ButtonState,
    down: ButtonState,
    left: ButtonState,
    right: ButtonState,
    start: ButtonState,
    back: ButtonState,
    left_joystick: JoystickState,
    right_joystick: JoystickState,
    left_trigger_ratio: f32,
    right_trigger_ratio: f32,
    enabled: bool,
}

impl ControllerState {
    #[inline]
    #[must_use]
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    #[inline]
    pub fn set_enabled(&mut self, value: bool) {
        self.enabled = value;
    }

    #[inline]
    #[must_use]
    pub fn a(&self) -> &ButtonState {
        &self.a
    }

    #[inline]
    #[must_use]
    pub fn a_mut(&mut self) -> &mut ButtonState {
        &mut self.a
    }

    #[inline]
    #[must_use]
    pub fn b(&self) -> &ButtonState {
        &self.b
    }

    #[inline]
    #[must_use]
    pub fn b_mut(&mut self) -> &mut ButtonState {
        &mut self.b
    }

    #[inline]
    #[must_use]
    pub fn x(&self) -> &ButtonState {
        &self.x
    }

    #[inline]
    #[must_use]
    pub fn x_mut(&mut self) -> &mut ButtonState {
        &mut self.x
    }

    #[inline]
    #[must_use]
    pub fn y(&self) -> &ButtonState {
        &self.y
    }

    #[inline]
    #[must_use]
    pub fn y_mut(&mut self) -> &mut ButtonState {
        &mut self.y
    }

    #[inline]
    #[must_use]
    pub fn start(&self) -> &ButtonState {
        &self.start
    }

    #[inline]
    #[must_use]
    pub fn start_mut(&mut self) -> &mut ButtonState {
        &mut self.start
    }

    #[inline]
    #[must_use]
    pub fn back(&self) -> &ButtonState {
        &self.back
    }

    #[inline]
    #[must_use]
    pub fn back_mut(&mut self) -> &mut ButtonState {
        &mut self.back
    }

    #[inline]
    #[must_use]
    pub fn up(&self) -> &ButtonState {
        &self.up
    }

    #[inline]
    #[must_use]
    pub fn up_mut(&mut self) -> &mut ButtonState {
        &mut self.up
    }

    #[inline]
    #[must_use]
    pub fn down(&self) -> &ButtonState {
        &self.down
    }

    #[inline]
    #[must_use]
    pub fn down_mut(&mut self) -> &mut ButtonState {
        &mut self.down
    }

    #[inline]
    #[must_use]
    pub fn left(&self) -> &ButtonState {
        &self.left
    }

    #[inline]
    #[must_use]
    pub fn left_mut(&mut self) -> &mut ButtonState {
        &mut self.left
    }

    #[inline]
    #[must_use]
    pub fn right(&self) -> &ButtonState {
        &self.right
    }

    #[inline]
    #[must_use]
    pub fn right_mut(&mut self) -> &mut ButtonState {
        &mut self.right
    }

    #[inline]
    #[must_use]
    pub fn left_shoulder(&self) -> &ButtonState {
        &self.left_shoulder
    }

    #[inline]
    #[must_use]
    pub fn left_shoulder_mut(&mut self) -> &mut ButtonState {
        &mut self.left_shoulder
    }

    #[inline]
    #[must_use]
    pub fn right_shoulder(&self) -> &ButtonState {
        &self.right_shoulder
    }

    #[inline]
    #[must_use]
    pub fn right_shoulder_mut(&mut self) -> &mut ButtonState {
        &mut self.right_shoulder
    }

    #[inline]
    #[must_use]
    pub fn left_trigger_ratio(&self) -> f32 {
        self.left_trigger_ratio
    }

    #[inline]
    pub fn set_left_trigger_ratio(&mut self, value: f32) {
        self.left_trigger_ratio = value;
    }

    #[inline]
    #[must_use]
    pub fn right_trigger_ratio(&self) -> f32 {
        self.right_trigger_ratio
    }

    #[inline]
    pub fn set_right_trigger_ratio(&mut self, value: f32) {
        self.right_trigger_ratio = value;
    }

    #[inline]
    #[must_use]
    pub fn left_joystick(&self) -> &JoystickState {
        &self.left_joystick
    }

    #[inline]
    #[must_use]
    pub fn left_joystick_mut(&mut self) -> &mut JoystickState {
        &mut self.left_joystick
    }

    #[inline]
    #[must_use]
    pub fn right_joystick(&self) -> &JoystickState {
        &self.right_joystick
    }

    #[inline]
    #[must_use]
    pub fn right_joystick_mut(&mut self) -> &mut JoystickState {
        &mut self.right_joystick
    }

    /// Looks a button up by name, so callers can be written once for every button.
    #[must_use]
    pub fn button(&self, button: Button) -> &ButtonState {
        match button {
            Button::A => &self.a,
            Button::B => &self.b,
            Button::X => &self.x,
            Button::Y => &self.y,
            Button::LeftShoulder => &self.left_shoulder,
            Button::RightShoulder => &self.right_shoulder,
            Button::Up => &self.up,
            Button::Down => &self.down,
            Button::Left => &self.left,
            Button::Right => &self.right,
            Button::Start => &self.start,
            Button::Back => &self.back,
        }
    }

    #[must_use]
    pub fn button_mut(&mut self, button: Button) -> &mut ButtonState {
        match button {
            Button::A => &mut self.a,
            Button::B => &mut self.b,
            Button::X => &mut self.x,
            Button::Y => &mut self.y,
            Button::LeftShoulder => &mut self.left_shoulder,
            Button::RightShoulder => &mut self.right_shoulder,
            Button::Up => &mut self.up,
            Button::Down => &mut self.down,
            Button::Left => &mut self.left,
            Button::Right => &mut self.right,
            Button::Start => &mut self.start,
            Button::Back => &mut self.back,
        }
    }

    pub fn reset_counts(&mut self) {
        self.a.reset_half_transition_count();
        self.b.reset_half_transition_count();
        self.x.reset_half_transition_count();
        self.y.reset_half_transition_count();
        self.left_shoulder.reset_half_transition_count();
        self.right_shoulder.reset_half_transition_count();
        self.up.reset_half_transition_count();
        self.down.reset_half_transition_count();
        self.left.reset_half_transition_count();
        self.right.reset_half_transition_count();
        self.start.reset_half_transition_count();
        self.back.reset_half_transition_count();
    }

    pub fn clear(&mut self) {
        self.a.clear();
        self.b.clear();
        self.x.clear();
        self.y.clear();
        self.left_shoulder.clear();
        self.right_shoulder.clear();
        self.up.clear();
        self.down.clear();
        self.left.clear();
        self.right.clear();
        self.start.clear();
        self.back.clear();
        self.left_trigger_ratio = 0.0;
        self.right_trigger_ratio = 0.0;
        self.left_joystick.clear();
        self.right_joystick.clear();
    }
}
