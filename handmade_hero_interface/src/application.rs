use crate::audio_context::AudioContext;
use crate::game_state::GameState;
use crate::input_state::InputState;
use crate::render_context::RenderContext;

pub trait Application {
    fn process_input(&self, input: &InputState, state: &mut GameState);

    fn render(&self, context: &mut RenderContext<'_>);

    fn write_sound(&self, context: &mut AudioContext<'_>);
}
