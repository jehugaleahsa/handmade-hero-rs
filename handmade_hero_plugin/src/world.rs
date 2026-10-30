use crate::chunk::Chunk;
use crate::chunk_key::ChunkKey;
use crate::collision_deltas::CollisionDeltas;
use crate::collision_result::CollisionResult;
use crate::tile_type::TileType;
use crate::world_coordinate::WorldCoordinate;
use handmade_hero_interface::units::si::length::Length;
use handmade_hero_interface::vector_2d::Vector2d;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::hash_map::Entry;

#[derive(Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct World {
    chunks: HashMap<ChunkKey, Chunk>,
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
        let chunks = HashMap::new();
        Self {
            chunks,
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

    pub fn add_chunk(&mut self, key: ChunkKey) -> (&mut Chunk, bool) {
        let entry = self.chunks.entry(key);
        match entry {
            Entry::Occupied(o) => (o.into_mut(), false),
            Entry::Vacant(v) => {
                let chunk = Chunk::new(self.rows, self.columns);
                (v.insert(chunk), true)
            }
        }
    }

    #[must_use]
    pub fn detect_collision(
        &self,
        coordinate: &WorldCoordinate,
        collision_deltas: CollisionDeltas,
    ) -> CollisionResult {
        let CollisionDeltas {
            top,
            bottom,
            left,
            right,
        } = collision_deltas;
        let top_left_collided = self.is_collision(coordinate, left, top);
        let top_right_collided = self.is_collision(coordinate, right, top);
        let bottom_left_collided = self.is_collision(coordinate, left, bottom);
        let bottom_right_collided = self.is_collision(coordinate, right, bottom);
        let top_any = top_left_collided || top_right_collided;
        let bottom_any = bottom_left_collided || bottom_right_collided;
        let left_any = top_left_collided || bottom_left_collided;
        let right_any = top_right_collided || bottom_right_collided;
        let top_collided = top_any && !bottom_any;
        let bottom_collided = bottom_any && !top_any;
        let left_collided = left_any && !right_any;
        let right_collided = right_any && !left_any;

        let collided = top_left_collided
            || top_right_collided
            || bottom_left_collided
            || bottom_right_collided;
        CollisionResult {
            collided,
            top_collided,
            bottom_collided,
            left_collided,
            right_collided,
        }
    }

    fn is_collision(&self, coordinate: &WorldCoordinate, delta_x: Length, delta_y: Length) -> bool {
        let corner = Vector2d::from_x_y(delta_x, delta_y);
        let corner = coordinate.shifted(corner, self.tile_size);
        !self.is_traversable_coordinate(&corner)
    }

    #[must_use]
    fn is_traversable_coordinate(&self, coordinate: &WorldCoordinate) -> bool {
        self.get_chunk(coordinate.chunk_key())
            .and_then(|m| m.get(coordinate.tile_y(), coordinate.tile_x()))
            .is_some_and(TileType::is_traversable)
    }

    #[inline]
    #[must_use]
    pub fn get_chunk(&self, key: ChunkKey) -> Option<&Chunk> {
        self.chunks.get(&key)
    }

    #[inline]
    #[must_use]
    pub fn get_chunk_mut(&mut self, key: ChunkKey) -> Option<&mut Chunk> {
        self.chunks.get_mut(&key)
    }

    pub fn ensure_chunks(
        &mut self,
        min: ChunkKey,
        max: ChunkKey,
        initializer: impl Fn(&mut Self, ChunkKey),
    ) {
        debug_assert_eq!(min.z(), max.z());
        let z = min.z();
        for y in min.y()..=max.y() {
            for x in min.x()..=max.x() {
                let key = ChunkKey::at_x_y_z(x, y, z);
                if let (_, true) = self.add_chunk(key) {
                    initializer(self, key);
                }
            }
        }
    }
}
