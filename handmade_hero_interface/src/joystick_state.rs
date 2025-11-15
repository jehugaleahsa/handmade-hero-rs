use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct JoystickState {
    x_ratio: f32,
    y_ratio: f32,
}

impl JoystickState {
    #[inline]
    #[must_use]
    pub fn x_ratio(self) -> f32 {
        self.x_ratio
    }

    #[inline]
    pub fn set_x_ratio(&mut self, value: f32) {
        self.x_ratio = value;
    }

    #[inline]
    #[must_use]
    pub fn y_ratio(self) -> f32 {
        self.y_ratio
    }

    #[inline]
    pub fn set_y_ratio(&mut self, value: f32) {
        self.y_ratio = value;
    }
}
