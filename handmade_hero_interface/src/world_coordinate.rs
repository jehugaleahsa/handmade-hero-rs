use crate::point_2d::Point2d;
use crate::tile_map_coordinate::TileMapCoordinate;
use crate::tile_map_key::TileMapKey;
use crate::units::si::length::{Length, pixel};
use crate::world::World;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone)]
struct ShiftedCoordinate {
    tile_map: isize,
    tile: usize,
    tile_offset: f32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WorldCoordinate {
    tile_map_key: TileMapKey,
    tile_map_coordinate: TileMapCoordinate,
    tile_size: Length,
    rows: usize,
    columns: usize,
}

impl WorldCoordinate {
    #[inline]
    #[must_use]
    pub fn new(world: &World, key: TileMapKey, tile_map_coordinate: TileMapCoordinate) -> Self {
        Self::new_internal(
            key,
            tile_map_coordinate,
            world.rows(),
            world.columns(),
            world.tile_size(),
        )
    }

    #[inline]
    #[must_use]
    fn new_internal(
        key: TileMapKey,
        tile_map_coordinate: TileMapCoordinate,
        rows: usize,
        columns: usize,
        tile_size: Length,
    ) -> Self {
        Self {
            tile_map_key: key,
            tile_map_coordinate,
            rows,
            columns,
            tile_size,
        }
    }

    #[inline]
    #[must_use]
    pub fn tile_map_key(&self) -> TileMapKey {
        self.tile_map_key
    }

    #[inline]
    #[must_use]
    pub fn tile_map_x(&self) -> isize {
        self.tile_map_key.x()
    }

    #[inline]
    #[must_use]
    pub fn tile_map_y(&self) -> isize {
        self.tile_map_key.y()
    }

    #[inline]
    #[must_use]
    pub fn tile_x(&self) -> usize {
        self.tile_map_coordinate.x()
    }

    #[inline]
    #[must_use]
    pub fn tile_y(&self) -> usize {
        self.tile_map_coordinate.y()
    }

    #[inline]
    #[must_use]
    pub fn tile_offset(&self) -> Point2d<f32> {
        self.tile_map_coordinate.offset()
    }

    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    #[allow(clippy::cast_possible_wrap)]
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    pub fn shifted(&self, delta_x: f32, delta_y: f32) -> WorldCoordinate {
        let tile_offset = self.tile_offset();
        let tile_size = self.tile_size.get::<pixel>();

        let x_shift = ShiftedCoordinate {
            tile_map: self.tile_map_key.x(),
            tile: self.tile_x(),
            tile_offset: tile_offset.x(),
        };
        let ShiftedCoordinate {
            tile_map: tile_map_x,
            tile: tile_x,
            tile_offset: tile_offset_x,
        } = Self::shifted_axis(x_shift, delta_x, tile_size, self.columns);

        let y_shift = ShiftedCoordinate {
            tile_map: self.tile_map_key.y(),
            tile: self.tile_y(),
            tile_offset: tile_offset.y(),
        };
        let ShiftedCoordinate {
            tile_map: tile_map_y,
            tile: tile_y,
            tile_offset: tile_offset_y,
        } = Self::shifted_axis(y_shift, delta_y, tile_size, self.rows);

        let new_tile_map_key = TileMapKey {
            x: tile_map_x,
            y: tile_map_y,
        };
        let new_tile_offset = Point2d::from_x_y(tile_offset_x, tile_offset_y);
        let new_tile_coordinate = TileMapCoordinate::at_x_y_offset(tile_x, tile_y, new_tile_offset);
        WorldCoordinate::new_internal(
            new_tile_map_key,
            new_tile_coordinate,
            self.rows,
            self.columns,
            self.tile_size,
        )
    }

    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    #[allow(clippy::cast_possible_wrap)]
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    fn shifted_axis(
        axis: ShiftedCoordinate,
        delta: f32,
        tile_size: f32,
        max_tiles: usize,
    ) -> ShiftedCoordinate {
        let ShiftedCoordinate {
            tile_offset,
            tile,
            tile_map,
        } = axis;
        let mut tile_offset = tile_offset + delta;
        let mut tile = tile as isize;
        if tile_offset < 0f32 {
            let delta_tile = tile_offset / tile_size;
            tile = f32::floor(tile as f32 + delta_tile) as isize;
            tile_offset %= tile_size;
            tile_offset += tile_size;
        } else if tile_offset >= tile_size {
            let delta_tile = tile_offset / tile_size;
            tile = f32::floor(tile as f32 + delta_tile) as isize;
            tile_offset %= tile_size;
        }
        let mut tile_map = tile_map;
        if tile < 0 {
            let delta_tile_map = tile as f32 / max_tiles as f32;
            tile_map = f32::floor(tile_map as f32 + delta_tile_map) as isize;
            tile %= max_tiles as isize;
            tile += max_tiles as isize;
        } else if tile >= max_tiles as isize {
            let delta_tile_map_x = tile as f32 / max_tiles as f32;
            tile_map = f32::floor(tile_map as f32 + delta_tile_map_x) as isize;
            tile %= max_tiles as isize;
        }
        ShiftedCoordinate {
            tile_offset,
            tile: tile as usize,
            tile_map,
        }
    }
}
