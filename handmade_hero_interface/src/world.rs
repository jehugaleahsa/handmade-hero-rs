use crate::point_2d::Point2d;
use crate::rectangle::Rectangle;
use crate::tile_map::TileMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use uom::si::SI;
use uom::si::length::Length;

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, Serialize, Deserialize)]
pub enum TileMapKey {
    South,
    Hub,
    West,
    East,
    North,
}

#[derive(Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct World {
    pub rows: usize,
    pub columns: usize,
    pub tile_maps: HashMap<TileMapKey, TileMap>,
    pub current_tile_map_id: TileMapKey,
    pub x_offset: f32,
    pub y_offset: f32,
    pub tile_size_meters: Length<SI<f32>, f32>,
    pub tile_size_pixels: u32,
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
        player: Rectangle<f32>,
        width: f32,
        height: f32,
    ) -> Option<Rectangle<f32>> {
        let player_middle = player.width() / 2f32;
        if player.top() <= 0f32 {
            self.try_navigate_north(player, height)
        } else if player.top() >= height {
            self.try_navigate_south(player)
        } else if player.left() - player_middle <= 0f32 {
            self.try_navigate_west(player, width)
        } else if player.left() + player_middle >= width {
            self.try_navigate_east(player)
        } else {
            None
        }
    }

    fn try_navigate_north(
        &mut self,
        player: Rectangle<f32>,
        height: f32,
    ) -> Option<Rectangle<f32>> {
        if let Some(new_tile_map_id) = self.find_north_tile_map() {
            self.current_tile_map_id = new_tile_map_id;
            let bottom = height + player.top();
            Some(player.move_to(player.left(), bottom))
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

    fn try_navigate_south(&mut self, player: Rectangle<f32>) -> Option<Rectangle<f32>> {
        if let Some(new_tile_map_id) = self.find_south_tile_map() {
            self.current_tile_map_id = new_tile_map_id;
            Some(player.move_to(player.left(), player.height()))
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

    fn try_navigate_west(&mut self, player: Rectangle<f32>, width: f32) -> Option<Rectangle<f32>> {
        if let Some(new_tile_map_id) = self.find_west_tile_map() {
            self.current_tile_map_id = new_tile_map_id;
            let right = width + (player.left() - player.width());
            Some(player.move_to(right, player.top()))
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

    fn try_navigate_east(&mut self, player: Rectangle<f32>) -> Option<Rectangle<f32>> {
        if let Some(new_tile_map_id) = self.find_east_tile_map() {
            self.current_tile_map_id = new_tile_map_id;
            Some(player.move_to(player.width(), player.top()))
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
    pub fn is_traversable(&self, point: Point2d<f32>) -> bool {
        let Some(tile_map) = &self.tile_maps.get(&self.current_tile_map_id) else {
            return false;
        };
        #[allow(clippy::cast_sign_loss)]
        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_precision_loss)]
        let tile_x = (point.x() / self.tile_size_pixels as f32) as usize;
        let tile_x = usize::clamp(tile_x, 0, self.columns - 1);
        #[allow(clippy::cast_sign_loss)]
        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_precision_loss)]
        let tile_y = (point.y() / self.tile_size_pixels as f32) as usize;
        let tile_y = usize::clamp(tile_y, 0, self.rows - 1);
        let tile = tile_map.get(tile_y, tile_x);
        tile == 0
    }
}
