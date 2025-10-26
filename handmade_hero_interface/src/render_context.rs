use crate::game_state::GameState;
use crate::input_state::InputState;
use crate::u8_color::U8Color;

#[derive(Debug)]
pub struct RenderContext<'a> {
    pub input: &'a InputState,
    pub state: &'a mut GameState,
    pub buffer: &'a mut [U8Color],
}
