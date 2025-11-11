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
use handmade_hero_interface::rectangle::Rectangle;
use handmade_hero_interface::render_context::RenderContext;
use handmade_hero_interface::tile_map::TileMap;
use handmade_hero_interface::u8_color::U8Color;
use handmade_hero_interface::units::si::length::pixel;
use handmade_hero_interface::world::{TileMapKey, World};

#[derive(Debug)]
pub struct ApplicationPlugin {}

impl ApplicationPlugin {
    #[unsafe(no_mangle)]
    #[must_use]
    pub extern "Rust" fn create_application() -> Box<dyn Application> {
        Box::new(Self {})
    }

    fn initialize_direct(state: &mut GameState) {
        let x = f32::from(state.width()) / 2f32;
        let y = f32::from(state.height()) / 2f32;
        let new_player = state.player().move_to(x, y);
        state.set_player(new_player);

        let world = state.world_mut();
        let south = world.add_tile_map(TileMapKey::South);
        Self::load_south_tile_map(south);
        let hub = world.add_tile_map(TileMapKey::Hub);
        Self::load_hub_tile_map(hub);
        let west = world.add_tile_map(TileMapKey::West);
        Self::load_west_tile_map(west);
        let east = world.add_tile_map(TileMapKey::East);
        Self::load_east_tile_map(east);
        let north = world.add_tile_map(TileMapKey::North);
        Self::load_north_tile_map(north);
    }

