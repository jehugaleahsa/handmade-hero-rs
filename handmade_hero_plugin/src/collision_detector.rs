use std::cmp::Ordering;

use handmade_hero_interface::{
    line_segment::LineSegment,
    point_2d::Point2d,
    rectangle::Rectangle,
    units::si::{length::Length, time::Time, velocity::Velocity},
    vector_2d::Vector2d,
};

use crate::{world::World, world_coordinate::WorldCoordinate};
use uom::{num_traits::Zero, si::ratio::ratio};

pub fn detect_collision(
    world: &World,
    center: &WorldCoordinate,
    start: &WorldCoordinate,
    velocity: Vector2d<Velocity>,
    duration: Time,
) -> WorldCoordinate {
    let tile_size = world.tile_size();
    let starting_offset = start.distance_to(center, tile_size);
    let starting_point = Point2d::from_x_y(starting_offset.x(), starting_offset.y());
    let distance = velocity * duration; // m/s * s = m
    let line = LineSegment::from_point_vector(starting_point, distance);
    let (step_x, step_y) = find_axis_steps(distance);
    if step_x == 0 && step_y == 0 {
        return start.clone();
    }
    // Tiles are tracked as whole-tile counts from the center tile. Rectangles in the same
    // space as the line are built from those counts for intersection tests, and world
    // coordinates are built from them for traversal checks, so neither is converted back
    // into the other.
    let (center_x_offset, center_y_offset) = center.tile_offset().to_tuple();
    let half_tile_size = tile_size / 2.0;
    let intersects = |tile_x: i16, tile_y: i16| {
        let rectangle = to_tile_rectangle(
            center_x_offset,
            center_y_offset,
            tile_x,
            tile_y,
            tile_size,
            half_tile_size,
        );
        line.intersects_rectangle(&rectangle)
    };
    let is_open = |tile_x: i16, tile_y: i16| {
        let shifted = center.shifted_tiles(tile_x, tile_y);
        world.is_traversable(&shifted)
    };
    let mut tile_x = to_tile_index(starting_offset.x() + center_x_offset, tile_size);
    let mut tile_y = to_tile_index(starting_offset.y() + center_y_offset, tile_size);
    let mut best = start.clone();
    loop {
        let tile_coordinate = center.shifted_tiles(tile_x, tile_y);
        if !world.is_traversable(&tile_coordinate) {
            break;
        }
        best = tile_coordinate;
        if step_x != 0 && intersects(tile_x + step_x, tile_y) {
            tile_x += step_x;
        } else if step_y != 0 && intersects(tile_x, tile_y + step_y) {
            tile_y += step_y;
        } else if step_x != 0 && step_y != 0 && intersects(tile_x + step_x, tile_y + step_y) {
            // The line goes directly through the corner
            if !is_open(tile_x + step_x, tile_y) || !is_open(tile_x, tile_y + step_y) {
                break;
            }
            tile_x += step_x;
            tile_y += step_y;
        } else {
            // The line ends within this tile.
            break;
        }
    }
    best
}

#[inline]
#[must_use]
fn find_axis_steps(distance: Vector2d<Length>) -> (i16, i16) {
    let step_x = sign_step(distance.x());
    let step_y = sign_step(distance.y());
    (step_x, step_y)
}

#[inline]
#[must_use]
fn sign_step(length: Length) -> i16 {
    match length.partial_cmp(&Length::zero()) {
        Some(Ordering::Less) => -1,
        Some(Ordering::Greater) => 1,
        Some(Ordering::Equal) | None => 0,
    }
}

/// Finds which tile, counted from the center tile, holds a point at the given offset from the
/// center tile's center.
#[inline]
#[must_use]
fn to_tile_index(offset_from_tile_center: Length, tile_size: Length) -> i16 {
    // Tile k covers [k - 0.5, k + 0.5) in tile units, including its left or bottom side and
    // excluding its right or top side, the same as its rectangle. Shifting by half a tile lines
    // those sides up with whole numbers, so flooring lands every point, negative ones and ones
    // exactly on a side included, in the same tile the rectangle would.
    let tiles = (offset_from_tile_center / tile_size).get::<ratio>();
    #[expect(clippy::cast_possible_truncation)]
    let index = (tiles + 0.5).floor() as i16;
    index
}

fn to_tile_rectangle(
    center_x_offset: Length,
    center_y_offset: Length,
    tiles_x: i16,
    tiles_y: i16,
    tile_size: Length,
    half_tile_size: Length,
) -> Rectangle<Length> {
    let delta_x = f32::from(tiles_x) * tile_size;
    let delta_y = f32::from(tiles_y) * tile_size;
    let left = delta_x - center_x_offset - half_tile_size;
    let bottom = delta_y - center_y_offset - half_tile_size;
    Rectangle::new(bottom, left, tile_size, tile_size)
}
