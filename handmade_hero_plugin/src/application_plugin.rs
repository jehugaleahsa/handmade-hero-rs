use crate::rectangle::Rectangle;
use handmade_hero_interface::application::Application;
use handmade_hero_interface::application_error::Result;
use handmade_hero_interface::audio_context::AudioContext;
use handmade_hero_interface::button_state::ButtonState;
use handmade_hero_interface::controller_state::ControllerState;
use handmade_hero_interface::coordinate_2d::Coordinate2d;
use handmade_hero_interface::f32_color::F32Color;
use handmade_hero_interface::game_state::GameState;
use handmade_hero_interface::input_context::InputContext;
use handmade_hero_interface::render_context::RenderContext;
use handmade_hero_interface::u8_color::U8Color;

#[allow(dead_code)]
#[derive(Debug)]
pub struct ApplicationPlugin {
    tile_map: [[u32; 17]; 9],
}

impl ApplicationPlugin {
    const PLAYER_HEIGHT: f32 = 40f32;
    const PLAYER_WIDTH: f32 = 30f32;
    const TILE_HEIGHT: f32 = 56f32;
    const TILE_WIDTH: f32 = 58f32;
    const X_OFFSET: f32 = 30f32;

    #[unsafe(no_mangle)]
    #[must_use]
    pub extern "Rust" fn create_application() -> Box<dyn Application> {
        Box::new(Self {
            tile_map: [
                [0, 0, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1],
                [0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1],
                [1, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 1],
                [1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1],
                [0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0],
                [1, 1, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 1],
                [1, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1],
                [1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1],
                [1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1],
            ],
        })
    }

    #[inline]
    #[must_use]
    fn calculate_keyboard_delta_x(keyboard: &ControllerState) -> f32 {
        Self::calculate_input_delta(*keyboard.right(), *keyboard.left())
    }

    #[inline]
    #[must_use]
    fn calculate_keyboard_delta_y(keyboard: &ControllerState) -> f32 {
        Self::calculate_input_delta(*keyboard.down(), *keyboard.up())
    }

    #[must_use]
    fn calculate_input_delta(positive: ButtonState, negative: ButtonState) -> f32 {
        if positive.ended_down() {
            if negative.ended_down() { 0f32 } else { 1f32 }
        } else if negative.ended_down() {
            -1f32
        } else {
            0f32
        }
    }

    #[inline]
    #[must_use]
    fn calculate_controller_delta_x(controller: &ControllerState) -> f32 {
        controller.left_joystick().x()
    }

