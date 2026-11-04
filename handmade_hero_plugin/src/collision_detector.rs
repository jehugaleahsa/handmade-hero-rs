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
    let (x_multiplier, y_multiplier) = find_axis_multipliers(distance);
    if x_multiplier == 0.0 && y_multiplier == 0.0 {
        return start.clone();
    }
    let x_shift = x_multiplier * tile_size;
    let y_shift = y_multiplier * tile_size;
    let (center_x_offset, center_y_offset) = center.tile_offset().to_tuple();
    let start_tile_x = to_tile_index(starting_offset.x() + center_x_offset, tile_size);
    let start_tile_y = to_tile_index(starting_offset.y() + center_y_offset, tile_size);
    let half_tile_size = tile_size / 2.0;
    let mut tile = to_tile_rectangle(
        center_x_offset,
        center_y_offset,
        start_tile_x,
        start_tile_y,
        tile_size,
        half_tile_size,
    );
    let mut best = start.clone();
    loop {
        let tile_coordinate = to_tile_cooridinate(
            center,
            &tile,
            tile_size,
            half_tile_size,
            center_x_offset,
            center_y_offset,
        );
        if !world.is_traversable(&tile_coordinate) {
            break;
        }
        best = tile_coordinate;
        if x_shift != Length::zero() {
            let x_neighbor = tile.shifted(x_shift, Length::zero());
            if line.intersects_rectangle(&x_neighbor) {
                tile = x_neighbor;
            } else if y_shift != Length::zero() {
                let y_neighbor = tile.shifted(Length::zero(), y_shift);
                if line.intersects_rectangle(&y_neighbor) {
                    tile = y_neighbor;
                } else {
                    // The line goes directly through the corner
                    let diagonal_neighbor = tile.shifted(x_shift, y_shift);
                    if !line.intersects_rectangle(&diagonal_neighbor) {
                        break;
                    }
                    let x_neighbor_coordinate = to_tile_cooridinate(
                        center,
                        &x_neighbor,
                        tile_size,
                        half_tile_size,
                        center_x_offset,
                        center_y_offset,
                    );
                    if !world.is_traversable(&x_neighbor_coordinate) {
                        break;
                    }
                    let y_neighbor_coordinate = to_tile_cooridinate(
                        center,
                        &y_neighbor,
                        tile_size,
                        half_tile_size,
                        center_x_offset,
                        center_y_offset,
                    );
                    if !world.is_traversable(&y_neighbor_coordinate) {
                        break;
                    }
                    tile = diagonal_neighbor;
                }
            }
        } else if y_shift != Length::zero() {
            let y_neighbor = tile.shifted(Length::zero(), y_shift);
            if line.intersects_rectangle(&y_neighbor) {
                tile = y_neighbor;
            }
        }
    }
    best
}

#[inline]
#[must_use]
fn find_axis_multipliers(distance: Vector2d<Length>) -> (f32, f32) {
    let x_multiplier = sign_multiplier(distance.x());
    let y_multiplier = sign_multiplier(distance.y());
    (x_multiplier, y_multiplier)
}

#[inline]
#[must_use]
fn sign_multiplier(length: Length) -> f32 {
    match length.partial_cmp(&Length::zero()) {
        Some(Ordering::Less) => -1.0,
        Some(Ordering::Greater) => 1.0,
        Some(Ordering::Equal) | None => 0.0,
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

fn to_tile_cooridinate(
    center: &WorldCoordinate,
    tile: &Rectangle<Length>,
    tile_size: Length,
    half_tile_size: Length,
    center_x_offset: Length,
    center_y_offset: Length,
) -> WorldCoordinate {
    let tile_horizontal_center = tile.left() + half_tile_size + center_x_offset;
    let tile_vertical_center = tile.bottom() + half_tile_size + center_y_offset;
    let delta = Vector2d::from_x_y(tile_horizontal_center, tile_vertical_center);
    center.shifted(delta, tile_size)
}
