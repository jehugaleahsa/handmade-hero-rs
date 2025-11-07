#[derive(Debug)]
pub struct TileMap {
    rows: usize,
    columns: usize,
    tile_width: f32,
    tile_height: f32,
    x_offset: f32,
    y_offset: f32,
    tiles: Vec<u32>,
}

impl TileMap {
    #[inline]
    #[must_use]
    pub fn new(
        rows: usize,
        columns: usize,
        tile_width: f32,
        tile_height: f32,
        x_offset: f32,
        y_offset: f32,
    ) -> Self {
        let tiles = vec![0; rows * columns];
        Self {
            rows,
            columns,
            tile_width,
            tile_height,
            x_offset,
            y_offset,
            tiles,
        }
    }

    #[inline]
    #[must_use]
    pub fn rows(&self) -> usize {
        self.rows
    }

    #[inline]
    #[must_use]
    pub fn columns(&self) -> usize {
        self.columns
    }

    #[inline]
    #[must_use]
    pub fn tile_height(&self) -> f32 {
        self.tile_height
    }

    #[inline]
    #[must_use]
    pub fn tile_width(&self) -> f32 {
        self.tile_width
    }

    #[inline]
    #[must_use]
    pub fn x_offset(&self) -> f32 {
        self.x_offset
    }

    #[inline]
    #[must_use]
    pub fn y_offset(&self) -> f32 {
        self.y_offset
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
