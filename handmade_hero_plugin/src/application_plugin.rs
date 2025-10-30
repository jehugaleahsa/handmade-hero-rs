use crate::rectangle::Rectangle;
use handmade_hero_interface::application::Application;
use handmade_hero_interface::application_error::Result;
use handmade_hero_interface::audio_context::AudioContext;
use handmade_hero_interface::coordinate_2d::Coordinate2d;
use handmade_hero_interface::f32_color::F32Color;
use handmade_hero_interface::input_context::InputContext;
use handmade_hero_interface::render_context::RenderContext;
use handmade_hero_interface::u8_color::U8Color;

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
        window_bounds: &Rectangle<f32>,
        rectangle: &Rectangle<f32>,
        color: F32Color,
        buffer: &mut [U8Color],
    ) -> Result<()> {
        let rectangle = rectangle.bound_to(window_bounds);
        let rectangle = rectangle.round_to_usize()?;
        if rectangle.width() == 0 {
            // Avoid iterating over rows when there are no columns.
            return Ok(());
        }

        #[allow(clippy::cast_sign_loss)]
        #[allow(clippy::cast_possible_truncation)]
        let pitch = window_bounds.width() as usize;
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
    fn process_input(&self, context: InputContext<'_>) {
        let InputContext { input, state } = context;
        let keyboard = input.keyboard();
        let mut delta_x = 0f32;
        let mut delta_y = 0f32;
        if keyboard.up().ended_down() {
            delta_y -= 1f32;
        }
        if keyboard.down().ended_down() {
            delta_y += 1f32;
        }
        if keyboard.left().ended_down() {
            delta_x -= 1f32;
        }
        if keyboard.right().ended_down() {
            delta_x += 1f32;
        }
        delta_x *= 10f32;
        delta_y *= 10f32;

        let player = state.player();
        let max_height = f32::from(state.height());
        let max_width = f32::from(state.width());
        let x = f32::clamp(player.x() + delta_x, 0f32, max_width);
        let y = f32::clamp(player.y() + delta_y, 0f32, max_height);
        let player_position = Coordinate2d::from_x_y(x, y);
        state.set_player(player_position);
    }

    fn render(&self, context: RenderContext<'_>) {
        let RenderContext {
            input: _input,
            state,
            buffer,
        } = context;

        let width = f32::from(state.width());
        let height = f32::from(state.height());
        let window_bounds = Rectangle::new(0f32, 0f32, height, width);
        let black = F32Color::from(U8Color::from_rgb(0x00, 0x00, 0x00));
        Self::render_rectangle(&window_bounds, &window_bounds, black, buffer).unwrap_or_default(); // Ignore errors

        let player_position = state.player();
        let player = Rectangle::new(player_position.y(), player_position.x(), 10f32, 10f32);
        let player_color = F32Color::from(U8Color::from_rgb(0xFF, 0xFF, 0x00));
        Self::render_rectangle(&window_bounds, &player, player_color, buffer).unwrap_or_default(); // Ignore errors
    }

    fn write_sound(&self, _context: AudioContext<'_>) {}
}