    #[inline]
    #[must_use]
    fn calculate_controller_delta_y(controller_state: &ControllerState) -> f32 {
        controller_state.left_joystick().y()
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

    fn render_tilemap(&self, window_bounds: &Rectangle<f32>, buffer: &mut [U8Color]) -> Result<()> {
        let black = F32Color::from(U8Color::from_rgb(0x00, 0x00, 0x00));
        Self::render_rectangle(window_bounds, window_bounds, black, buffer).unwrap_or_default(); // Ignore errors

        let white = F32Color::from(U8Color::from_rgb(0xFF, 0xFF, 0xFF));
        let grey = F32Color::from(U8Color::from_rgb(0xCC, 0xCC, 0xCC));
        let upper_left_x = -Self::X_OFFSET;
        let upper_left_y = 0f32;
        for (row_index, row) in self.tile_map.iter().enumerate() {
            for (column_index, tile) in row.iter().enumerate() {
                let color = if *tile == 0 { grey } else { white };
                #[allow(clippy::cast_precision_loss)]
                let top = row_index as f32 * Self::TILE_HEIGHT + upper_left_y;
                #[allow(clippy::cast_precision_loss)]
                let left = column_index as f32 * Self::TILE_WIDTH + upper_left_x;
                let tile_rectangle = Rectangle::new(top, left, Self::TILE_HEIGHT, Self::TILE_WIDTH);
                Self::render_rectangle(window_bounds, &tile_rectangle, color, buffer)?;
            }
        }
        Ok(())
    }

    fn render_player(
        state: &mut GameState,
        window_bounds: &Rectangle<f32>,
        buffer: &mut [U8Color],
    ) -> Result<()> {
        // We want the center of gravity to be the bottom middle of the rectangle.
        // So we render the rectangle above the y position and halfway past the x position.
        // The player's position is constrained while processing input.
        let player_position = state.player();
        let player_y = player_position.y() - Self::PLAYER_HEIGHT;
        let player_x = player_position.x() - (Self::PLAYER_WIDTH / 2f32);
        let player = Rectangle::new(player_y, player_x, Self::PLAYER_HEIGHT, Self::PLAYER_WIDTH);
        let player_color = F32Color::from(U8Color::from_rgb(0xFF, 0xFF, 0x00));
        Self::render_rectangle(window_bounds, &player, player_color, buffer)
    }
}

impl Application for ApplicationPlugin {
    fn process_input(&self, context: InputContext<'_>) {
        let InputContext { input, state } = context;
        let player = state.player();
        if player.x() == 0f32 && player.y() == 0f32 {
            let player_position = Coordinate2d::from_x_y(Self::PLAYER_WIDTH / 2f32, 0f32);
            state.set_player(player_position);
            return;
        }

        let keyboard = input.keyboard();
        let mut delta_x = Self::calculate_keyboard_delta_x(keyboard);
        let mut delta_y = Self::calculate_keyboard_delta_y(keyboard);
        if delta_x == 0f32 && delta_y == 0f32 {
            for controller in input.controllers() {
                if controller.enabled() {
                    delta_x = Self::calculate_controller_delta_x(controller);
                    delta_y = Self::calculate_controller_delta_y(controller);
                    if delta_x != 0f32 || delta_y != 0f32 {
                        break;
                    }
                }
            }
        }
        let frame_duration = state.frame_duration().as_secs_f32();
        let speed = frame_duration * 128f32;
        delta_x *= speed;
        delta_y *= speed;

        if delta_x == 0f32 && delta_y == 0f32 {
            return;
        }

        let max_height = f32::from(state.height());
        let max_width = f32::from(state.width());

        let min_x = Self::PLAYER_WIDTH / 2f32;
        let max_x = max_width - Self::PLAYER_WIDTH / 2f32;
        let x = f32::clamp(player.x() + delta_x, min_x, max_x);
        let min_y = Self::PLAYER_HEIGHT;
        let max_y = max_height;
        let y = f32::clamp(player.y() + delta_y, min_y, max_y);

        #[allow(clippy::cast_sign_loss)]
        #[allow(clippy::cast_possible_truncation)]
        let player_tile_y = usize::clamp((y / Self::TILE_HEIGHT) as usize, 0, 8);

        let x_center = x + Self::X_OFFSET;
        let min_x = x_center - Self::PLAYER_WIDTH / 2f32 + 1f32;
        let max_x = x_center + Self::PLAYER_WIDTH / 2f32 - 1f32;

        #[allow(clippy::cast_sign_loss)]
        #[allow(clippy::cast_possible_truncation)]
        let min_player_tile_x = usize::clamp((min_x / Self::TILE_WIDTH) as usize, 0, 15);
        let min_tile = self.tile_map[player_tile_y][min_player_tile_x];
        if min_tile != 0 {
            return;
        }

        #[allow(clippy::cast_sign_loss)]
        #[allow(clippy::cast_possible_truncation)]
        let max_player_tile_x = usize::clamp((max_x / Self::TILE_WIDTH) as usize, 0, 15);
        let max_tile = self.tile_map[player_tile_y][max_player_tile_x];
        if max_tile != 0 {
            return;
        }

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

        self.render_tilemap(&window_bounds, buffer)
            .unwrap_or_default(); // Ignore errors

        Self::render_player(state, &window_bounds, buffer).unwrap_or_default(); // Ignore errors
    }

    fn write_sound(&self, _context: AudioContext<'_>) {}
}
