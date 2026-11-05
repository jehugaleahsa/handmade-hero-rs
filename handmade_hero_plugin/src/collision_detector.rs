use std::cmp::Ordering;

use handmade_hero_interface::{
    directions::Directions, line_segment::LineSegment, point_2d::Point2d, rectangle::Rectangle,
    units::si::length::Length, vector_2d::Vector2d,
};

use crate::{world::World, world_coordinate::WorldCoordinate};
use uom::{num_traits::Zero, si::ratio::ratio};

/// How far short of the wall, as a fraction of the frame's movement, the player stops.
const STOP_SHORT_FRACTION: f32 = 0.001;

pub fn detect_collision(
    world: &World,
    center: &WorldCoordinate,
    start: &WorldCoordinate,
    distance: Vector2d<Length>,
) -> (WorldCoordinate, Directions) {
    let tile_size = world.tile_size();
    let starting_offset = start.distance_to(center, tile_size);
    let starting_point = Point2d::from_x_y(starting_offset.x(), starting_offset.y());
    if !world.is_traversable(start) {
        // The player somehow got stuck in a wall. Let them escape.
        let shifted = start.shifted(distance, tile_size);
        return (shifted, Directions::default());
    }
    let line = LineSegment::from_point_vector(starting_point, distance);
    let (step_x, step_y) = find_axis_steps(distance);
    if step_x == 0 && step_y == 0 {
        return (start.clone(), Directions::default());
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
    let x_direction = find_direction(step_x, Directions::left(), Directions::right());
    let y_direction = find_direction(step_y, Directions::down(), Directions::up());
    let mut directions = Directions::default();
    // The last tile known to be open. The walk stops on the boundary leading out of it.
    let mut open_x = tile_x;
    let mut open_y = tile_y;
    loop {
        let tile_coordinate = center.shifted_tiles(tile_x, tile_y);
        if !world.is_traversable(&tile_coordinate) {
            break;
        }
        open_x = tile_x;
        open_y = tile_y;
        if step_x != 0 && intersects(tile_x + step_x, tile_y) {
            tile_x += step_x;
            directions = x_direction;
        } else if step_y != 0 && intersects(tile_x, tile_y + step_y) {
            tile_y += step_y;
            directions = y_direction;
        } else if step_x != 0 && step_y != 0 && intersects(tile_x + step_x, tile_y + step_y) {
            // The line goes directly through the corner
            let x_open = is_open(tile_x + step_x, tile_y);
            let y_open = is_open(tile_x, tile_y + step_y);
            if x_open && y_open {
                tile_x += step_x;
                tile_y += step_y;
                directions = x_direction.union(y_direction);
            } else {
                directions = Directions::default();
                if !x_open {
                    directions = directions.union(x_direction);
                }
                if !y_open {
                    directions = directions.union(y_direction);
                }
                break;
            }
        } else {
            // The line ends within this tile.
            let new_coordinate = start.shifted(distance, tile_size);
            return (new_coordinate, Directions::default());
        }
    }
    // The player stops where the line crosses the side of the last open tile facing the wall.
    // Finding the fraction of the movement needed to reach that side along one axis is enough,
    // because the line is straight: the same fraction of the way along x is the same fraction
    // of the way along y. When the wall is at a corner, either axis gives the same fraction.
    let fraction = if directions.is_left() || directions.is_right() {
        fraction_to_side(
            open_x,
            step_x,
            center_x_offset,
            starting_point.x(),
            distance.x(),
            tile_size,
        )
    } else {
        fraction_to_side(
            open_y,
            step_y,
            center_y_offset,
            starting_point.y(),
            distance.y(),
            tile_size,
        )
    };
    // Stopping exactly on the side risks floating-point rounding placing the player inside the
    // wall on the next frame, so stop just short of it.
    let fraction = (fraction - STOP_SHORT_FRACTION).max(0.0);
    let new_coordinate = start.shifted(distance * fraction, tile_size);
    (new_coordinate, directions)
}

/// Finds the fraction of the movement along one axis at which the line reaches the side of an
/// open tile it leaves through, moving in the direction of the step.
fn fraction_to_side(
    open_tile: i16,
    step: i16,
    center_offset: Length,
    start: Length,
    distance: Length,
    tile_size: Length,
) -> f32 {
    let tile_center = f32::from(open_tile) * tile_size - center_offset;
    let side = tile_center + f32::from(step) * (tile_size / 2.0);
    ((side - start) / distance).get::<ratio>()
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

#[inline]
#[must_use]
fn find_direction(step: i16, negative: Directions, positive: Directions) -> Directions {
    match step.cmp(&0) {
        Ordering::Equal => Directions::default(),
        Ordering::Less => negative,
        Ordering::Greater => positive,
    }
}
