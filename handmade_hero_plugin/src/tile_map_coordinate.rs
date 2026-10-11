use handmade_hero_interface::{point_2d::Point2d, units::si::length::Length};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Copy, Clone)]
pub struct TileMapCoordinate {
    offset: Point2d<u16>,
    tile_offset: Point2d<Length>,
}

impl TileMapCoordinate {
    #[inline]
    #[must_use]
    pub fn at_x_y(x: u16, y: u16) -> Self {
        let offset = Point2d::from_x_y(x, y);
        let tile_offset = Point2d::default();
        Self {
            offset,
            tile_offset,
        }
    }

    #[inline]
    #[must_use]
    pub fn at_x_y_offset(x: u16, y: u16, tile_offset: Point2d<Length>) -> Self {
        let offset = Point2d::from_x_y(x, y);
        Self {
            offset,
            tile_offset,
        }
    }

    #[inline]
    #[must_use]
    pub fn x(&self) -> u16 {
        self.offset.x()
    }

    #[inline]
    #[must_use]
    pub fn y(&self) -> u16 {
        self.offset.y()
    }

    #[inline]
    #[must_use]
    pub fn tile_offset(&self) -> Point2d<Length> {
        self.tile_offset
    }
}
