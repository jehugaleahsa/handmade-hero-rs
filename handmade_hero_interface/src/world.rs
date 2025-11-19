use crate::point_2d::Point2d;
use crate::rectangle::Rectangle;
use crate::tile_map::TileMap;
use crate::units::si::length::{Length, pixel};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::hash_map::Entry;

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, Serialize, Deserialize)]
pub struct TileMapKey {
    pub x: isize,
    pub y: isize,
}

#[derive(Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct World {
    pub rows: usize,
    pub columns: usize,
    pub tile_maps: HashMap<TileMapKey, TileMap>,
    pub current_tile_map_key: TileMapKey,
    pub x_offset: Length,
    pub y_offset: Length,
    pub tile_size: Length,
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
        if player.bottom() >= height {
            self.try_navigate_north(player)
        } else if player.top() <= 0f32 {
            self.try_navigate_south(player, height)
        } else if player.right() <= 0f32 {
            self.try_navigate_west(player, width)
        } else if player.left() >= width {
            self.try_navigate_east(player)
        } else {
            None
        }
    }

    fn try_navigate_north(&mut self, player: Rectangle<f32>) -> Option<Rectangle<f32>> {
        if let Some(new_tile_map_key) = self.find_north_tile_map() {
            self.current_tile_map_key = new_tile_map_key;
            Some(player.moved_to(player.left(), 0f32))
        } else {
            None
        }
    }

    fn find_north_tile_map(&self) -> Option<TileMapKey> {
        let TileMapKey { x, y } = self.current_tile_map_key;
        let north_key = TileMapKey { x, y: y + 1 };
        if self.tile_maps.contains_key(&north_key) {
            Some(north_key)
        } else {
            None
        }
    }

    fn try_navigate_south(
        &mut self,
        player: Rectangle<f32>,
        height: f32,
    ) -> Option<Rectangle<f32>> {
        if let Some(new_tile_map_id) = self.find_south_tile_map() {
            self.current_tile_map_key = new_tile_map_id;
            Some(player.moved_to(player.left(), height - player.height()))
        } else {
            None
        }
    }

    fn find_south_tile_map(&self) -> Option<TileMapKey> {
        let TileMapKey { x, y } = self.current_tile_map_key;
        let south_key = TileMapKey { x, y: y - 1 };
        if self.tile_maps.contains_key(&south_key) {
            Some(south_key)
        } else {
            None
        }
    }

    fn try_navigate_west(&mut self, player: Rectangle<f32>, width: f32) -> Option<Rectangle<f32>> {
        if let Some(new_tile_map_id) = self.find_west_tile_map() {
            self.current_tile_map_key = new_tile_map_id;
            let left = width - player.width();
            Some(player.moved_to(left, player.bottom()))
        } else {
            None
        }
    }

    fn find_west_tile_map(&self) -> Option<TileMapKey> {
        let TileMapKey { x, y } = self.current_tile_map_key;
        let west_key = TileMapKey { x: x - 1, y };
        if self.tile_maps.contains_key(&west_key) {
            Some(west_key)
        } else {
            None
        }
    }

    fn try_navigate_east(&mut self, player: Rectangle<f32>) -> Option<Rectangle<f32>> {
        if let Some(new_tile_map_id) = self.find_east_tile_map() {
            self.current_tile_map_key = new_tile_map_id;
            Some(player.moved_to(player.width(), player.bottom()))
        } else {
            None
        }
    }

    fn find_east_tile_map(&self) -> Option<TileMapKey> {
        let TileMapKey { x, y } = self.current_tile_map_key;
        let east_key = TileMapKey { x: x + 1, y };
        if self.tile_maps.contains_key(&east_key) {
            Some(east_key)
        } else {
            None
        }
    }

    /// Indicates whether all four corners of the rectangle fall within a
    /// traversable tile.
    #[must_use]
    pub fn is_traversable_rectangle(&self, bounds: Rectangle<f32>) -> bool {
        self.is_traversable_point(bounds.top_left())
            && self.is_traversable_point(bounds.bottom_left())
            && self.is_traversable_point(bounds.top_right())
            && self.is_traversable_point(bounds.bottom_right())
    }

    /// Indicates whether the point falls within a traversable tile.
    #[must_use]
    pub fn is_traversable_point(&self, point: Point2d<f32>) -> bool {
        self.get_tile_map(point).is_some_and(|tile| tile == 0)
    }

    #[must_use]
    pub fn get_tile_map(&self, point: Point2d<f32>) -> Option<u32> {
        let tile_map = self.tile_maps.get(&self.current_tile_map_key)?;
        let tile_size = self.tile_size.get::<pixel>();

        #[allow(clippy::cast_sign_loss)]
        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_precision_loss)]
        let tile_x = (point.x() / tile_size) as usize;
        let tile_x = usize::clamp(tile_x, 0, self.columns - 1);

        #[allow(clippy::cast_sign_loss)]
        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_precision_loss)]
        let tile_y = (point.y() / tile_size) as usize;
        let tile_y = usize::clamp(tile_y, 0, self.rows - 1);

        tile_map.get(tile_y, tile_x)
    }
}
