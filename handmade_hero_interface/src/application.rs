use crate::audio_context::AudioContext;
use crate::initialize_context::InitializeContext;
use crate::input_context::InputContext;
use crate::render_context::RenderContext;

pub trait Application {
    fn initialize(&self, context: InitializeContext<'_>);

    fn process_input(&self, context: InputContext<'_>);

    fn render(&self, context: RenderContext<'_>);

    fn write_sound(&self, context: AudioContext<'_>);
}
