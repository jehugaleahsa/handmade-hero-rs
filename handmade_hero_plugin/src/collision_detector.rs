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
    let mut best = start.clone();
    let mut current_generation = false;
    let mut generation_lengths = [1, 0]; // Start with just the start tile
    let (center_x_offset, center_y_offset) = center.tile_offset().to_tuple();
    #[expect(clippy::cast_possible_truncation)]
    let start_tile: Vector2d<i16> = (starting_offset / tile_size).map(|d| d.get::<ratio>() as i16);
    let half_tile_size = tile_size / 2.0;
    let start_rectangle = to_tile_rectangle(
        center_x_offset,
        center_y_offset,
        start_tile.x(),
        start_tile.y(),
        tile_size,
        half_tile_size,
    );
    let mut generation_a = [start_rectangle, Rectangle::default(), Rectangle::default()];
    let mut generation_b = [
        Rectangle::default(),
        Rectangle::default(),
        Rectangle::default(),
    ];
    loop {
        let (generation, next_generation) = if current_generation {
            (&generation_b, &mut generation_a)
        } else {
            (&generation_a, &mut generation_b)
        };
        let generation_index = usize::from(current_generation);
        let generation_length = generation_lengths[generation_index];
        let mut next_generation_length = 0;
        for tile in generation.iter().take(generation_length) {
            if !line.intersects_rectangle(tile) {
                continue;
            }
            let tile_horizontal_center = tile.left() + half_tile_size + center_x_offset;
            let tile_vertical_center = tile.bottom() + half_tile_size + center_y_offset;
            let delta = Vector2d::from_x_y(tile_horizontal_center, tile_vertical_center);
            let tile_coordinate = center.shifted(delta, tile_size);
            if !world.is_traversable(&tile_coordinate) {
                continue;
            }
            best = tile_coordinate;
            if x_shift != Length::zero() {
                let x_neighbor = tile.shifted(x_shift, Length::zero());
                next_generation[next_generation_length] = x_neighbor;
                next_generation_length += 1;
                if y_shift != Length::zero() {
                    let y_neighbor = tile.shifted(Length::zero(), y_shift);
                    next_generation[next_generation_length] = y_neighbor;
                    next_generation_length += 1;

                    let diagonal_neighbor = tile.shifted(x_shift, y_shift);
                    next_generation[next_generation_length] = diagonal_neighbor;
                    next_generation_length += 1;
                }
            } else if y_shift != Length::zero() {
                let y_neighbor = tile.shifted(Length::zero(), y_shift);
                next_generation[next_generation_length] = y_neighbor;
                next_generation_length += 1;
            }
        }
        if next_generation_length == 0 {
            break;
        }
        current_generation = !current_generation;
        let next_generation_index = usize::from(current_generation);
        generation_lengths[next_generation_index] = next_generation_length;
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
