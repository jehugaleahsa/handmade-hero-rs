use crate::game_state::GameState;

#[derive(Debug)]
pub struct InitializeContext<'a> {
    pub state: &'a mut GameState,
}
