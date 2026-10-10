use handmade_hero_interface::{point_2d::Point2d, units::si::length::Length};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Copy, Clone)]
pub struct TileMapCoordinate {
    x: u16,
    y: u16,
    offset: Point2d<Length>,
}

impl TileMapCoordinate {
    #[inline]
    #[must_use]
    pub fn at_x_y(x: u16, y: u16) -> Self {
        Self {
            x,
            y,
            offset: Point2d::default(),
        }
    }

    #[inline]
    #[must_use]
    pub fn at_x_y_offset(x: u16, y: u16, offset: Point2d<Length>) -> Self {
        Self { x, y, offset }
    }

    #[inline]
    #[must_use]
    pub fn x(&self) -> u16 {
        self.x
    }

    #[inline]
    #[must_use]
    pub fn y(&self) -> u16 {
        self.y
    }

    #[inline]
    #[must_use]
    pub fn offset(&self) -> Point2d<Length> {
        self.offset
    }
}
