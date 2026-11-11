use handmade_hero_interface::{units::si::length::Length, vector_2d::Vector2d};

use crate::{collision_deltas::CollisionDeltas, world::World, world_coordinate::WorldCoordinate};
use uom::{
    num_traits::Zero,
    si::{length::meter, ratio::ratio},
};

/// How far short of a wall, as a fraction of the movement, the entity stops.
const STOP_SHORT_FRACTION: f32 = 0.001;

/// How far behind a wall's face an entity can start and still be stopped by that wall.
const MAX_PENETRATION_METERS: f32 = 0.001;

/// Moves an entity `distance` from `start`, stopping just short of the first wall it touches.
pub fn detect_collision(
    world: &World,
    center: &WorldCoordinate,
    start: &WorldCoordinate,
    distance: Vector2d<Length>,
    collision_deltas: CollisionDeltas,
) -> WorldCoordinate {
    let tile_size = world.tile_size();
    let half_tile_size = tile_size / 2.0;
    // Positions are measured from the center of the center tile, so the tile `x` tiles over and
    // `y` tiles up from it has its center at `(x, y) * tile_size`. Tiles are found by whole-tile
    // counts from the center tile, so no position is ever converted back into a tile coordinate.
    let (center_x_offset, center_y_offset) = center.tile_offset().to_tuple();
    let center_offset = Vector2d::from_x_y(center_x_offset, center_y_offset);
    let starting = start.distance_to(center, tile_size) + center_offset;
    let ending = starting + distance;

    // Every tile the entity could touch lies between the tiles it starts and ends in, widened
    // by how many tiles the entity itself spans.
    let width_tiles = tiles_spanned(collision_deltas.right - collision_deltas.left, tile_size);
    let height_tiles = tiles_spanned(collision_deltas.top - collision_deltas.bottom, tile_size);
    let (min_tile_x, max_tile_x) = tile_range(starting.x(), ending.x(), tile_size, width_tiles);
    let (min_tile_y, max_tile_y) = tile_range(starting.y(), ending.y(), tile_size, height_tiles);

    // The Minkowski sum: instead of testing the entity's rectangle against each tile, grow the
    // tile by the entity's shape mirrored through its center and test the entity's center point
    // against that. The entity's right edge is what reaches a tile's left side, so the left side
    // moves out by the right delta, and likewise for the other sides. Left and bottom deltas are
    // negative offsets from the center, so subtracting them moves those sides outward.
    let min_x = -half_tile_size - collision_deltas.right;
    let min_y = -half_tile_size - collision_deltas.top;
    let min_corner = Vector2d::from_x_y(min_x, min_y);
    let max_x = half_tile_size - collision_deltas.left;
    let max_y = half_tile_size - collision_deltas.bottom;
    let max_corner = Vector2d::from_x_y(max_x, max_y);

    let mut min_percent = 1.0f32;
    for tile_y in min_tile_y..=max_tile_y {
        for tile_x in min_tile_x..=max_tile_x {
            let tile = center.shifted_tiles(tile_x, tile_y);
            if world.is_traversable(&tile) {
                continue;
            }
            let tile_center_x = f32::from(tile_x) * tile_size;
            let tile_center_y = f32::from(tile_y) * tile_size;
            let tile_center = Vector2d::from_x_y(tile_center_x, tile_center_y);
            let relative = starting - tile_center;
            // The vertical walls are tested along x. The horizontal walls swap the axes so the
            // same test runs along y.
            let flipped_relative = Vector2d::from_x_y(relative.y(), relative.x());
            let flipped_distance = Vector2d::from_x_y(distance.y(), distance.x());
            let x_span = (min_corner.x(), max_corner.x());
            let y_span = (min_corner.y(), max_corner.y());
            // A wall on the minimum side faces the negative direction, and a wall on the maximum
            // side faces the positive direction.
            test_wall(
                min_corner.x(),
                -1.0,
                y_span,
                relative,
                distance,
                &mut min_percent,
            );
            test_wall(
                max_corner.x(),
                1.0,
                y_span,
                relative,
                distance,
                &mut min_percent,
            );
            test_wall(
                min_corner.y(),
                -1.0,
                x_span,
                flipped_relative,
                flipped_distance,
                &mut min_percent,
            );
            test_wall(
                max_corner.y(),
                1.0,
                x_span,
                flipped_relative,
                flipped_distance,
                &mut min_percent,
            );
        }
    }
    start.shifted(distance * min_percent, tile_size)
}

