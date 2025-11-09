use crate::tile_map::TileMap;
use std::collections::HashMap;

#[derive(Debug)]
#[non_exhaustive]
pub struct World {
    pub rows: usize,
    pub columns: usize,
    pub tile_maps: HashMap<usize, TileMap>,
    pub current_tile_map_id: usize,
    pub tile_width: f32,
    pub tile_height: f32,
    pub x_offset: f32,
    pub y_offset: f32,
}

impl World {
    pub const TILE_ROWS: usize = 9;
    pub const TILE_COLUMNS: usize = 17;

    pub fn add_tile_map(&mut self, id: usize) -> &mut TileMap {
        let tile_map = TileMap::new(self.rows, self.columns);
        self.tile_maps.insert(id, tile_map);
        self.tile_maps
            .get_mut(&id)
            .expect("Could not retrieve a tile map that was just inserted")
    }
}
