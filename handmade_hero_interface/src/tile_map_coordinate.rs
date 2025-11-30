use crate::point_2d::Point2d;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TileMapCoordinate {
    x: usize,
    y: usize,
    offset: Point2d<f32>,
}

impl TileMapCoordinate {
    #[inline]
    #[must_use]
    pub fn at_x_y(x: usize, y: usize) -> Self {
        Self {
            x,
            y,
            offset: Point2d::default(),
        }
    }

    #[inline]
    #[must_use]
    pub fn at_x_y_offset(x: usize, y: usize, offset: Point2d<f32>) -> Self {
        Self { x, y, offset }
    }

    #[inline]
    #[must_use]
    pub fn x(&self) -> usize {
        self.x
    }

    #[inline]
    #[must_use]
    pub fn y(&self) -> usize {
        self.y
    }

    #[inline]
    #[must_use]
    pub fn offset(&self) -> Point2d<f32> {
        self.offset
    }
}
