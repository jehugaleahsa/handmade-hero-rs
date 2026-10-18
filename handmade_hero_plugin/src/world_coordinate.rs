use crate::chunk_coordinate::ChunkCoordinate;
use crate::chunk_key::ChunkKey;
use crate::world::World;
use handmade_hero_interface::point_2d::Point2d;
use handmade_hero_interface::units::si::length::Length;
use serde::{Deserialize, Serialize};
use uom::si::length::meter;

#[derive(Debug, Copy, Clone)]
struct ShiftedCoordinate {
    chunk: i16,
    tile: u8,
    tile_offset: Length,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WorldCoordinate {
    chunk_key: ChunkKey,
    chunk_coordinate: ChunkCoordinate,
    rows: u8,
    columns: u8,
}

impl WorldCoordinate {
    #[inline]
    #[must_use]
    pub fn new(world: &World, key: ChunkKey) -> Self {
        let chunk_coordinate = ChunkCoordinate::default();
        Self::new_internal(key, chunk_coordinate, world.rows(), world.columns())
    }

    #[inline]
    #[must_use]
    fn new_internal(
        key: ChunkKey,
        chunk_coordinate: ChunkCoordinate,
        rows: u8,
        columns: u8,
    ) -> Self {
        Self {
            chunk_key: key,
            chunk_coordinate,
            rows,
            columns,
        }
    }

    #[inline]
    #[must_use]
    pub fn chunk_key(&self) -> ChunkKey {
        self.chunk_key
    }

    #[inline]
    #[must_use]
    pub fn tile_x(&self) -> u8 {
        self.chunk_coordinate.x()
    }

    #[inline]
    #[must_use]
    pub fn tile_y(&self) -> u8 {
        self.chunk_coordinate.y()
    }

    #[inline]
    #[must_use]
    pub fn tile_offset(&self) -> Point2d<Length> {
        self.chunk_coordinate.tile_offset()
    }

    #[must_use]
    pub fn shifted(&self, delta_x: Length, delta_y: Length, tile_size: Length) -> WorldCoordinate {
        let (tile_offset_x, tile_offset_y) = self.tile_offset().to_tuple();

        let x_shift = ShiftedCoordinate {
            chunk: self.chunk_key.x(),
            tile: self.tile_x(),
            tile_offset: tile_offset_x,
        };
        let ShiftedCoordinate {
            chunk: chunk_x,
            tile: tile_x,
            tile_offset: tile_offset_x,
        } = Self::shifted_axis(x_shift, delta_x, tile_size, self.columns);

        let y_shift = ShiftedCoordinate {
            chunk: self.chunk_key.y(),
            tile: self.tile_y(),
            tile_offset: tile_offset_y,
        };
        let ShiftedCoordinate {
            chunk: chunk_y,
            tile: tile_y,
            tile_offset: tile_offset_y,
        } = Self::shifted_axis(y_shift, delta_y, tile_size, self.rows);

        let new_chunk_key = ChunkKey::at_x_y(chunk_x, chunk_y);
        let new_tile_offset = Point2d::from_x_y(tile_offset_x, tile_offset_y);
        let new_tile_coordinate = ChunkCoordinate::at_x_y_offset(tile_x, tile_y, new_tile_offset);
        WorldCoordinate::new_internal(new_chunk_key, new_tile_coordinate, self.rows, self.columns)
    }

    #[must_use]
    #[expect(clippy::cast_possible_truncation)]
    #[expect(clippy::cast_sign_loss)]
    fn shifted_axis(
        axis: ShiftedCoordinate,
        delta: Length,
        tile_size: Length,
        max_tiles: u8,
    ) -> ShiftedCoordinate {
        let ShiftedCoordinate {
            chunk,
            tile,
            tile_offset,
        } = axis;

        // Carry whole tiles out of the offset so it lands back in [-tile_size / 2, tile_size / 2),
        // measured from the tile's center. Shifting by half a tile measures from the tile's low
        // edge instead, which turns the carry into plain floor division.
        let tile_size_m = tile_size.get::<meter>();
        let half_tile_size_m = tile_size_m / 2.0;
        let position = tile_offset + delta;
        let position_m = position.get::<meter>() + half_tile_size_m;
        let mut tile_delta_m = position_m.div_euclid(tile_size_m);
        let mut tile_offset_m = position_m.rem_euclid(tile_size_m);
        if tile_offset_m >= tile_size_m {
            // `f32::rem_euclid` is documented to round up to the divisor when `position` is
            // a tiny negative value. Snapping to the next tile keeps the pair consistent.
            tile_offset_m = 0.0;
            tile_delta_m += 1.0;
        }
        tile_offset_m -= half_tile_size_m;
        // The intermediate tile index can leave the range of a single chunk in either
        // direction, so the carry runs in `i32` and narrows back at the end.
        let tile = i32::from(tile) + tile_delta_m as i32;

        // Carry whole chunks out of the tile index. Euclidean division is floor division
        // for a positive divisor, so the negative and positive cases share one expression.
        let max_tiles = i32::from(max_tiles);
        let chunk = i32::from(chunk) + tile.div_euclid(max_tiles);
        let tile = tile.rem_euclid(max_tiles);
        let tile_offset = Length::new::<meter>(tile_offset_m);

        ShiftedCoordinate {
            chunk: chunk as i16,
            tile: tile as u8,
            tile_offset,
        }
    }
}
