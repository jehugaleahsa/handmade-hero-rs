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
    #[expect(clippy::cast_possible_wrap)]
    #[expect(clippy::cast_possible_truncation)]
    #[expect(clippy::cast_sign_loss)]
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

        // Carry whole tiles out of the offset so it lands back in [0, tile_size).
        let position = tile_offset + delta;
        let mut tile_delta = f32::floor(position / tile_size);
        let mut tile_offset = position.rem_euclid(tile_size);
        if tile_offset >= tile_size {
            // `f32::rem_euclid` is documented to round up to the divisor when `position` is
            // a tiny negative value. Snapping to the next tile keeps the pair consistent.
            tile_offset = 0f32;
            tile_delta += 1f32;
        }
        let tile = tile as isize + tile_delta as isize;

        // Carry whole tile maps out of the tile index. Euclidean division is floor division
        // for a positive divisor, so the negative and positive cases share one expression.
        let max_tiles = max_tiles as isize;
        let tile_map = tile_map + tile.div_euclid(max_tiles);
        let tile = tile.rem_euclid(max_tiles);

        ShiftedCoordinate {
            tile_offset,
            tile: tile as usize,
            tile_map,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::world_coordinate::{ShiftedCoordinate, WorldCoordinate};

    const TILE_SIZE: f32 = 60f32;
    const MAX_TILES: usize = 9;

    fn shift(tile_map: isize, tile: usize, tile_offset: f32, delta: f32) -> (isize, usize, f32) {
        let axis = ShiftedCoordinate {
            tile_map,
            tile,
            tile_offset,
        };
        let shifted = WorldCoordinate::shifted_axis(axis, delta, TILE_SIZE, MAX_TILES);
        (shifted.tile_map, shifted.tile, shifted.tile_offset)
    }

    fn assert_axis(actual: (isize, usize, f32), expected: (isize, usize, f32)) {
        assert_eq!(actual.0, expected.0, "tile map");
        assert_eq!(actual.1, expected.1, "tile");
        assert!(
            (actual.2 - expected.2).abs() < 1e-4f32,
            "tile offset: expected {}, found {}",
            expected.2,
            actual.2
        );
    }

    #[test]
    fn test_shift_stays_within_tile() {
        assert_axis(shift(0, 3, 10f32, 15f32), (0, 3, 25f32));
    }

    #[test]
    fn test_shift_crosses_forward_into_next_tile() {
        assert_axis(shift(0, 3, 50f32, 20f32), (0, 4, 10f32));
    }

    #[test]
    fn test_shift_crosses_backward_into_previous_tile() {
        assert_axis(shift(0, 3, 10f32, -20f32), (0, 2, 50f32));
    }

    /// Landing exactly on a tile boundary must not leave the offset sitting at `tile_size`,
    /// which would be one whole tile past where the coordinate actually is.
    #[test]
    fn test_shift_lands_on_exact_tile_boundary() {
        assert_axis(shift(0, 1, 0f32, -TILE_SIZE), (0, 0, 0f32));
    }

    #[test]
    fn test_shift_crosses_forward_into_next_tile_map() {
        assert_axis(shift(0, 8, 50f32, 20f32), (1, 0, 10f32));
    }

    #[test]
    fn test_shift_crosses_backward_into_previous_tile_map() {
        assert_axis(shift(0, 0, 10f32, -20f32), (-1, 8, 50f32));
    }

    /// Landing exactly on a tile map boundary must wrap the tile index to zero rather than
    /// to `max_tiles`, which is one past the end of the map.
    #[test]
    fn test_shift_lands_on_exact_tile_map_boundary() {
        #[expect(clippy::cast_precision_loss)]
        let delta = -(MAX_TILES as f32) * TILE_SIZE;
        assert_axis(shift(0, 0, 0f32, delta), (-1, 0, 0f32));
    }

    #[test]
    fn test_shift_spans_multiple_tile_maps() {
        #[expect(clippy::cast_precision_loss)]
        let delta = -((MAX_TILES * 2) as f32) * TILE_SIZE - 30f32;
        assert_axis(shift(0, 0, 0f32, delta), (-3, 8, 30f32));
    }
}
