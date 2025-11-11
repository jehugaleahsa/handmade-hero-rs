use crate::color::Color;
use crate::game_state::GameState;
use crate::input_state::InputState;

#[derive(Debug)]
pub struct RenderContext<'a> {
    pub input: &'a InputState,
    pub state: &'a mut GameState,
    pub buffer: &'a mut [Color<u8>],
}
