use crate::rectangle::Rectangle;
use handmade_hero_interface::application::Application;
use handmade_hero_interface::application_error::Result;
use handmade_hero_interface::audio_context::AudioContext;
use handmade_hero_interface::f32_color::F32Color;
use handmade_hero_interface::game_state::GameState;
use handmade_hero_interface::input_context::InputContext;
use handmade_hero_interface::render_context::RenderContext;
use handmade_hero_interface::u8_color::U8Color;
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
        rectangle: &Rectangle<f32>,
        color: F32Color,
        buffer: &mut [U8Color],
    ) -> Result<()> {
        let width = f32::from(state.width());
        let height = f32::from(state.height());
        let window_bounds = Rectangle::new(0f32, 0f32, height, width);
        let rectangle = rectangle.bound_to(&window_bounds);
        let rectangle = rectangle.round_to_usize()?;
        if rectangle.width() == 0 {
            // Avoid iterating over rows when there are no columns.
            return Ok(());
        }

        let pitch = usize::from(state.width());
        let color = U8Color::from(color);
        let mut index = rectangle.top() * pitch + rectangle.left();
        for _y in rectangle.top()..rectangle.bottom() {
            let row = index;
            for _x in rectangle.left()..rectangle.right() {
                buffer[index] = color;
                index += 1;
            }
            index = row + pitch;
        }
        Ok(())
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
        let mut random = rand::rng();
        let red = random.random_range(0x00..=0xFF);
        let green = random.random_range(0x00..=0xFF);
        let blue = random.random_range(0x00..=0xFF);
        let color = U8Color::from_rgb(red, green, blue);
        let color = F32Color::from(color);
        let rectangle = Rectangle::new(
            random.random_range(-50f32..(height + 50f32)),
            random.random_range(-50f32..(width + 50f32)),
            random.random_range(0f32..100f32),
            random.random_range(0f32..100f32),
        );
        Self::render_rectangle(state, &rectangle, color, buffer).unwrap_or_default(); // Ignore errors
    }

    fn write_sound(&self, _context: AudioContext<'_>) {}
}
