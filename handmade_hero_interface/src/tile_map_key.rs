use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, Serialize, Deserialize)]
pub struct TileMapKey {
    pub x: i16,
    pub y: i16,
}

impl TileMapKey {
    #[inline]
    #[must_use]
    pub fn x(&self) -> i16 {
        self.x
    }

    #[inline]
    #[must_use]
    pub fn y(&self) -> i16 {
        self.y
    }
}
