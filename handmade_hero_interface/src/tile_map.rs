use bincode::{Decode, Encode};

#[derive(Debug, Encode, Decode)]
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
    pub fn get(&self, row: usize, column: usize) -> u32 {
        let index = row * self.columns + column;
        self.tiles[index]
    }

    #[inline]
    pub fn set(&mut self, row: usize, column: usize, value: u32) {
        let index = row * self.columns + column;
        self.tiles[index] = value;
    }
}
