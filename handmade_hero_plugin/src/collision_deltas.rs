use handmade_hero_interface::units::si::length::Length;

#[derive(Debug, Copy, Clone)]
pub struct CollisionDeltas {
    pub top: Length,
    pub bottom: Length,
    pub left: Length,
    pub right: Length,
}
