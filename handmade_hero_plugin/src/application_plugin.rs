use crate::rectangle::Rectangle;
use crate::tile_map::TileMap;
use crate::world::World;
use handmade_hero_interface::application::Application;
use handmade_hero_interface::application_error::{ApplicationError, Result};
use handmade_hero_interface::audio_context::AudioContext;
use handmade_hero_interface::button_state::ButtonState;
use handmade_hero_interface::controller_state::ControllerState;
use handmade_hero_interface::f32_color::F32Color;
use handmade_hero_interface::game_state::GameState;
use handmade_hero_interface::initialize_context::InitializeContext;
use handmade_hero_interface::input_context::InputContext;
use handmade_hero_interface::input_state::InputState;
use handmade_hero_interface::point_2d::Point2d;
use handmade_hero_interface::render_context::RenderContext;
use handmade_hero_interface::u8_color::U8Color;
use std::collections::HashMap;

const SOUTH: usize = 0;
const HUB: usize = 1;
const WEST: usize = 2;
const EAST: usize = 3;
const NORTH: usize = 4;

#[derive(Debug)]
pub struct ApplicationPlugin {
    world: World,
}

impl ApplicationPlugin {
    const PLAYER_HEIGHT: f32 = 40f32;
    const PLAYER_WIDTH: f32 = 30f32;

