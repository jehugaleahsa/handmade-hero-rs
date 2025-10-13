use crate::button_state::ButtonState;
use bincode::{Decode, Encode};

#[derive(Debug, Default, Encode, Decode)]
pub struct MouseState {
    x: u32,
    y: u32,
    left: ButtonState,
    middle: ButtonState,
    right: ButtonState,
}

impl MouseState {
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
        &self.left
    }

    #[inline]
    #[must_use]
    pub fn left_mut(&mut self) -> &mut ButtonState {
        &mut self.left
    }

    #[inline]
    #[must_use]
    pub fn middle(&self) -> &ButtonState {
        &self.middle
    }

    #[inline]
    #[must_use]
    pub fn middle_mut(&mut self) -> &mut ButtonState {
        &mut self.middle
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
}
