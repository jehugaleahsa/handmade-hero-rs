use crate::rectangle::Rectangle;
use crate::tile_map::TileMap;
use handmade_hero_interface::application::Application;
use handmade_hero_interface::application_error::Result;
use handmade_hero_interface::audio_context::AudioContext;
use handmade_hero_interface::button_state::ButtonState;
use handmade_hero_interface::controller_state::ControllerState;
use handmade_hero_interface::f32_color::F32Color;
use handmade_hero_interface::game_state::GameState;
use handmade_hero_interface::initialize_context::InitializeContext;
use handmade_hero_interface::input_context::InputContext;
use handmade_hero_interface::point_2d::Point2d;
use handmade_hero_interface::render_context::RenderContext;
use handmade_hero_interface::u8_color::U8Color;

const TILE_ROWS: usize = 9;
const TILE_COLUMNS: usize = 17;
const SOUTH: usize = 0;
const HUB: usize = 1;
const WEST: usize = 2;
const EAST: usize = 3;
const NORTH: usize = 4;

#[allow(dead_code)]
#[derive(Debug)]
pub struct ApplicationPlugin {
    tile_maps: Vec<TileMap>,
    tile_map_index: usize,
}

impl ApplicationPlugin {
    const PLAYER_HEIGHT: f32 = 40f32;
    const PLAYER_WIDTH: f32 = 30f32;

