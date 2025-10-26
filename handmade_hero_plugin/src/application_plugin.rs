use handmade_hero_interface::application::Application;
use handmade_hero_interface::audio_context::AudioContext;
use handmade_hero_interface::game_state::GameState;
use handmade_hero_interface::input_context::InputContext;
use handmade_hero_interface::pixel::Pixel;
use handmade_hero_interface::render_context::RenderContext;
use rand::Rng;

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
        if min_x >= max_x {
            // Avoid iterating over rows when there are no columns.
            return;
        }

        let pitch = usize::from(state.width());
        let mut index = min_y * pitch + min_x;
        for _y in min_y..max_y {
            let row = index;
            for _x in min_x..max_x {
                buffer[index] = pixel;
                index += 1;
            }
            index = row + pitch;
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
        if !state.rendered() {
            let black = Pixel::from_rgb(0x00, 0x00, 0x00);
            Self::render_rectangle(state, 0f32, 0f32, width, height, black, buffer);
            state.set_rendered();
        }

        let mut random = rand::rng();
        let red = random.random_range(0x00..=0xFF);
        let green = random.random_range(0x00..=0xFF);
        let blue = random.random_range(0x00..=0xFF);
        let pixel = Pixel::from_rgb(red, green, blue);
        let min_x = random.random_range(-50f32..(width + 50f32));
        let min_y = random.random_range(-50f32..(height + 50f32));
        let max_x = min_x + random.random_range(0f32..100f32);
        let max_y = min_y + random.random_range(0f32..100f32);
        Self::render_rectangle(state, min_x, min_y, max_x, max_y, pixel, buffer);
    }

    fn write_sound(&self, _context: AudioContext<'_>) {}
}
