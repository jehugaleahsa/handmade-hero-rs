use handmade_hero_interface::units::si::length::Length;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct CollisionDeltas {
    pub top: Length,
    pub bottom: Length,
    pub left: Length,
    pub right: Length,
}
