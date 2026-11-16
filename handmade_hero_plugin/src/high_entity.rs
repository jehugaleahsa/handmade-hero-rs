use handmade_hero_interface::{units::si::length::Length, vector_2d::Vector2d};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct HighEntity {
    index: usize,
    position: Vector2d<Length>, // camera-relative
}

impl HighEntity {
    #[inline]
    #[must_use]
    pub fn new(index: usize, position: Vector2d<Length>) -> Self {
        Self { index, position }
    }

    #[inline]
    #[must_use]
    pub fn index(&self) -> usize {
        self.index
    }
}
