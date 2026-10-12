use crate::collision_deltas::CollisionDeltas;
use crate::tile_map::TileMap;
use crate::tile_map_coordinate::TileMapCoordinate;
use crate::tile_map_key::TileMapKey;
use crate::world_coordinate::WorldCoordinate;
use handmade_hero_interface::point_2d::Point2d;
use handmade_hero_interface::units::si::length::Length;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use uom::si::ratio::ratio;

#[derive(Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct World {
    tile_maps: HashMap<TileMapKey, TileMap>,
    tile_size: Length,
    rows: u8,
    columns: u8,
}

impl World {
    pub const TILE_ROWS: u8 = 9;
    pub const TILE_COLUMNS: u8 = 17;

    #[inline]
    #[must_use]
    pub fn new(rows: u8, columns: u8, tile_size: Length) -> Self {
        let tile_maps = HashMap::new();
        Self {
            tile_maps,
            tile_size,
            rows,
            columns,
        }
    }

    #[inline]
    #[must_use]
    pub fn rows(&self) -> u8 {
        self.rows
    }

    #[inline]
    #[must_use]
    pub fn columns(&self) -> u8 {
        self.columns
    }

    #[inline]
    #[must_use]
    pub fn tile_size(&self) -> Length {
        self.tile_size
    }

    #[must_use]
    pub fn add_tile_map(&mut self, id: TileMapKey) -> &mut TileMap {
        let tile_map = TileMap::new(self.rows, self.columns);
        let entry = self.tile_maps.entry(id);
        match entry {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => v.insert(tile_map),
        }
    }

    /// Indicates whether all four corners of the collision box fall within a traversable tile.
    #[must_use]
    pub fn is_traversable(
        &self,
        coordinate: &WorldCoordinate,
        collision_deltas: CollisionDeltas,
    ) -> bool {
        let CollisionDeltas {
            top,
            bottom,
            left,
            right,
        } = collision_deltas;
        let bottom_left = coordinate.shifted(left, bottom, self.tile_size);
        if !self.is_traversable_coordinate(&bottom_left) {
            return false;
        }
        let bottom_right = coordinate.shifted(right, bottom, self.tile_size);
        if !self.is_traversable_coordinate(&bottom_right) {
            return false;
        }
        let top_left = coordinate.shifted(left, top, self.tile_size);
        if !self.is_traversable_coordinate(&top_left) {
            return false;
        }
        let top_right = coordinate.shifted(right, top, self.tile_size);
        self.is_traversable_coordinate(&top_right)
    }

    #[must_use]
    fn is_traversable_coordinate(&self, coordinate: &WorldCoordinate) -> bool {
        self.tile_maps
            .get(&coordinate.tile_map_key())
            .and_then(|m| m.get(coordinate.tile_y(), coordinate.tile_x()))
            .is_some_and(|t| t == 0)
    }

    #[inline]
    #[must_use]
    pub fn get_tile_map(&self, key: TileMapKey) -> Option<&TileMap> {
        self.tile_maps.get(&key)
    }

    #[must_use]
    pub fn convert_to_tile_map_coordinate(&self, x: Length, y: Length) -> TileMapCoordinate {
        let (tile_x, tile_y) = self.convert_to_tile_x_y(x, y);
        let tile_size = self.tile_size;
        let tile_x_offset = f32::from(tile_x) * tile_size;
        let tile_y_offset = f32::from(tile_y) * tile_size;
        let x = x - tile_x_offset;
        let y = y - tile_y_offset;
        let offset = Point2d::from_x_y(x, y);
        TileMapCoordinate::at_x_y_offset(tile_x, tile_y, offset)
    }

    #[must_use]
    #[expect(clippy::cast_sign_loss)]
    #[expect(clippy::cast_possible_truncation)]
    pub fn convert_to_tile_x_y(&self, x: Length, y: Length) -> (u8, u8) {
        let tile_size = self.tile_size;

        let tile_x = (x / tile_size).get::<ratio>() as u8;
        let tile_x = tile_x.min(self.columns - 1);

        let tile_y = (y / tile_size).get::<ratio>() as u8;
        let tile_y = tile_y.min(self.rows - 1);

        (tile_x, tile_y)
    }
}
