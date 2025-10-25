use crate::game_state::GameState;
use crate::input_state::InputState;

#[derive(Debug)]
pub struct InputContext<'a> {
    pub input: &'a InputState,
    pub state: &'a mut GameState,
}