/// Tests whether a move crosses a wall running parallel to the y-axis at `wall_x`, between
/// `min_y` and `max_y`, sooner than `t_min`. On a hit, `t_min` is lowered to stop just short of
/// the wall. The horizontal walls use the same test with x and y swapped in every argument.
/// A wall only blocks movement heading into the side it faces. `facing` is `-1.0` when that
/// side looks toward negative x and `1.0` when it looks toward positive x. Movement along the
/// wall or away from it is never a hit, so an entity resting on a wall can always back off it.
fn test_wall(
    wall_x: Length,
    facing: f32,
    (min_y, max_y): (Length, Length),
    relative: Vector2d<Length>,
    delta: Vector2d<Length>,
    min_percent: &mut f32,
) {
    // How far the move travels toward the wall's face, positive when heading into it.
    let approach = delta.x() * -facing;
    if approach <= Length::zero() {
        return;
    }
    // How far the entity starts in front of the wall's face, negative when it is behind it.
    let gap = (relative.x() - wall_x) * facing;
    // Rounding can leave an entity a hair behind the face it stopped at. That still counts as
    // touching the wall. Anything deeper started on the far side, and this wall is not what
    // stands in its way.
    if gap < -Length::new::<meter>(MAX_PENETRATION_METERS) {
        return;
    }
    let percent = (gap / approach).get::<ratio>().max(0.0);
    if percent >= *min_percent {
        return;
    }
    let y = relative.y() + delta.y() * percent;
    if y < min_y || y > max_y {
        return;
    }
    // Stopping exactly on the wall risks floating-point rounding placing the entity inside it
    // on the next move, so stop just short of it.
    *min_percent = (percent - STOP_SHORT_FRACTION).max(0.0);
}

/// Finds the tiles, counted from the center tile, that a move along one axis from `start` to
/// `end` could touch, widened on both sides by `margin_tiles`.
#[inline]
#[must_use]
fn tile_range(start: Length, end: Length, tile_size: Length, margin_tiles: i16) -> (i16, i16) {
    let start_tile = to_tile_index(start, tile_size);
    let end_tile = to_tile_index(end, tile_size);
    let min_tile = start_tile.min(end_tile) - margin_tiles;
    let max_tile = start_tile.max(end_tile) + margin_tiles;
    (min_tile, max_tile)
}

/// Finds how many whole tiles a length covers, rounding up.
#[inline]
#[must_use]
fn tiles_spanned(length: Length, tile_size: Length) -> i16 {
    let tiles = (length / tile_size).get::<ratio>();
    #[expect(clippy::cast_possible_truncation)]
    let tiles = tiles.ceil() as i16;
    tiles
}

/// Finds which tile, counted from the center tile, holds a point at the given offset from the
/// center tile's center.
#[inline]
#[must_use]
fn to_tile_index(offset_from_tile_center: Length, tile_size: Length) -> i16 {
    // Tile k covers [k - 0.5, k + 0.5) in tile units, including its left or bottom side and
    // excluding its right or top side. Shifting by half a tile lines those sides up with whole
    // numbers, so flooring lands every point, negative ones and ones exactly on a side included,
    // in the right tile.
    let tiles = (offset_from_tile_center / tile_size).get::<ratio>();
    #[expect(clippy::cast_possible_truncation)]
    let index = (tiles + 0.5).floor() as i16;
    index
}
