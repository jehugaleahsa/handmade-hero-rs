use handmade_hero_interface::application::Application;
use handmade_hero_interface::audio_context::AudioContext;
use handmade_hero_interface::game_state::GameState;
use handmade_hero_interface::input_context::InputContext;
use handmade_hero_interface::pixel::Pixel;
use handmade_hero_interface::render_context::RenderContext;

#[allow(dead_code)]
#[derive(Debug)]
pub struct ApplicationPlugin;

impl ApplicationPlugin {
    #[unsafe(no_mangle)]
    #[must_use]
    pub extern "Rust" fn create_application() -> Box<dyn Application> {
        Box::new(ApplicationPlugin)
    }

    fn render_rectangle(
        state: &GameState,
        min_x: f32,
        min_y: f32,
        max_x: f32,
        max_y: f32,
        pixel: Pixel,
        buffer: &mut [Pixel],
    ) {
        let width = f32::from(state.width());
        let height = f32::from(state.height());
        let min_x = Self::round_and_bound(min_x, 0f32, width);
        let min_y = Self::round_and_bound(min_y, 0f32, height);
        let max_x = Self::round_and_bound(max_x, 0f32, width);
        let max_y = Self::round_and_bound(max_y, 0f32, height);
        if min_x == max_x || min_y == max_y {
            return;
        }

        let pitch = usize::from(state.width());
        let mut row_offset = min_y * pitch;
        let mut index = row_offset + min_x;
        for _y in min_y..max_y {
            for _x in min_x..max_x {
                buffer[index] = pixel;
                index += 1;
            }
            row_offset += pitch;
            index = row_offset + min_x;
        }
    }

    #[inline]
    #[must_use]
    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_possible_truncation)]
    fn round_and_bound(value: f32, min: f32, max: f32) -> usize {
        value.clamp(min, max).round() as usize
    }
}

impl Application for ApplicationPlugin {
    fn process_input(&self, _context: InputContext<'_>) {}

    fn render(&self, context: RenderContext<'_>) {
        let RenderContext {
            input: _input,
            state,
            buffer,
        } = context;
        let width = f32::from(state.width());
        let height = f32::from(state.height());
        let black = Pixel::from_rgb(0x00, 0x00, 0x00);
        Self::render_rectangle(state, 0f32, 0f32, width, height, black, buffer);
        let white = Pixel::from_rgb(0xFF, 0xFF, 0xFF);
        Self::render_rectangle(state, 480f32, 270f32, 520f32, 310f32, white, buffer);
    }

    fn write_sound(&self, _context: AudioContext<'_>) {}
}
