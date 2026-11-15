use handmade_hero_interface::{
    units::si::{length::Length, velocity::Velocity},
    vector_2d::Vector2d,
};
use serde::{Deserialize, Serialize};

use crate::{collision_deltas::CollisionDeltas, world_coordinate::WorldCoordinate};

#[derive(Debug, Serialize, Deserialize)]
pub struct Entity {
    coordinate: WorldCoordinate,
    velocity: Vector2d<Velocity>,
    collision: Option<CollisionDeltas>,
    height: Length,
    width: Length,
}

impl Entity {
    #[inline]
    #[must_use]
    pub fn new(coordinate: WorldCoordinate, height: Length, width: Length) -> Self {
        Self {
            coordinate,
            velocity: Vector2d::default(),
            collision: None,
            height,
            width,
        }
    }

    #[inline]
    #[must_use]
    pub fn coordinate(&self) -> &WorldCoordinate {
        &self.coordinate
    }

    #[inline]
    #[must_use]
    pub fn coordinate_mut(&mut self) -> &mut WorldCoordinate {
        &mut self.coordinate
    }

    #[inline]
    #[must_use]
    pub fn velocity(&self) -> Vector2d<Velocity> {
        self.velocity
    }

    #[inline]
    #[must_use]
    pub fn velocity_mut(&mut self) -> &mut Vector2d<Velocity> {
        &mut self.velocity
    }

    #[inline]
    #[must_use]
    pub fn collision(&self) -> Option<CollisionDeltas> {
        self.collision
    }

    #[inline]
    pub fn set_collision(&mut self, deltas: CollisionDeltas) {
        self.collision = Some(deltas);
    }

    #[inline]
    #[must_use]
    #[allow(unused)]
    pub fn height(&self) -> Length {
        self.height
    }

    #[inline]
    #[must_use]
    #[allow(unused)]
    pub fn width(&self) -> Length {
        self.width
    }
}
