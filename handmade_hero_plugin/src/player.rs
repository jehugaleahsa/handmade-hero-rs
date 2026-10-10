use crate::tile_map_coordinate::TileMapCoordinate;
use crate::tile_map_key::TileMapKey;
use crate::world::World;
use crate::world_coordinate::WorldCoordinate;
use handmade_hero_interface::{color::Color, rectangle::Rectangle, units::si::length::pixel};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Player {
    coordinate: WorldCoordinate,
    color: Color<f32>,
}

impl Player {
    #[must_use]
    pub fn new(world: &World, tile_map_key: TileMapKey) -> Self {
        let tile_map_coordinate = TileMapCoordinate::at_x_y(0, 0);
        let coordinate = WorldCoordinate::new(world, tile_map_key, tile_map_coordinate);
        let color = Color::from(Color::from_rgb(0xFF, 0xFF, 0x00));
        Self { coordinate, color }
    }

    #[inline]
    #[must_use]
    pub fn render_bounds(&self, world: &World) -> Rectangle<f32> {
        let tile_size = world.tile_size();
        let height_px = (tile_size * 0.9f32).get::<pixel>();
        let width_px = (tile_size * 0.75f32).get::<pixel>();
        let offset = self.coordinate.tile_offset();
        Rectangle::new(offset.y(), offset.x(), height_px, width_px)
    }

    #[inline]
    #[must_use]
    pub fn collision_bounds(&self, world: &World) -> Rectangle<f32> {
        let tile_size = world.tile_size();
        let height_px = (tile_size * 0.9f32).get::<pixel>();
        let width_px = (tile_size * 0.75f32).get::<pixel>();
        let offset = self.coordinate.tile_offset();
        let bound_height_px = height_px / 4.0;
        Rectangle::new(offset.y(), offset.x(), bound_height_px, width_px)
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
