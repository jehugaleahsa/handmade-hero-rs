use std::any::Any;

use crate::{game_state::GameState, input_state::InputState};

#[derive(Debug)]
pub struct InputContext<'a> {
    pub input: &'a InputState,
    pub state: &'a mut GameState,
    pub plugin_game_state: Option<&'a mut dyn Any>,
}
