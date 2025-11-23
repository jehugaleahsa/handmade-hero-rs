use crate::point_2d::Point2d;
use crate::rectangle::Rectangle;
use crate::tile_map::TileMap;
use crate::tile_map_coordinate::TileMapCoordinate;
use crate::tile_map_key::TileMapKey;
use crate::units::si::length::{Length, pixel};
use crate::world_coordinate::WorldCoordinate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::hash_map::Entry;

#[derive(Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct World {
    pub rows: usize,
    pub columns: usize,
    pub tile_maps: HashMap<TileMapKey, TileMap>,
    pub x_offset: Length,
    pub y_offset: Length,
    pub tile_size: Length,
}

impl World {
    pub const TILE_ROWS: usize = 9;
    pub const TILE_COLUMNS: usize = 17;

    #[must_use]
    #[inline]
    pub fn rows(&self) -> usize {
        self.rows
    }

    #[must_use]
    #[inline]
    pub fn columns(&self) -> usize {
        self.columns
    }

    #[must_use]
    #[inline]
    pub fn tile_size(&self) -> Length {
        self.tile_size
    }

    pub fn add_tile_map(&mut self, id: TileMapKey) -> &mut TileMap {
        let tile_map = TileMap::new(self.rows, self.columns);
        let entry = self.tile_maps.entry(id);
        match entry {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => v.insert(tile_map),
        }
    }

    /// Indicates whether all four corners of the rectangle fall within a traversable tile.
    #[must_use]
    pub fn is_traversable(&self, coordinate: &WorldCoordinate, bounds: Rectangle<f32>) -> bool {
        if !self.is_traversable_coordinate(coordinate) {
            return false;
        }
        let top_left = coordinate.shifted(0f32, bounds.height());
        if !self.is_traversable_coordinate(&top_left) {
            return false;
        }
        let bottom_right = coordinate.shifted(bounds.width(), 0f32);
        if !self.is_traversable_coordinate(&bottom_right) {
            return false;
        }
        let top_right = coordinate.shifted(bounds.width(), bounds.height());
        self.is_traversable_coordinate(&top_right)
    }

    #[must_use]
    fn is_traversable_coordinate(&self, coordinate: &WorldCoordinate) -> bool {
        self.tile_maps
            .get(&coordinate.tile_map_key())
            .and_then(|m| m.get(coordinate.tile_y(), coordinate.tile_x()))
            .is_some_and(|t| t == 0)
    }

    #[must_use]
    #[inline]
    pub fn get_tile_map(&self, key: TileMapKey) -> Option<&TileMap> {
        self.tile_maps.get(&key)
    }

    #[must_use]
    pub fn get_tile_map_coordinate(&self, point: Point2d<f32>) -> TileMapCoordinate {
        let (tile_x, tile_y) = self.get_tile_x_y(point);
        let tile_size = self.tile_size.get::<pixel>();
        #[allow(clippy::cast_precision_loss)]
        let (tile_x_offset, tile_y_offset) = (tile_x as f32 * tile_size, tile_y as f32 * tile_size);
        let offset = Point2d::from_x_y(point.x() - tile_x_offset, point.y() - tile_y_offset);
        TileMapCoordinate::at_x_y_offset(tile_x, tile_y, offset)
    }

    #[must_use]
    pub fn get_tile_x_y(&self, point: Point2d<f32>) -> (usize, usize) {
        let tile_size = self.tile_size.get::<pixel>();

        #[allow(clippy::cast_sign_loss)]
        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_precision_loss)]
        let tile_x = (point.x() / tile_size) as usize;
        let tile_x = usize::clamp(tile_x, 0, self.columns - 1);

        #[allow(clippy::cast_sign_loss)]
        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_precision_loss)]
        let tile_y = (point.y() / tile_size) as usize;
        let tile_y = usize::clamp(tile_y, 0, self.rows - 1);

        (tile_x, tile_y)
    }
}
