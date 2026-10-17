use crate::collision_deltas::CollisionDeltas;
use crate::tile_map::TileMap;
use crate::tile_map_key::TileMapKey;
use crate::world_coordinate::WorldCoordinate;
use handmade_hero_interface::units::si::length::Length;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::hash_map::Entry;

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

    pub fn add_tile_map(&mut self, id: TileMapKey) -> (&mut TileMap, bool) {
        let entry = self.tile_maps.entry(id);
        match entry {
            Entry::Occupied(o) => (o.into_mut(), false),
            Entry::Vacant(v) => {
                let tile_map = TileMap::new(self.rows, self.columns);
                (v.insert(tile_map), true)
            }
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
        self.get_tile_map(coordinate.tile_map_key())
            .and_then(|m| m.get(coordinate.tile_y(), coordinate.tile_x()))
            .is_some_and(|t| t == 0)
    }

    #[inline]
    #[must_use]
    pub fn get_tile_map(&self, key: TileMapKey) -> Option<&TileMap> {
        self.tile_maps.get(&key)
    }

    #[inline]
    #[must_use]
    pub fn get_tile_map_mut(&mut self, key: TileMapKey) -> Option<&mut TileMap> {
        self.tile_maps.get_mut(&key)
    }

    pub fn ensure_tile_maps(
        &mut self,
        min: TileMapKey,
        max: TileMapKey,
        initializer: impl Fn(&mut Self, TileMapKey),
    ) {
        for y in min.y()..=max.y() {
            for x in min.x()..=max.x() {
                let key = TileMapKey::at_x_y(x, y);
                if let (_, true) = self.add_tile_map(key) {
                    initializer(self, key);
                }
            }
        }
    }
}
