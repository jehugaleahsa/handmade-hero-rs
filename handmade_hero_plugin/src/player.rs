use crate::tile_map_coordinate::TileMapCoordinate;
use crate::tile_map_key::TileMapKey;
use crate::world::World;
use crate::world_coordinate::WorldCoordinate;
use handmade_hero_interface::{
    color::Color,
    rectangle::Rectangle,
    units::si::length::{Length, pixel},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Player {
    coordinate: WorldCoordinate,
    height: f32,
    width: f32,
    color: Color<f32>,
}

impl Player {
    #[must_use]
    pub fn new(world: &World, tile_map_key: TileMapKey) -> Self {
        let tile_map_coordinate = TileMapCoordinate::at_x_y(0, 0);
        let coordinate = WorldCoordinate::new(world, tile_map_key, tile_map_coordinate);
        let color = Color::from(Color::from_rgb(0xFF, 0xFF, 0x00));
        Self {
            coordinate,
            height: 0.0,
            width: 0.0,
            color,
        }
    }

    pub fn resize(&mut self, tile_size: Length) {
        self.height = (tile_size * 0.9f32).get::<pixel>();
        self.width = (tile_size * 0.75f32).get::<pixel>();
    }

    #[inline]
    #[must_use]
    pub fn render_bounds(&self) -> Rectangle<f32> {
        let offset = self.coordinate.tile_offset();
        Rectangle::new(offset.y(), offset.x(), self.height, self.width)
    }

    #[inline]
    #[must_use]
    pub fn collision_bounds(&self) -> Rectangle<f32> {
        let offset = self.coordinate.tile_offset();
        let bound_height = self.height / 4f32;
        let bound_width = self.width;
        Rectangle::new(offset.y(), offset.x(), bound_height, bound_width)
    }

    #[must_use]
    #[inline]
    pub fn color(&self) -> Color<f32> {
        self.color
    }

    #[inline]
    #[must_use]
    pub fn tile_map_key(&self) -> TileMapKey {
        self.coordinate.tile_map_key()
    }

    #[inline]
    #[must_use]
    pub fn coordinate(&self) -> &WorldCoordinate {
        &self.coordinate
    }

    #[inline]
    pub fn set_coordinates(&mut self, coordinate: WorldCoordinate) {
        self.coordinate = coordinate;
    }
}
