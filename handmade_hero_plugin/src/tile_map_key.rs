use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, Serialize, Deserialize)]
pub struct TileMapKey {
    x: i16,
    y: i16,
}

impl TileMapKey {
    #[inline]
    #[must_use]
    pub fn at_x_y(x: i16, y: i16) -> Self {
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

    #[inline]
    #[must_use]
    #[allow(dead_code)]
    pub fn shifted(self, delta_x: i16, delta_y: i16) -> Self {
        let x = self.x.wrapping_add(delta_x);
        let y = self.y.wrapping_add(delta_y);
        Self::at_x_y(x, y)
    }
}
