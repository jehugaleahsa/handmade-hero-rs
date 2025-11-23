use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, Serialize, Deserialize)]
pub struct TileMapKey {
    pub x: isize,
    pub y: isize,
}

impl TileMapKey {
    #[inline]
    #[must_use]
    pub fn x(&self) -> isize {
        self.x
    }

    #[inline]
    #[must_use]
    pub fn y(&self) -> isize {
        self.y
    }
}