    fn load_south_tile_map(south: &mut TileMap) {
        let source_south: [[u32; World::TILE_COLUMNS]; World::TILE_ROWS] = [
            [1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 1, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        ];
        Self::load_tile_map(
            south,
            source_south.as_flattened(),
            World::TILE_ROWS,
            World::TILE_COLUMNS,
        );
    }

    fn load_hub_tile_map(hub: &mut TileMap) {
        let source_hub: [[u32; World::TILE_COLUMNS]; World::TILE_ROWS] = [
            [1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1],
        ];
        Self::load_tile_map(
            hub,
            source_hub.as_flattened(),
            World::TILE_ROWS,
            World::TILE_COLUMNS,
        );
    }

    fn load_west_tile_map(west: &mut TileMap) {
        let source_west: [[u32; World::TILE_COLUMNS]; World::TILE_ROWS] = [
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        ];
        Self::load_tile_map(
            west,
            source_west.as_flattened(),
            World::TILE_ROWS,
            World::TILE_COLUMNS,
        );
    }

    fn load_east_tile_map(east: &mut TileMap) {
        let source_east: [[u32; World::TILE_COLUMNS]; World::TILE_ROWS] = [
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1],
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1],
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 0, 1],
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        ];
        Self::load_tile_map(
            east,
            source_east.as_flattened(),
            World::TILE_ROWS,
            World::TILE_COLUMNS,
        );
    }

    fn load_north_tile_map(north: &mut TileMap) {
        let source_north: [[u32; World::TILE_COLUMNS]; World::TILE_ROWS] = [
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 1, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1],
        ];
        Self::load_tile_map(
            north,
            source_north.as_flattened(),
            World::TILE_ROWS,
            World::TILE_COLUMNS,
        );
    }

    fn load_tile_map(
        destination: &mut TileMap,
        source: &[u32],
        row_count: usize,
        column_count: usize,
    ) {
        let mut index = 0;
        for row_index in 0..row_count {
            for column_index in 0..column_count {
                let value = source[index];
                destination.set(row_index, column_index, value);
                index += 1;
            }
        }
    }

    fn process_input_direct(input: &InputState, state: &mut GameState) {
        let (delta_x, delta_y) = Self::calculate_delta_x_y(input, state);
        if delta_x == 0f32 && delta_y == 0f32 {
            return;
        }

        // Handle the player moving off the screen through a doorway.
        let updated_player = Self::calculate_player_x_y(state, delta_x, delta_y);
        let width = f32::from(state.width());
        let height = f32::from(state.height());
        let world = state.world_mut();
        if let Some(updated_player) = world.try_navigate(updated_player, width, height) {
            state.set_player(updated_player);
            return;
        }

        // Check that the player isn't trying to walk through a wall.
        let x = updated_player.left();
        let y = updated_player.top();
        let y_offset = y - world.y_offset;
        let min_y = y_offset - updated_player.height() / 4f32;
        let max_y = y_offset;

        let x_offset = x - world.x_offset;
        let min_x = x_offset - updated_player.width() / 2f32 + 1f32;
        let max_x = x_offset + updated_player.width() / 2f32 - 1f32;
        if !world.is_traversable(Point2d::from_x_y(min_x, min_y)) {
            return;
        }
        if !world.is_traversable(Point2d::from_x_y(min_x, max_y)) {
            return;
        }
        if !world.is_traversable(Point2d::from_x_y(max_x, min_y)) {
            return;
        }
        if !world.is_traversable(Point2d::from_x_y(max_x, max_y)) {
            return;
        }

        state.set_player(updated_player);
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

    fn calculate_player_x_y(state: &GameState, delta_x: f32, delta_y: f32) -> Rectangle<f32> {
        let max_height = f32::from(state.height());
        let max_width = f32::from(state.width());

        let player = state.player();
        let min_x = 0f32;
        let max_x = max_width;
        let x = f32::clamp(player.left() + delta_x, min_x, max_x);
        let min_y = 0f32;
        let max_y = max_height;
        let y = f32::clamp(player.top() + delta_y, min_y, max_y);
        player.move_to(x, y)
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

    fn render_direct(state: &GameState, buffer: &mut [U8Color]) {
        let width = f32::from(state.width());
        let height = f32::from(state.height());
        let window_bounds = Rectangle::new(0f32, 0f32, height, width);

        Self::render_tilemap(state, &window_bounds, buffer).unwrap_or_default(); // Ignore errors

        Self::render_player(state, &window_bounds, buffer).unwrap_or_default(); // Ignore errors
    }

    fn render_tilemap(
        state: &GameState,
        window_bounds: &Rectangle<f32>,
        buffer: &mut [U8Color],
    ) -> Result<()> {
        let black = F32Color::from(U8Color::from_rgb(0x00, 0x00, 0x00));
        Self::render_rectangle(window_bounds, window_bounds, black, buffer).unwrap_or_default(); // Ignore errors

        let white = F32Color::from(U8Color::from_rgb(0xFF, 0xFF, 0xFF));
        let grey = F32Color::from(U8Color::from_rgb(0xCC, 0xCC, 0xCC));
        let world = state.world();
        let Some(tile_map) = world.tile_maps.get(&world.current_tile_map_id) else {
            return Err(ApplicationError::new("Fell out of the world"));
        };

        let upper_left_x = world.x_offset;
        let upper_left_y = world.y_offset;
        for row_index in 0..world.rows {
            for column_index in 0..world.columns {
                let tile = tile_map.get(row_index, column_index);
                let color = if tile == 0 { grey } else { white };
                #[allow(clippy::cast_precision_loss)]
                let tile_size = world.tile_size.get::<pixel>();
                #[allow(clippy::cast_precision_loss)]
                let top = row_index as f32 * tile_size + upper_left_y;
                #[allow(clippy::cast_precision_loss)]
                let left = column_index as f32 * tile_size + upper_left_x;
                let tile_rectangle = Rectangle::new(top, left, tile_size, tile_size);
                Self::render_rectangle(window_bounds, &tile_rectangle, color, buffer)?;
            }
        }
        Ok(())
    }

    fn render_player(
        state: &GameState,
        window_bounds: &Rectangle<f32>,
        buffer: &mut [U8Color],
    ) -> Result<()> {
        // We want the center of gravity to be the bottom middle of the rectangle.
        // So we render the rectangle above the y position and halfway past the x position.
        // The player's position is constrained while processing input.
        let player = state.player();
        let player_y = player.top() - player.height();
        let player_x = player.left() - (player.width() / 2f32);
        let player = player.move_to(player_x, player_y);
        let player_color = F32Color::from(U8Color::from_rgb(0xFF, 0xFF, 0x00));
        Self::render_rectangle(window_bounds, &player, player_color, buffer)
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
    #[inline]
    fn initialize(&self, context: InitializeContext<'_>) {
        let InitializeContext { state } = context;
        Self::initialize_direct(state);
    }

    #[inline]
    fn process_input(&self, context: InputContext<'_>) {
        let InputContext { input, state } = context;
        Self::process_input_direct(input, state);
    }

    #[inline]
    fn render(&self, context: RenderContext<'_>) {
        let RenderContext {
            input: _input,
            state,
            buffer,
        } = context;

        Self::render_direct(state, buffer);
    }

    #[inline]
    fn write_sound(&self, _context: AudioContext<'_>) {}
}
