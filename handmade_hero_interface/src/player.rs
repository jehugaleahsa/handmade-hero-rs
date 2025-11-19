use crate::color::Color;
use crate::rectangle::Rectangle;
use crate::units::si::length::pixel;
use crate::world::TileMapKey;
use serde::{Deserialize, Serialize};
use uom::si::SI;

type Length = uom::si::length::Length<SI<f32>, f32>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Player {
    tile_map_key: TileMapKey,
    render_bounds: Rectangle<f32>,
    color: Color<f32>,
}

impl Player {
    #[must_use]
    pub fn new(tile_map_key: TileMapKey, tile_size: Length) -> Self {
        let render_bounds = Rectangle::new(
            0f32,
            0f32,
            (tile_size * 0.9f32).get::<pixel>(),
            (tile_size * 0.75f32).get::<pixel>(),
        );
        let color = Color::from(Color::from_rgb(0xFF, 0xFF, 0x00));
        Self {
            tile_map_key,
            render_bounds,
            color,
        }
    }

    #[inline]
    #[must_use]
    pub fn tile_map_key(&self) -> TileMapKey {
        self.tile_map_key
    }

    #[inline]
    pub fn set_tile_map_key(&mut self, key: TileMapKey) {
        self.tile_map_key = key;
    }

    #[inline]
    #[must_use]
    pub fn render_bounds(&self) -> Rectangle<f32> {
        self.render_bounds
    }

    #[inline]
    pub fn set_render_bounds(&mut self, bounds: Rectangle<f32>) {
        self.render_bounds = bounds;
    }

    #[must_use]
    #[inline]
    pub fn collision_bounds(&self) -> Rectangle<f32> {
        let bound_height = self.render_bounds.height() / 4f32;
        let bound_width = self.render_bounds.width();
        self.render_bounds.resized(bound_height, bound_width)
    }

    #[must_use]
    #[inline]
    pub fn color(&self) -> Color<f32> {
        self.color
    }
}
