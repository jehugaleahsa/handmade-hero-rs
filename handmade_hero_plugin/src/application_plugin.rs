use handmade_hero_interface::application_state::ApplicationState;
use handmade_hero_interface::pixel::Pixel;
use handmade_hero_interface::stereo_sample::StereoSample;
use handmade_hero_interface::Application;

#[allow(dead_code)]
#[derive(Debug)]
pub struct ApplicationPlugin;

impl Application for ApplicationPlugin {
    fn render(
        &self,
        state: &mut ApplicationState,
        bitmap_buffer: &mut [Pixel],
        width: u16,
        height: u16,
    ) {
        let mut index = 0;
        for y in 0..height {
            for x in 0..width {
                let color = Pixel::from_rgb(
                    0x00,
                    (y.wrapping_add(state.y_offset()) & 0xFF) as u8,
                    (x.wrapping_add(state.x_offset()) & 0xFF) as u8,
                );
                bitmap_buffer[index] = color;
                index += 1;
            }
        }
    }

    fn write_sound(&self, state: &mut ApplicationState, sound_buffer: &mut [StereoSample]) {
        let time_delta = state.time_delta();
        for sample in sound_buffer {
            let sine_value = state.sound_theta().sin();
            let volume = f32::from(state.sound_volume());
            #[allow(clippy::cast_possible_truncation)]
            let sample_value = (sine_value * volume) as i16;
            *sample = StereoSample::from_left_right(sample_value, sample_value);
            state.advance_sound_theta(time_delta);
        }
    }
}
