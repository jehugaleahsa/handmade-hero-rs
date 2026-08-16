use serde::{Deserialize, Serialize};
use std::ops::{Index, IndexMut};

#[derive(Debug, Serialize, Deserialize)]
pub struct TileMap {
    tiles: Vec<u32>,
    columns: u16,
}

impl TileMap {
    #[inline]
    #[must_use]
    pub fn new(rows: u16, columns: u16) -> Self {
        let tiles = vec![0; usize::from(rows) * usize::from(columns)];
        Self { tiles, columns }
    }

    #[inline]
    #[must_use]
    pub fn get(&self, row: u16, column: u16) -> Option<u32> {
        self.tiles.get(self.offset_of(row, column)).copied()
    }

    #[inline]
    fn offset_of(&self, row: u16, column: u16) -> usize {
        usize::from(row) * usize::from(self.columns) + usize::from(column)
    }
}

impl Index<(u16, u16)> for TileMap {
    type Output = u32;

    #[inline]
    fn index(&self, index: (u16, u16)) -> &Self::Output {
        let (row, column) = index;
        &self.tiles[self.offset_of(row, column)]
    }
}

impl IndexMut<(u16, u16)> for TileMap {
    #[inline]
    fn index_mut(&mut self, index: (u16, u16)) -> &mut Self::Output {
        let (row, column) = index;
        let offset = self.offset_of(row, column);
        &mut self.tiles[offset]
    }
}
