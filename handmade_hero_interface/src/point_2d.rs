use bincode::{Decode, Encode};

#[derive(Debug, Encode, Decode, Default, Copy, Clone)]
pub struct Point2d {
    x: f32,
    y: f32,
}

impl Point2d {
    #[inline]
    #[must_use]
    pub fn from_x_y(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[inline]
    #[must_use]
    pub fn x(self) -> f32 {
        self.x
    }

    #[inline]
    #[must_use]
    pub fn y(self) -> f32 {
        self.y
    }
}
