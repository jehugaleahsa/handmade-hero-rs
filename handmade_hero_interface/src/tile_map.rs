use serde::{Deserialize, Serialize};
use std::ops::{Index, IndexMut};

#[derive(Debug, Serialize, Deserialize)]
pub struct TileMap {
    tiles: Vec<u32>,
    columns: usize,
}

impl TileMap {
    #[inline]
    #[must_use]
    pub fn new(rows: usize, columns: usize) -> Self {
        let tiles = vec![0; rows * columns];
        Self { tiles, columns }
    }

    #[inline]
    #[must_use]
    pub fn get(&self, row: usize, column: usize) -> Option<u32> {
        let index = row * self.columns + column;
        self.tiles.get(index).copied()
    }
}

impl Index<(usize, usize)> for TileMap {
    type Output = u32;

    #[inline]
    fn index(&self, index: (usize, usize)) -> &Self::Output {
        let (row, column) = index;
        let index = row * self.columns + column;
        &self.tiles[index]
    }
}

impl IndexMut<(usize, usize)> for TileMap {
    #[inline]
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        let (row, column) = index;
        let index = row * self.columns + column;
        &mut self.tiles[index]
    }
}
