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
    pub fn shifted(&self, delta_x: Length, delta_y: Length, tile_size: Length) -> Self {
        self.shifted_at_offset(delta_x, delta_y, self.tile_offset(), tile_size)
    }

    #[must_use]
    fn shifted_at_offset(
        &self,
        delta_x: Length,
        delta_y: Length,
        tile_offset: Point2d<Length>,
        tile_size: Length,
    ) -> Self {
        let (tile_offset_x, tile_offset_y) = tile_offset.to_tuple();

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
        Self::new_internal(new_chunk_key, new_tile_coordinate, self.rows, self.columns)
    }

    #[must_use]
    #[expect(clippy::cast_possible_truncation)]
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
        let (chunk, tile) = Self::carry_tiles(chunk, tile, tile_delta_m as i32, max_tiles);
        let tile_offset = Length::new::<meter>(tile_offset_m);

        ShiftedCoordinate {
            chunk,
            tile,
            tile_offset,
        }
    }

    /// Moves by whole tiles, keeping the offset within the tile.
    ///
    /// Unlike [`Self::shifted`], no floating-point math is involved, so a point sitting exactly
    /// on a tile edge stays on the same side of it. Shifting by a multiple of the tile size can
    /// round that point into the neighboring tile instead.
    #[must_use]
    pub fn shifted_tiles(&self, delta_x: i16, delta_y: i16) -> Self {
        let (chunk_x, tile_x) = Self::carry_tiles(
            self.chunk_key.x(),
            self.tile_x(),
            i32::from(delta_x),
            self.columns,
        );
        let (chunk_y, tile_y) = Self::carry_tiles(
            self.chunk_key.y(),
            self.tile_y(),
            i32::from(delta_y),
            self.rows,
        );

        let new_chunk_key = ChunkKey::at_x_y_z(chunk_x, chunk_y, self.chunk_key.z());
        let new_tile_coordinate =
            ChunkCoordinate::at_x_y_offset(tile_x, tile_y, self.tile_offset());
        Self::new_internal(new_chunk_key, new_tile_coordinate, self.rows, self.columns)
    }

    /// Adds `tile_delta` to a tile index, carrying whole chunks out of the result.
    #[must_use]
    #[expect(clippy::cast_possible_truncation)]
    #[expect(clippy::cast_sign_loss)]
    fn carry_tiles(chunk: i16, tile: u8, tile_delta: i32, max_tiles: u8) -> (i16, u8) {
        // The intermediate tile index can leave the range of a single chunk in either
        // direction, so the carry runs in `i32` and narrows back at the end.
        let tile = i32::from(tile) + tile_delta;

        // Euclidean division is floor division for a positive divisor, so the negative and
        // positive cases share one expression.
        let max_tiles = i32::from(max_tiles);
        let chunk = i32::from(chunk) + tile.div_euclid(max_tiles);
        let tile = tile.rem_euclid(max_tiles);
        (chunk as i16, tile as u8)
    }

    /// Indicates whether this and the other world coordinate locate the
    /// same tile in the same chunk, ignoring the inner-tile offset.
    #[inline]
    #[must_use]
    pub fn is_same_tile(&self, other: &Self) -> bool {
        self.chunk_key == other.chunk_key
            && self.chunk_coordinate.is_same_tile(&other.chunk_coordinate)
    }

    #[inline]
    #[must_use]
    pub fn elevated(&self, delta_z: i16) -> Self {
        let chunk_key = self.chunk_key.shifted(0, 0, delta_z);
        Self::new_internal(chunk_key, self.chunk_coordinate, self.rows, self.columns)
    }

    #[inline]
    #[must_use]
    pub fn centered(&self) -> Self {
        let x = self.columns / 2;
        let y = self.rows / 2;
        let chunk_key = self.chunk_key;
        let chunk_coordinate = ChunkCoordinate::at_x_y_offset(x, y, Point2d::default());
        WorldCoordinate::new_internal(chunk_key, chunk_coordinate, self.rows, self.columns)
    }

    // The distance between two coordinates in the same chunk
    #[inline]
    #[must_use]
    pub fn distance_to(&self, other: &Self, tile_size: Length) -> (Length, Length) {
        debug_assert_eq!(self.chunk_key, other.chunk_key);
        let delta_x = (f32::from(self.chunk_coordinate.x())
            - f32::from(other.chunk_coordinate.x()))
            * tile_size;
        let delta_y = (f32::from(self.chunk_coordinate.y())
            - f32::from(other.chunk_coordinate.y()))
            * tile_size;
        let (current_offset_x, current_offset_y) = self.chunk_coordinate.tile_offset().to_tuple();
        let (other_offset_x, other_offset_y) = other.chunk_coordinate.tile_offset().to_tuple();
        let delta_x = delta_x + (current_offset_x - other_offset_x);
        let delta_y = delta_y + (current_offset_y - other_offset_y);
        (delta_x, delta_y)
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

    #[test]
    fn test_shifted_tiles_keeps_a_point_on_a_tile_edge_on_the_same_side() {
        // Shifting this point by three tiles' worth of meters rounds it just below the edge of
        // the destination tile, into the tile underneath. Stepping whole tiles must not.
        let tile_size = Length::new::<meter>(1.4);
        let tile_offset = Point2d::from_x_y(Length::zero(), -tile_size / 2.0);
        let chunk_coordinate = ChunkCoordinate::at_x_y_offset(8, 5, tile_offset);
        let key = ChunkKey::at_x_y_z(0, 0, 0);
        let coordinate = WorldCoordinate::new_internal(key, chunk_coordinate, 9, 17);

        let shifted = coordinate.shifted_tiles(0, 3);

        assert_eq!(shifted.chunk_key(), key);
        assert_eq!(shifted.tile_x(), 8);
        assert_eq!(shifted.tile_y(), 8);
        assert_eq!(shifted.tile_offset(), tile_offset);
    }

    #[test]
    fn test_shifted_tiles_carries_into_neighboring_chunks() {
        let tile_offset = Point2d::from_x_y(Length::zero(), Length::zero());
        let chunk_coordinate = ChunkCoordinate::at_x_y_offset(1, 7, tile_offset);
        let key = ChunkKey::at_x_y_z(0, 0, 0);
        let coordinate = WorldCoordinate::new_internal(key, chunk_coordinate, 9, 17);

        let shifted = coordinate.shifted_tiles(-3, 4);

        assert_eq!(shifted.chunk_key(), ChunkKey::at_x_y_z(-1, 1, 0));
        assert_eq!(shifted.tile_x(), 15);
        assert_eq!(shifted.tile_y(), 2);
    }
}
