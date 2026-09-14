use std::any::Any;

use crate::audio_context::AudioContext;
use crate::initialize_context::InitializeContext;
use crate::input_context::InputContext;
use crate::render_context::RenderContext;

pub trait Application {
    fn create_game_state(&self) -> Box<dyn Any>;

    fn create_audio_state(&self) -> Box<dyn Any>;

    fn initialize(&self, context: InitializeContext<'_>);

    fn process_input(&self, context: InputContext<'_>);

    fn render(&self, context: RenderContext<'_>);

    fn write_sound(&self, context: AudioContext<'_>);
}
