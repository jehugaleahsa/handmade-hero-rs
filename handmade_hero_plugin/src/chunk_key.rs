use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, Serialize, Deserialize)]
pub struct ChunkKey {
    x: i16,
    y: i16,
    z: i16,
}

impl ChunkKey {
    #[inline]
    #[must_use]
    pub fn at_x_y(x: i16, y: i16) -> Self {
        Self::at_x_y_z(x, y, 0)
    }

    #[inline]
    #[must_use]
    pub fn at_x_y_z(x: i16, y: i16, z: i16) -> Self {
        Self { x, y, z }
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
    pub fn z(self) -> i16 {
        self.z
    }

    #[inline]
    #[must_use]
    #[allow(dead_code)]
    pub fn shifted(self, delta_x: i16, delta_y: i16, delta_z: i16) -> Self {
        let x = self.x.wrapping_add(delta_x);
        let y = self.y.wrapping_add(delta_y);
        let z = self.z.wrapping_add(delta_z);
        Self::at_x_y_z(x, y, z)
    }
}
