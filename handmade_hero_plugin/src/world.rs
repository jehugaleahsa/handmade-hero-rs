use crate::chunk::Chunk;
use crate::chunk_key::ChunkKey;
use crate::tile_type::TileType;
use crate::world_coordinate::WorldCoordinate;
use handmade_hero_interface::units::si::length::Length;
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
    pub fn is_traversable(&self, coordinate: &WorldCoordinate) -> bool {
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
