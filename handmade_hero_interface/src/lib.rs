pub mod application_state;
pub mod audio_context;
pub mod button_state;
pub mod controller_state;
pub mod input_state;
pub mod joystick_state;
pub mod pixel;
pub mod render_context;
pub mod stereo_sample;

use crate::audio_context::AudioContext;
use crate::render_context::RenderContext;

pub trait Application {
    fn render(&self, context: &mut RenderContext<'_>);

    fn write_sound(&self, context: &mut AudioContext<'_>);
}
