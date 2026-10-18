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

        let new_chunk_key = ChunkKey::at_x_y_z(chunk_x, chunk_y, self.chunk_key.z());
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
        let mut tile_offset_m = position_m.rem_euclid(tile_size_m);
        if tile_offset_m >= tile_size_m {
            // `f32::rem_euclid` is documented to round up to the divisor when `position` is
            // a tiny negative value. Zero is the same point measured from the next tile.
            tile_offset_m = 0.0;
        }
        // Derive the tile count from the remainder rather than dividing separately, so the pair
        // always adds back up to `position`. `f32::div_euclid` divides first, which can round the
        // quotient up to the next whole number while the remainder stays just under a full tile.
        let tile_delta_m = ((position_m - tile_offset_m) / tile_size_m).round();
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

    /// Indicates whether this and the other world coordinate locate the
    /// same tile in the same chunk, ignoring the inner-tile offset.
    #[inline]
    #[must_use]
    pub fn is_same_tile(&self, other: &WorldCoordinate, include_z: bool) -> bool {
        if include_z {
            self.chunk_key == other.chunk_key
                && self.chunk_coordinate.is_same_tile(&other.chunk_coordinate)
        } else {
            self.chunk_key.x() == other.chunk_key.x()
                && self.chunk_key.y() == other.chunk_key.y()
                && self.chunk_coordinate.is_same_tile(&other.chunk_coordinate)
        }
    }

    #[inline]
    #[must_use]
    pub fn elevated(&self, delta_z: i16) -> WorldCoordinate {
        let chunk_key = self.chunk_key.shifted(0, 0, delta_z);
        Self::new_internal(chunk_key, self.chunk_coordinate, self.rows, self.columns)
    }
}

#[cfg(test)]
mod tests {
    use crate::chunk_coordinate::ChunkCoordinate;
    use crate::chunk_key::ChunkKey;
    use crate::world_coordinate::WorldCoordinate;
    use handmade_hero_interface::point_2d::Point2d;
    use handmade_hero_interface::units::si::length::Length;
    use uom::num_traits::Zero;
    use uom::si::length::meter;

    #[test]
    fn test_shifted_offset_just_under_half_a_tile_lands_on_the_right_tile() {
        // An offset this close to half a tile used to make `f32::div_euclid` round the tile
        // count up while the remainder stayed just under a full tile, landing a tile too far.
        let tile_size = Length::new::<meter>(1.4);
        let tile_offset = Point2d::from_x_y(Length::zero(), Length::new::<meter>(0.699_999_27));
        let chunk_coordinate = ChunkCoordinate::at_x_y_offset(8, 2, tile_offset);
        let key = ChunkKey::at_x_y_z(0, 0, 0);
        let coordinate = WorldCoordinate::new_internal(key, chunk_coordinate, 9, 17);

        let shifted = coordinate.shifted(Length::zero(), tile_size * 5.0, tile_size);

        assert_eq!(shifted.chunk_key(), key);
        assert_eq!(shifted.tile_x(), 8);
        assert_eq!(shifted.tile_y(), 7);
    }
}
