use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default, Copy, Clone)]
pub struct Point2d<T> {
    x: T,
    y: T,
}

impl<T> Point2d<T>
where
    T: Copy,
{
    #[inline]
    #[must_use]
    pub fn from_x_y(x: T, y: T) -> Self {
        Self { x, y }
    }

    #[inline]
    #[must_use]
    pub fn x(self) -> T {
        self.x
    }

    #[inline]
    #[must_use]
    pub fn y(self) -> T {
        self.y
    }
}
