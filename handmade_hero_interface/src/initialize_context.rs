use crate::{back_buffer::BackBuffer, game_state::GameState};

#[derive(Debug)]
pub struct InitializeContext<'a> {
    pub state: &'a mut GameState,
    pub back_buffer: &'a mut BackBuffer,
}
