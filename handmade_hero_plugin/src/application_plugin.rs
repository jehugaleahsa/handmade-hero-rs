use handmade_hero_interface::Application;
use handmade_hero_interface::application_state::ApplicationState;
use handmade_hero_interface::audio_context::AudioContext;
use handmade_hero_interface::pixel::Pixel;
use handmade_hero_interface::render_context::RenderContext;
use handmade_hero_interface::stereo_sample::StereoSample;

#[allow(dead_code)]
#[derive(Debug)]
pub struct ApplicationPlugin;

impl ApplicationPlugin {
    #[unsafe(no_mangle)]
    #[must_use]
    pub extern "Rust" fn create_application() -> Box<dyn Application> {
        Box::new(ApplicationPlugin)
    }

    fn render_player(context: &mut RenderContext<'_>) {
        #[allow(clippy::cast_sign_loss)]
        #[allow(clippy::cast_possible_truncation)]
        let player_top =
            (f32::from(context.player_y()) + -100f32 * (context.jump_time() * 2f32).sin()) as usize;
        let player_left = usize::from(context.player_x());
        let player_bottom = player_top.saturating_add(usize::from(ApplicationState::PLAYER_HEIGHT));
        let player_right = player_left.saturating_add(usize::from(ApplicationState::PLAYER_WIDTH));
        let pitch = usize::from(context.width());
        for y in player_top..player_bottom {
            for x in player_left..player_right {
                let index = y * pitch + x;
                let pixel = Pixel::from_rgb(0xFF, 0xFF, 0x00);
                context.set_pixel(index, pixel);
            }
        }
    }
}

impl Application for ApplicationPlugin {
    fn render(&self, context: &mut RenderContext<'_>) {
        let mut index = 0;
        for y in 0..context.height() {
            for x in 0..context.width() {
                let color = Pixel::from_rgb(
                    0x00,
                    (y.wrapping_add(context.y_offset()) & 0xFF) as u8,
                    (x.wrapping_add(context.x_offset()) & 0xFF) as u8,
                );
                context.set_pixel(index, color);
                index += 1;
            }
        }
        Self::render_player(context);
    }

    fn write_sound(&self, context: &mut AudioContext<'_>) {
        let time_delta = context.time_delta();
        for index in 0..context.sample_count() {
            let sine_value = context.theta().sin();
            let volume = f32::from(context.volume());
            #[allow(clippy::cast_possible_truncation)]
            let sample_value = (sine_value * volume) as i16;
            let sample = StereoSample::from_left_right(sample_value, sample_value);
            context.set_sample(index, sample);
            context.advance_theta(time_delta);
        }
    }
}
