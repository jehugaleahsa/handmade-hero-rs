#[derive(Debug)]
pub struct CollisionResult {
    pub collided: bool,
    pub top_collided: bool,
    pub bottom_collided: bool,
    pub left_collided: bool,
    pub right_collided: bool,
}
