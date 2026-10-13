use crate::tile_map_coordinate::TileMapCoordinate;
use crate::tile_map_key::TileMapKey;
use crate::world::World;
use handmade_hero_interface::point_2d::Point2d;
use handmade_hero_interface::units::si::length::Length;
use serde::{Deserialize, Serialize};
use uom::si::length::meter;

#[derive(Debug, Copy, Clone)]
struct ShiftedCoordinate {
    tile_map: i16,
    tile: u8,
    tile_offset: Length,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WorldCoordinate {
    tile_map_key: TileMapKey,
    tile_map_coordinate: TileMapCoordinate,
    rows: u8,
    columns: u8,
}

impl WorldCoordinate {
    #[inline]
    #[must_use]
    pub fn new(world: &World, key: TileMapKey) -> Self {
        let tile_map_coordinate = TileMapCoordinate::default();
        Self::new_internal(key, tile_map_coordinate, world.rows(), world.columns())
    }

    #[inline]
    #[must_use]
    fn new_internal(
        key: TileMapKey,
        tile_map_coordinate: TileMapCoordinate,
        rows: u8,
        columns: u8,
    ) -> Self {
        Self {
            tile_map_key: key,
            tile_map_coordinate,
            rows,
            columns,
        }
    }

    #[inline]
    #[must_use]
    pub fn within_tile_map(self, tile_map_coordinate: TileMapCoordinate) -> Self {
        Self::new_internal(
            self.tile_map_key,
            tile_map_coordinate,
            self.rows,
            self.columns,
        )
    }

    #[inline]
    #[must_use]
    pub fn tile_map_key(&self) -> TileMapKey {
        self.tile_map_key
    }

    #[inline]
    #[must_use]
    pub fn tile_map_x(&self) -> i16 {
        self.tile_map_key.x()
    }

    #[inline]
    #[must_use]
    pub fn tile_map_y(&self) -> i16 {
        self.tile_map_key.y()
    }

    #[inline]
    #[must_use]
    pub fn tile_x(&self) -> u8 {
        self.tile_map_coordinate.x()
    }

    #[inline]
    #[must_use]
    pub fn tile_y(&self) -> u8 {
        self.tile_map_coordinate.y()
    }

    #[inline]
    #[must_use]
    pub fn tile_offset(&self) -> Point2d<Length> {
        self.tile_map_coordinate.tile_offset()
    }

    #[must_use]
    pub fn shifted(&self, delta_x: Length, delta_y: Length, tile_size: Length) -> WorldCoordinate {
        let (tile_offset_x, tile_offset_y) = self.tile_offset().to_tuple();

        let x_shift = ShiftedCoordinate {
            tile_map: self.tile_map_key.x(),
            tile: self.tile_x(),
            tile_offset: tile_offset_x,
        };
        let ShiftedCoordinate {
            tile_map: tile_map_x,
            tile: tile_x,
            tile_offset: tile_offset_x,
        } = Self::shifted_axis(x_shift, delta_x, tile_size, self.columns);

        let y_shift = ShiftedCoordinate {
            tile_map: self.tile_map_key.y(),
            tile: self.tile_y(),
            tile_offset: tile_offset_y,
        };
        let ShiftedCoordinate {
            tile_map: tile_map_y,
            tile: tile_y,
            tile_offset: tile_offset_y,
        } = Self::shifted_axis(y_shift, delta_y, tile_size, self.rows);

        let new_tile_map_key = TileMapKey::at_x_y(tile_map_x, tile_map_y);
        let new_tile_offset = Point2d::from_x_y(tile_offset_x, tile_offset_y);
        let new_tile_coordinate = TileMapCoordinate::at_x_y_offset(tile_x, tile_y, new_tile_offset);
        WorldCoordinate::new_internal(
            new_tile_map_key,
            new_tile_coordinate,
            self.rows,
            self.columns,
        )
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
            tile_map,
            tile,
            tile_offset,
        } = axis;

        // Carry whole tiles out of the offset so it lands back in [0, tile_size).
        let tile_size_m = tile_size.get::<meter>();
        let position = tile_offset + delta;
        let position_m = position.get::<meter>();
        let mut tile_delta = (position_m / tile_size_m).floor();
        let mut tile_offset_m = position_m.rem_euclid(tile_size_m);
        if tile_offset_m >= tile_size_m {
            // `f32::rem_euclid` is documented to round up to the divisor when `position` is
            // a tiny negative value. Snapping to the next tile keeps the pair consistent.
            tile_offset_m = 0.0;
            tile_delta += 1.0;
        }
        // The intermediate tile index can leave the range of a single tile map in either
        // direction, so the carry runs in `i32` and narrows back at the end.
        let tile = i32::from(tile) + tile_delta as i32;

        // Carry whole tile maps out of the tile index. Euclidean division is floor division
        // for a positive divisor, so the negative and positive cases share one expression.
        let max_tiles = i32::from(max_tiles);
        let tile_map = i32::from(tile_map) + tile.div_euclid(max_tiles);
        let tile = tile.rem_euclid(max_tiles);
        let tile_offset = Length::new::<meter>(tile_offset_m);

        ShiftedCoordinate {
            tile_map: tile_map as i16,
            tile: tile as u8,
            tile_offset,
        }
    }
}