    #[unsafe(no_mangle)]
    #[must_use]
    pub extern "Rust" fn create_application() -> Box<dyn Application> {
        let mut world = World {
            rows: World::TILE_ROWS,
            columns: World::TILE_COLUMNS,
            tile_height: 58f32,
            tile_width: 56f32,
            x_offset: 20f32,
            y_offset: 0f32,
            current_tile_map_id: SOUTH,
            tile_maps: HashMap::new(),
        };

        let south = world.add_tile_map(SOUTH);
        let source_south: [[u32; World::TILE_COLUMNS]; World::TILE_ROWS] = [
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
        Self::load_tile_map(south, &source_south);

        let hub = world.add_tile_map(HUB);
        let source_hub: [[u32; World::TILE_COLUMNS]; World::TILE_ROWS] = [
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
        Self::load_tile_map(hub, &source_hub);

        let west = world.add_tile_map(WEST);
        let source_west: [[u32; World::TILE_COLUMNS]; World::TILE_ROWS] = [
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
        Self::load_tile_map(west, &source_west);

        let east = world.add_tile_map(EAST);
        let source_east: [[u32; World::TILE_COLUMNS]; World::TILE_ROWS] = [
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
        Self::load_tile_map(east, &source_east);

        let north = world.add_tile_map(NORTH);
        let source_north: [[u32; World::TILE_COLUMNS]; World::TILE_ROWS] = [
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
        Self::load_tile_map(north, &source_north);

        Box::new(Self { world })
    }

    fn load_tile_map(destination: &mut TileMap, source: &[[u32; World::TILE_COLUMNS]]) {
        for (row_index, row) in source.iter().enumerate() {
            for (column_index, column) in row.iter().enumerate() {
                let value = *column;
                destination.set(row_index, column_index, value);
            }
        }
    }

    fn calculate_delta_x_y(input: &InputState, state: &GameState) -> (f32, f32) {
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
        (delta_x, delta_y)
    }

    fn calculate_player_x_y(state: &GameState, delta_x: f32, delta_y: f32) -> Point2d {
        let max_height = f32::from(state.height());
        let max_width = f32::from(state.width());

        let player = state.player();
        let min_x = Self::PLAYER_WIDTH / 2f32;
        let max_x = max_width - Self::PLAYER_WIDTH / 2f32;
        let x = f32::clamp(player.x() + delta_x, min_x, max_x);
        let min_y = 0f32;
        let max_y = max_height;
        let y = f32::clamp(player.y() + delta_y, min_y, max_y);
        Point2d::from_x_y(x, y)
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
        let world = &self.world;
        let Some(tile_map) = &world.tile_maps.get(&world.current_tile_map_id) else {
            return false;
        };
        #[allow(clippy::cast_sign_loss)]
        #[allow(clippy::cast_possible_truncation)]
        let tile_x = (point.x() / world.tile_width) as usize;
        let tile_x = usize::clamp(tile_x, 0, world.columns - 1);
        #[allow(clippy::cast_sign_loss)]
        #[allow(clippy::cast_possible_truncation)]
        let tile_y = (point.y() / world.tile_height) as usize;
        let tile_y = usize::clamp(tile_y, 0, world.rows - 1);
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
        let world = &self.world;
        let tile_map = world
            .tile_maps
            .get(&world.current_tile_map_id)
            .ok_or_else(|| ApplicationError::new("Fell out of the world"))?;
        let upper_left_x = -world.x_offset;
        let upper_left_y = -world.y_offset;
        for row_index in 0..world.rows {
            for column_index in 0..world.columns {
                let tile = tile_map.get(row_index, column_index);
                let color = if tile == 0 { grey } else { white };
                #[allow(clippy::cast_precision_loss)]
                let top = row_index as f32 * world.tile_height + upper_left_y;
                #[allow(clippy::cast_precision_loss)]
                let left = column_index as f32 * world.tile_width + upper_left_x;
                let tile_rectangle = Rectangle::new(top, left, world.tile_height, world.tile_width);
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

        let (delta_x, delta_y) = Self::calculate_delta_x_y(input, state);
        if delta_x == 0f32 && delta_y == 0f32 {
            return;
        }

        let updated_position = Self::calculate_player_x_y(state, delta_x, delta_y);
        let x = updated_position.x();
        let y = updated_position.y();

        let world = &mut self.world;
        if y == 0f32 {
            if world.current_tile_map_id == SOUTH {
                world.current_tile_map_id = HUB;
                let middle = f32::from(state.width()) / 2f32;
                let bottom = f32::from(state.height());
                state.set_player(Point2d::from_x_y(middle, bottom));
                return;
            } else if world.current_tile_map_id == HUB {
                world.current_tile_map_id = NORTH;
                let middle = f32::from(state.width()) / 2f32;
                let bottom = f32::from(state.height());
                state.set_player(Point2d::from_x_y(middle, bottom));
                return;
            }
        } else if y >= f32::from(state.height()) {
            if world.current_tile_map_id == HUB {
                world.current_tile_map_id = SOUTH;
                let middle = f32::from(state.width()) / 2f32;
                let top = Self::PLAYER_HEIGHT;
                state.set_player(Point2d::from_x_y(middle, top));
                return;
            } else if world.current_tile_map_id == NORTH {
                world.current_tile_map_id = HUB;
                let middle = f32::from(state.width()) / 2f32;
                let top = Self::PLAYER_HEIGHT;
                state.set_player(Point2d::from_x_y(middle, top));
                return;
            }
        } else if x - Self::PLAYER_WIDTH / 2f32 <= 0f32 {
            if world.current_tile_map_id == HUB {
                world.current_tile_map_id = WEST;
                let middle = f32::from(state.height()) / 2f32;
                let right = f32::from(state.width()) - Self::PLAYER_WIDTH;
                state.set_player(Point2d::from_x_y(right, middle));
                return;
            } else if world.current_tile_map_id == EAST {
                world.current_tile_map_id = HUB;
                let middle = f32::from(state.height()) / 2f32;
                let right = f32::from(state.width()) - Self::PLAYER_WIDTH;
                state.set_player(Point2d::from_x_y(right, middle));
                return;
            }
        } else if x + Self::PLAYER_WIDTH / 2f32 >= f32::from(state.width()) {
            if world.current_tile_map_id == HUB {
                world.current_tile_map_id = EAST;
                let middle = f32::from(state.height()) / 2f32;
                let left = Self::PLAYER_WIDTH;
                state.set_player(Point2d::from_x_y(left, middle));
                return;
            } else if world.current_tile_map_id == WEST {
                world.current_tile_map_id = HUB;
                let middle = f32::from(state.height()) / 2f32;
                let left = Self::PLAYER_WIDTH;
                state.set_player(Point2d::from_x_y(left, middle));
                return;
            }
        }

        let y_offset = y + world.y_offset;
        let min_y = y_offset - Self::PLAYER_HEIGHT / 4f32;
        let max_y = y_offset;

        let x_offset = x + world.x_offset;
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
