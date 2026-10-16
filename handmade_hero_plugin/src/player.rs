use crate::collision_deltas::CollisionDeltas;
use crate::dimensions::Dimensions;
use crate::tile_map_key::TileMapKey;
use crate::world::World;
use crate::world_coordinate::WorldCoordinate;
use handmade_hero_interface::units::si::length::Length;
use handmade_hero_interface::{color::Color, rectangle::Rectangle};
use serde::{Deserialize, Serialize};
use uom::num_traits::Zero;

#[derive(Debug, Serialize, Deserialize)]
pub struct Player {
    coordinate: WorldCoordinate,
    color: Color<f32>,
    running: bool,
}

impl Player {
    #[must_use]
    pub fn new(world: &World, tile_map_key: TileMapKey) -> Self {
        let coordinate = WorldCoordinate::new(world, tile_map_key);
        let color = Color::from(Color::from_rgb(0xFF, 0xFF, 0x00));
        Self {
            coordinate,
            color,
            running: false,
        }
    }

    #[must_use]
    pub fn render_bounds(tile_size: Length) -> Rectangle<Length> {
        let Dimensions { width, height } = Self::dimensions(tile_size);
        let x = -width / 2.0;
        Rectangle::new(Length::zero(), x, height, width)
    }

    #[must_use]
    pub fn collision_bound_deltas(tile_size: Length) -> CollisionDeltas {
        let Dimensions { width, height } = Self::dimensions(tile_size);
        let left_delta = -0.5 * width;
        let right_delta = 0.5 * width;
        let bound_height = 0.125 * height;
        CollisionDeltas {
            top: bound_height,
            bottom: Length::zero(),
            left: left_delta,
            right: right_delta,
        }
    }

    fn dimensions(tile_size: Length) -> Dimensions {
        let height = tile_size * 0.9f32;
        let width = tile_size * 0.75f32;
        Dimensions { width, height }
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

    #[inline]
    #[must_use]
    pub fn running(&self) -> bool {
        self.running
    }

    #[inline]
    pub fn toggle_running(&mut self) {
        self.running = !self.running;
    }
}
