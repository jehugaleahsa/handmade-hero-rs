pub mod application_state;
pub mod button_state;
pub mod controller_state;
pub mod input_state;
pub mod joystick_state;
pub mod pixel;
pub mod stereo_sample;

use crate::application_state::ApplicationState;
use crate::pixel::Pixel;
use crate::stereo_sample::StereoSample;

pub trait Application {
    fn render(
        &self,
        state: &mut ApplicationState,
        bitmap_buffer: &mut [Pixel],
        width: u16,
        height: u16,
    );

    fn write_sound(&self, state: &mut ApplicationState, sound_buffer: &mut [StereoSample]);
}
