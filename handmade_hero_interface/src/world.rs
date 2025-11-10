use crate::point_2d::Point2d;
use crate::tile_map::TileMap;
use bincode::{Decode, Encode};
use std::collections::HashMap;
use std::collections::hash_map::Entry;

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, Encode, Decode)]
pub enum TileMapKey {
    South,
    Hub,
    West,
    East,
    North,
}

#[derive(Debug, Encode, Decode)]
#[non_exhaustive]
pub struct World {
    pub rows: usize,
    pub columns: usize,
    pub tile_maps: HashMap<TileMapKey, TileMap>,
    pub current_tile_map_id: TileMapKey,
    pub tile_width: f32,
    pub tile_height: f32,
    pub x_offset: f32,
    pub y_offset: f32,
}

impl World {
    pub const TILE_ROWS: usize = 9;
    pub const TILE_COLUMNS: usize = 17;

    pub fn add_tile_map(&mut self, id: TileMapKey) -> &mut TileMap {
        let tile_map = TileMap::new(self.rows, self.columns);
        let entry = self.tile_maps.entry(id);
        match entry {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => v.insert(tile_map),
        }
    }

    pub fn try_navigate(
        &mut self,
        x: f32,
        y: f32,
        player_width: f32,
        player_height: f32,
        width: f32,
        height: f32,
    ) -> Option<Point2d> {
        let player_middle = player_width / 2f32;
        if y <= 0f32 {
            self.try_navigate_north(x, y, height)
        } else if y >= height {
            self.try_navigate_south(x, player_height)
        } else if x - player_middle <= 0f32 {
            self.try_navigate_west(x, y, player_width, width)
        } else if x + player_middle >= width {
            self.try_navigate_east(y, player_width)
        } else {
            None
        }
    }

    fn try_navigate_north(&mut self, x: f32, y: f32, height: f32) -> Option<Point2d> {
        if let Some(new_tile_map_id) = self.find_north_tile_map() {
            self.current_tile_map_id = new_tile_map_id;
            let bottom = height + y;
            Some(Point2d::from_x_y(x, bottom))
        } else {
            None
        }
    }

    fn find_north_tile_map(&self) -> Option<TileMapKey> {
        match self.current_tile_map_id {
            TileMapKey::South => Some(TileMapKey::Hub),
            TileMapKey::Hub => Some(TileMapKey::North),
            _ => None,
        }
    }

    fn try_navigate_south(&mut self, x: f32, player_height: f32) -> Option<Point2d> {
        if let Some(new_tile_map_id) = self.find_south_tile_map() {
            self.current_tile_map_id = new_tile_map_id;
            Some(Point2d::from_x_y(x, player_height))
        } else {
            None
        }
    }

    fn find_south_tile_map(&self) -> Option<TileMapKey> {
        match self.current_tile_map_id {
            TileMapKey::Hub => Some(TileMapKey::South),
            TileMapKey::North => Some(TileMapKey::Hub),
            _ => None,
        }
    }

    fn try_navigate_west(
        &mut self,
        x: f32,
        y: f32,
        player_width: f32,
        width: f32,
    ) -> Option<Point2d> {
        if let Some(new_tile_map_id) = self.find_west_tile_map() {
            self.current_tile_map_id = new_tile_map_id;
            let right = width + (x - player_width);
            Some(Point2d::from_x_y(right, y))
        } else {
            None
        }
    }

    fn find_west_tile_map(&self) -> Option<TileMapKey> {
        match self.current_tile_map_id {
            TileMapKey::Hub => Some(TileMapKey::West),
            TileMapKey::East => Some(TileMapKey::Hub),
            _ => None,
        }
    }

    fn try_navigate_east(&mut self, y: f32, player_width: f32) -> Option<Point2d> {
        if let Some(new_tile_map_id) = self.find_east_tile_map() {
            self.current_tile_map_id = new_tile_map_id;
            Some(Point2d::from_x_y(player_width, y))
        } else {
            None
        }
    }

    fn find_east_tile_map(&self) -> Option<TileMapKey> {
        match self.current_tile_map_id {
            TileMapKey::Hub => Some(TileMapKey::East),
            TileMapKey::West => Some(TileMapKey::Hub),
            _ => None,
        }
    }

    #[must_use]
    pub fn is_traversable(&self, point: Point2d) -> bool {
        let Some(tile_map) = &self.tile_maps.get(&self.current_tile_map_id) else {
            return false;
        };
        #[allow(clippy::cast_sign_loss)]
        #[allow(clippy::cast_possible_truncation)]
        let tile_x = (point.x() / self.tile_width) as usize;
        let tile_x = usize::clamp(tile_x, 0, self.columns - 1);
        #[allow(clippy::cast_sign_loss)]
        #[allow(clippy::cast_possible_truncation)]
        let tile_y = (point.y() / self.tile_height) as usize;
        let tile_y = usize::clamp(tile_y, 0, self.rows - 1);
        let tile = tile_map.get(tile_y, tile_x);
        tile == 0
    }
}