    #[unsafe(no_mangle)]
    #[must_use]
    pub extern "Rust" fn create_application() -> Box<dyn Application> {
        let mut south = TileMap::new(TILE_ROWS, TILE_COLUMNS, 58f32, 56f32, 20f32, 0f32);
        let source_south: [[u32; TILE_COLUMNS]; TILE_ROWS] = [
            [1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        ];
        Self::load_tile_map(&mut south, &source_south);

        let mut hub = TileMap::new(TILE_ROWS, TILE_COLUMNS, 58f32, 56f32, 20f32, 0f32);
        let source_hub: [[u32; TILE_COLUMNS]; TILE_ROWS] = [
            [1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [1, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1],
        ];
        Self::load_tile_map(&mut hub, &source_hub);

        let mut west = TileMap::new(TILE_ROWS, TILE_COLUMNS, 58f32, 56f32, 20f32, 0f32);
        let source_west: [[u32; TILE_COLUMNS]; TILE_ROWS] = [
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 1, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [1, 0, 0, 1, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        ];
        Self::load_tile_map(&mut west, &source_west);

        let mut east = TileMap::new(TILE_ROWS, TILE_COLUMNS, 58f32, 56f32, 20f32, 0f32);
        let source_east: [[u32; TILE_COLUMNS]; TILE_ROWS] = [
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        ];
        Self::load_tile_map(&mut east, &source_east);

        let mut north = TileMap::new(TILE_ROWS, TILE_COLUMNS, 58f32, 56f32, 20f32, 0f32);
        let source_north: [[u32; TILE_COLUMNS]; TILE_ROWS] = [
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 1, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 1, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 1, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1],
        ];
        Self::load_tile_map(&mut north, &source_north);

        Box::new(Self {
            tile_map_index: SOUTH,
            tile_maps: vec![south, hub, west, east, north],
        })
    }

    fn load_tile_map(destination: &mut TileMap, source: &[[u32; TILE_COLUMNS]]) {
        for (row_index, row) in source.iter().enumerate() {
            for (column_index, column) in row.iter().enumerate() {
                let value = *column;
                destination.set(row_index, column_index, value);
            }
        }
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

    fn is_traversable(&self, point: Point2d) -> bool {
        let tile_map = &self.tile_maps[self.tile_map_index];
        #[allow(clippy::cast_sign_loss)]
        #[allow(clippy::cast_possible_truncation)]
        let tile_x = (point.x() / tile_map.tile_width()) as usize;
        let tile_x = usize::clamp(tile_x, 0, tile_map.columns() - 1);
        #[allow(clippy::cast_sign_loss)]
        #[allow(clippy::cast_possible_truncation)]
        let tile_y = (point.y() / tile_map.tile_height()) as usize;
        let tile_y = usize::clamp(tile_y, 0, tile_map.rows() - 1);
        let tile = tile_map.get(tile_y, tile_x);
        tile == 0
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
        let tile_map = &self.tile_maps[self.tile_map_index];
        let upper_left_x = -tile_map.x_offset();
        let upper_left_y = -tile_map.y_offset();
        for row_index in 0..tile_map.rows() {
            for column_index in 0..tile_map.columns() {
                let tile = tile_map.get(row_index, column_index);
                let color = if tile == 0 { grey } else { white };
                #[allow(clippy::cast_precision_loss)]
                let top = row_index as f32 * tile_map.tile_height() + upper_left_y;
                #[allow(clippy::cast_precision_loss)]
                let left = column_index as f32 * tile_map.tile_width() + upper_left_x;
                let tile_rectangle =
                    Rectangle::new(top, left, tile_map.tile_height(), tile_map.tile_width());
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
    fn initialize(&mut self, context: InitializeContext<'_>) {
        let InitializeContext { state } = context;
        let x = f32::from(state.width()) / 2f32;
        let y = f32::from(state.height()) / 2f32;
        state.set_player(Point2d::from_x_y(x, y));
    }

    fn process_input(&mut self, context: InputContext<'_>) {
        let InputContext { input, state } = context;

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

        let player = state.player();
        let min_x = Self::PLAYER_WIDTH / 2f32;
        let max_x = max_width - Self::PLAYER_WIDTH / 2f32;
        let x = f32::clamp(player.x() + delta_x, min_x, max_x);
        let min_y = 0f32;
        let max_y = max_height;
        let y = f32::clamp(player.y() + delta_y, min_y, max_y);

        if y == 0f32 {
            if self.tile_map_index == SOUTH {
                self.tile_map_index = HUB;
                let middle = f32::from(state.width()) / 2f32;
                let bottom = f32::from(state.height());
                state.set_player(Point2d::from_x_y(middle, bottom));
                return;
            } else if self.tile_map_index == HUB {
                self.tile_map_index = NORTH;
                let middle = f32::from(state.width()) / 2f32;
                let bottom = f32::from(state.height());
                state.set_player(Point2d::from_x_y(middle, bottom));
                return;
            }
        } else if y >= f32::from(state.height()) {
            if self.tile_map_index == HUB {
                self.tile_map_index = SOUTH;
                let middle = f32::from(state.width()) / 2f32;
                let top = Self::PLAYER_HEIGHT;
                state.set_player(Point2d::from_x_y(middle, top));
                return;
            } else if self.tile_map_index == NORTH {
                self.tile_map_index = HUB;
                let middle = f32::from(state.width()) / 2f32;
                let top = Self::PLAYER_HEIGHT;
                state.set_player(Point2d::from_x_y(middle, top));
                return;
            }
        } else if x - Self::PLAYER_WIDTH / 2f32 <= 0f32 {
            if self.tile_map_index == HUB {
                self.tile_map_index = WEST;
                let middle = f32::from(state.height()) / 2f32;
                let right = f32::from(state.width()) - Self::PLAYER_WIDTH;
                state.set_player(Point2d::from_x_y(right, middle));
                return;
            } else if self.tile_map_index == EAST {
                self.tile_map_index = HUB;
                let middle = f32::from(state.height()) / 2f32;
                let right = f32::from(state.width()) - Self::PLAYER_WIDTH;
                state.set_player(Point2d::from_x_y(right, middle));
                return;
            }
        } else if x + Self::PLAYER_WIDTH / 2f32 >= f32::from(state.width()) {
            if self.tile_map_index == HUB {
                self.tile_map_index = EAST;
                let middle = f32::from(state.height()) / 2f32;
                let left = Self::PLAYER_WIDTH;
                state.set_player(Point2d::from_x_y(left, middle));
                return;
            } else if self.tile_map_index == WEST {
                self.tile_map_index = HUB;
                let middle = f32::from(state.height()) / 2f32;
                let left = Self::PLAYER_WIDTH;
                state.set_player(Point2d::from_x_y(left, middle));
                return;
            }
        }

        let tile_map = &self.tile_maps[self.tile_map_index];
        let y_offset = y + tile_map.y_offset();
        let min_y = y_offset - Self::PLAYER_HEIGHT / 4f32;
        let max_y = y_offset;

        let x_offset = x + tile_map.x_offset();
        let min_x = x_offset - Self::PLAYER_WIDTH / 2f32 + 1f32;
        let max_x = x_offset + Self::PLAYER_WIDTH / 2f32 - 1f32;

        if !self.is_traversable(Point2d::from_x_y(min_x, min_y)) {
            return;
        }
        if !self.is_traversable(Point2d::from_x_y(min_x, max_y)) {
            return;
        }
        if !self.is_traversable(Point2d::from_x_y(max_x, min_y)) {
            return;
        }
        if !self.is_traversable(Point2d::from_x_y(max_x, max_y)) {
            return;
        }

        let player_position = Point2d::from_x_y(x, y);
        state.set_player(player_position);
    }

    fn render(&mut self, context: RenderContext<'_>) {
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

    fn write_sound(&mut self, _context: AudioContext<'_>) {}
}
