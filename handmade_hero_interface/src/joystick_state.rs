use bincode::{Decode, Encode};

#[derive(Debug, Default, Copy, Clone, Encode, Decode)]
pub struct JoystickState {
    x: f32,
    y: f32,
}

impl JoystickState {
    #[inline]
    #[must_use]
    pub fn x(self) -> f32 {
        self.x
    }

    #[inline]
    pub fn set_x(&mut self, value: f32) {
        self.x = value;
    }

    #[inline]
    #[must_use]
    pub fn y(self) -> f32 {
        self.y
    }

    #[inline]
    pub fn set_y(&mut self, value: f32) {
        self.y = value;
    }
}
