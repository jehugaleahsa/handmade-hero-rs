use serde::{Deserialize, Serialize};
use std::ops::{Index, IndexMut};

#[derive(Debug, Serialize, Deserialize)]
pub struct TileMap {
    tiles: Vec<u32>,
    columns: u8,
}

impl TileMap {
    #[inline]
    #[must_use]
    pub fn new(rows: u8, columns: u8) -> Self {
        let tiles = vec![0; usize::from(rows) * usize::from(columns)];
        Self { tiles, columns }
    }

    #[inline]
    #[must_use]
    pub fn get(&self, row: u8, column: u8) -> Option<u32> {
        let offset = self.offset_of(row, column);
        self.tiles.get(offset).copied()
    }

    #[inline]
    #[must_use]
    fn offset_of(&self, row: u8, column: u8) -> usize {
        usize::from(row) * usize::from(self.columns) + usize::from(column)
    }
}

impl Index<(u8, u8)> for TileMap {
    type Output = u32;

    #[inline]
    fn index(&self, index: (u8, u8)) -> &Self::Output {
        let (row, column) = index;
        let offset = self.offset_of(row, column);
        &self.tiles[offset]
    }
}

impl IndexMut<(u8, u8)> for TileMap {
    #[inline]
    fn index_mut(&mut self, index: (u8, u8)) -> &mut Self::Output {
        let (row, column) = index;
        let offset = self.offset_of(row, column);
        &mut self.tiles[offset]
    }
}
