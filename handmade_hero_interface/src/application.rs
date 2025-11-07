use crate::audio_context::AudioContext;
use crate::initialize_context::InitializeContext;
use crate::input_context::InputContext;
use crate::render_context::RenderContext;

pub trait Application {
    fn initialize(&mut self, context: InitializeContext<'_>);

    fn process_input(&mut self, context: InputContext<'_>);

    fn render(&mut self, context: RenderContext<'_>);

    fn write_sound(&mut self, context: AudioContext<'_>);
}
