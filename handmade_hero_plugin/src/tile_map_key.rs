use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, Serialize, Deserialize)]
pub struct TileMapKey {
    x: i16,
    y: i16,
}

impl TileMapKey {
    #[inline]
    #[must_use]
    pub fn from_x_y(x: i16, y: i16) -> Self {
        Self { x, y }
    }

    #[inline]
    #[must_use]
    pub fn x(self) -> i16 {
        self.x
    }

    #[inline]
    #[must_use]
    pub fn y(self) -> i16 {
        self.y
    }
}
