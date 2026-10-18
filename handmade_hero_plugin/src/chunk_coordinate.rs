use handmade_hero_interface::{point_2d::Point2d, units::si::length::Length};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize, Copy, Clone)]
pub struct ChunkCoordinate {
    offset: Point2d<u8>,
    tile_offset: Point2d<Length>,
}

impl ChunkCoordinate {
    #[inline]
    #[must_use]
    pub fn at_x_y_offset(x: u8, y: u8, tile_offset: Point2d<Length>) -> Self {
        let offset = Point2d::from_x_y(x, y);
        Self {
            offset,
            tile_offset,
        }
    }

    #[inline]
    #[must_use]
    pub fn x(&self) -> u8 {
        self.offset.x()
    }

    #[inline]
    #[must_use]
    pub fn y(&self) -> u8 {
        self.offset.y()
    }

    #[inline]
    #[must_use]
    pub fn tile_offset(&self) -> Point2d<Length> {
        self.tile_offset
    }
}
