use crate::button_state::ButtonState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MouseState {
    x: u32,
    y: u32,
    vertical_wheel_delta: i32,
    horizontal_wheel_delta: i32,
    buttons: [ButtonState; MouseState::BUTTON_COUNT],
}

impl MouseState {
    const BUTTON_COUNT: usize = 5;
    const LEFT_BUTTON_INDEX: usize = 0;
    const MIDDLE_BUTTON_INDEX: usize = 1;
    const RIGHT_BUTTON_INDEX: usize = 2;

    #[inline]
    #[must_use]
    pub fn x(&self) -> u32 {
        self.x
    }

    #[inline]
    pub fn set_x(&mut self, value: u32) {
        self.x = value;
    }

    #[inline]
    #[must_use]
    pub fn y(&self) -> u32 {
        self.y
    }

    #[inline]
    pub fn set_y(&mut self, value: u32) {
        self.y = value;
    }

    #[inline]
    #[must_use]
    pub fn left(&self) -> &ButtonState {
        &self.buttons[Self::LEFT_BUTTON_INDEX]
    }

    #[inline]
    #[must_use]
    pub fn left_mut(&mut self) -> &mut ButtonState {
        &mut self.buttons[Self::LEFT_BUTTON_INDEX]
    }

    #[inline]
    #[must_use]
    pub fn middle(&self) -> &ButtonState {
        &self.buttons[Self::MIDDLE_BUTTON_INDEX]
    }

    #[inline]
    #[must_use]
    pub fn middle_mut(&mut self) -> &mut ButtonState {
        &mut self.buttons[Self::MIDDLE_BUTTON_INDEX]
    }

    #[inline]
    #[must_use]
    pub fn right(&self) -> &ButtonState {
        &self.buttons[Self::RIGHT_BUTTON_INDEX]
    }

    #[inline]
    #[must_use]
    pub fn right_mut(&mut self) -> &mut ButtonState {
        &mut self.buttons[Self::RIGHT_BUTTON_INDEX]
    }

    #[inline]
    #[must_use]
    pub fn vertical_wheel_delta(&self) -> i32 {
        self.vertical_wheel_delta
    }

    #[inline]
    pub fn set_vertical_wheel_delta(&mut self, value: i32) {
        self.vertical_wheel_delta = value;
    }

    #[inline]
    #[must_use]
    pub fn horizontal_wheel_delta(&self) -> i32 {
        self.horizontal_wheel_delta
    }

    #[inline]
    pub fn set_horizontal_wheel_delta(&mut self, value: i32) {
        self.horizontal_wheel_delta = value;
    }

    pub fn clear(&mut self) {
        for button in &mut self.buttons {
            button.reset_half_transition_count();
        }
        self.x = 0;
        self.y = 0;
        self.vertical_wheel_delta = 0;
        self.horizontal_wheel_delta = 0;
    }
}
