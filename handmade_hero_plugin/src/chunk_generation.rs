use rand::prelude::*;

use crate::{chunk_key::ChunkKey, tile_type::TileType, world::World};

// For now, assume stairs are always on the same tile
const STAIRS_X: u8 = 3;
const STAIRS_Y: u8 = 3;

pub fn initialize_chunk(world: &mut World, key: ChunkKey) {
    let chunk = world.get_chunk(key).unwrap();
    let rows = chunk.rows();
    let columns = chunk.columns();
    debug_assert!(rows > 2);
    debug_assert!(columns > 2);

    let mut generator = rand::rng();

    let has_stairs = has_stairs(world, key, || generator.random());
    if has_stairs
        && let Some(value) = world
            .get_chunk_mut(key)
            .unwrap()
            .get_mut(STAIRS_X, STAIRS_Y)
    {
        *value = if key.z() == 0 {
            TileType::UpStairs
        } else {
            TileType::DownStairs
        };
    }

    let [top_door, bottom_door, left_door, right_door] = pick_doors(world, key, &mut generator);

    // Top and bottom
    let top = 0;
    let bottom = rows - 1;
    let middle_column = columns / 2;
    let column_door_indexes = (middle_column - 1)..=(middle_column + 1);
    for column in 0..columns {
        if let Some(value) = world.get_chunk_mut(key).unwrap().get_mut(top, column) {
            if column_door_indexes.contains(&column) && top_door {
                *value = TileType::Door;
            } else {
                *value = TileType::Wall;
            }
        }
        if let Some(value) = world.get_chunk_mut(key).unwrap().get_mut(bottom, column) {
            if column_door_indexes.contains(&column) && bottom_door {
                *value = TileType::Door;
            } else {
                *value = TileType::Wall;
            }
        }
    }

    // Left and right
    let left = 0;
    let right = columns - 1;
    let middle_row = rows / 2;
    let row_door_indexes = (middle_row - 1)..=(middle_row + 1);
    for row in 1..bottom {
        if let Some(value) = world.get_chunk_mut(key).unwrap().get_mut(row, left) {
            if row_door_indexes.contains(&row) && left_door {
                *value = TileType::Door;
            } else {
                *value = TileType::Wall;
            }
        }
        if let Some(value) = world.get_chunk_mut(key).unwrap().get_mut(row, right) {
            if row_door_indexes.contains(&row) && right_door {
                *value = TileType::Door;
            } else {
                *value = TileType::Wall;
            }
        }
    }
}

fn pick_doors(world: &World, key: ChunkKey, generator: &mut ThreadRng) -> [bool; 4] {
    const TOP_INDEX: usize = 0;
    const BOTTOM_INDEX: usize = 1;
    const LEFT_INDEX: usize = 2;
    const RIGHT_INDEX: usize = 3;
    const DOOR_CHANCE: [f64; 4] = [0.5, 0.65, 0.8, 1.0];
    let mut indexes = [0, 1, 2, 3];
    indexes.shuffle(generator);
    let mut doors = [false; 4];
    let mut wall_count = 0;
    for index in indexes {
        let pick = || generator.random_bool(DOOR_CHANCE[wall_count]);
        let is_door = match index {
            TOP_INDEX => top_door(world, key, pick),
            BOTTOM_INDEX => bottom_door(world, key, pick),
            LEFT_INDEX => left_door(world, key, pick),
            RIGHT_INDEX => right_door(world, key, pick),
            _ => unreachable!(),
        };
        if is_door {
            doors[index] = true;
        } else {
            wall_count += 1;
        }
    }
    doors
}

fn has_stairs(world: &World, key: ChunkKey, generator: impl FnOnce() -> bool) -> bool {
    let z_delta = find_delta_z(key.z());
    let neighbor = world.get_chunk(key.shifted(0, 0, z_delta));
    let neighbor_has_stairs = neighbor
        .and_then(|n| n.get(STAIRS_X, STAIRS_Y))
        .map(TileType::is_stairs);
    neighbor_has_stairs.unwrap_or_else(generator)
}

pub fn find_delta_z(current: i16) -> i16 {
    // For now, we will only support two floors. If we're on the top floor, we see if
    // the bottom floor already has stairs and mirror that. Reversed for the bottom floor.
    // If this chunk is generated first, we randomly decide if there are stairs or not.
    if current == 0 {
        1
    } else if current == 1 {
        -1
    } else {
        unreachable!("There should only ever be two floors");
    }
}

fn top_door(world: &World, key: ChunkKey, generator: impl FnOnce() -> bool) -> bool {
    let bottom = world.rows() - 1;
    let middle_column = world.columns() / 2;
    let top_neighbor = world.get_chunk(key.shifted(0, -1, 0));
    let top_neighbor_has_door = top_neighbor
        .and_then(|n| n.get(bottom, middle_column))
        .map(|t| t == TileType::Door);
    top_neighbor_has_door.unwrap_or_else(generator)
}

fn bottom_door(world: &World, key: ChunkKey, generator: impl FnOnce() -> bool) -> bool {
    let top = 0;
    let middle_column = world.columns() / 2;
    let bottom_neighbor = world.get_chunk(key.shifted(0, 1, 0));
    let bottom_neighbor_has_door = bottom_neighbor
        .and_then(|n| n.get(top, middle_column))
        .map(|t| t == TileType::Door);
    bottom_neighbor_has_door.unwrap_or_else(generator)
}

fn left_door(world: &World, key: ChunkKey, generator: impl FnOnce() -> bool) -> bool {
    let right = world.columns() - 1;
    let middle_row = world.rows() / 2;
    let left_neighbor = world.get_chunk(key.shifted(-1, 0, 0));
    let left_neighbor_has_door = left_neighbor
        .and_then(|n| n.get(middle_row, right))
        .map(|t| t == TileType::Door);
    left_neighbor_has_door.unwrap_or_else(generator)
}

fn right_door(world: &World, key: ChunkKey, generator: impl FnOnce() -> bool) -> bool {
    let left = 0;
    let middle_row = world.rows() / 2;
    let right_neighbor = world.get_chunk(key.shifted(1, 0, 0));
    let right_neighbor_has_door = right_neighbor
        .and_then(|n| n.get(middle_row, left))
        .map(|t| t == TileType::Door);
    right_neighbor_has_door.unwrap_or_else(generator)
}
