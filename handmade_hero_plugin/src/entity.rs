use handmade_hero_interface::{units::si::velocity::Velocity, vector_2d::Vector2d};
use serde::{Deserialize, Serialize};

use crate::{collision_deltas::CollisionDeltas, world_coordinate::WorldCoordinate};

#[derive(Debug, Serialize, Deserialize)]
pub struct Entity {
    position: WorldCoordinate,
    velocity: Vector2d<Velocity>,
    collision: Option<CollisionDeltas>,
}

impl Entity {
    #[inline]
    #[must_use]
    pub fn new(position: WorldCoordinate) -> Self {
        Self {
            position,
            velocity: Vector2d::default(),
            collision: None,
        }
    }

    #[inline]
    #[must_use]
    pub fn position(&self) -> &WorldCoordinate {
        &self.position
    }

    #[inline]
    #[must_use]
    pub fn position_mut(&mut self) -> &mut WorldCoordinate {
        &mut self.position
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
}
