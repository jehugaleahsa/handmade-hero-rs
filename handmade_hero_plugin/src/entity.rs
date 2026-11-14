use handmade_hero_interface::{
    units::si::{length::Length, velocity::Velocity},
    vector_2d::Vector2d,
};
use serde::{Deserialize, Serialize};

use crate::{collision_deltas::CollisionDeltas, world_coordinate::WorldCoordinate};

#[derive(Debug, Serialize, Deserialize)]
pub enum EntityResidence {
    High { position: Vector2d<Length> },
    Low { position: WorldCoordinate },
    Dormant {},
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Entity {
    velocity: Vector2d<Velocity>,
    collision: Option<CollisionDeltas>,
    residence: EntityResidence,
}

impl Entity {
    #[inline]
    #[must_use]
    pub fn new(position: WorldCoordinate) -> Self {
        Self {
            velocity: Vector2d::default(),
            collision: None,
            residence: EntityResidence::Low { position },
        }
    }

    #[inline]
    #[must_use]
    pub fn residence(&self) -> &EntityResidence {
        &self.residence
    }

    #[inline]
    #[must_use]
    pub fn residence_mut(&mut self) -> &mut EntityResidence {
        &mut self.residence
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
