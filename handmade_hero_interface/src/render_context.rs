use std::any::Any;

use crate::back_buffer::BackBuffer;
use crate::game_state::GameState;
use crate::input_state::InputState;

#[derive(Debug)]
pub struct RenderContext<'a> {
    pub input: &'a InputState,
    pub state: &'a mut GameState,
    pub buffer: &'a mut BackBuffer,
    pub plugin_game_state: Option<&'a mut dyn Any>,
}
