use handmade_hero_interface::application::Application;
use handmade_hero_interface::application_error::{ApplicationError, Result};
use handmade_hero_interface::audio_context::AudioContext;
use handmade_hero_interface::button_state::ButtonState;
use handmade_hero_interface::color::Color;
use handmade_hero_interface::controller_state::ControllerState;
use handmade_hero_interface::game_state::GameState;
use handmade_hero_interface::initialize_context::InitializeContext;
use handmade_hero_interface::input_context::InputContext;
use handmade_hero_interface::input_state::InputState;
use handmade_hero_interface::point_2d::Point2d;
use handmade_hero_interface::rectangle::Rectangle;
use handmade_hero_interface::render_context::RenderContext;
use handmade_hero_interface::tile_map::TileMap;
use handmade_hero_interface::units::si::length::{Length, pixel};
use handmade_hero_interface::units::si::time::Time;
use handmade_hero_interface::world::{TileMapKey, World};
use uom::si::length::meter;
use uom::si::time::second;

#[derive(Debug)]
pub struct ApplicationPlugin {}

impl ApplicationPlugin {
    #[unsafe(no_mangle)]
    #[must_use]
    pub extern "Rust" fn create_application() -> Box<dyn Application> {
        Box::new(Self {})
    }

    fn initialize_direct(state: &mut GameState) {
        let player_bounds = state.player().render_bounds();
        let width = state.width().get::<pixel>() + player_bounds.width() / 1.5f32;
        let height = state.height().get::<pixel>() - player_bounds.height() / 3.5f32;
        let x = width / 2f32;
        let y = height / 2f32;
        let new_player_bounds = player_bounds.moved_to(x, y);
        state.player_mut().set_render_bounds(new_player_bounds);

        let world = state.world_mut();
        let hub = world.add_tile_map(TileMapKey { x: 0, y: 0 }); // Origin
        Self::load_hub_tile_map(hub);
        let south = world.add_tile_map(TileMapKey { x: 0, y: -1 });
        Self::load_south_tile_map(south);
        let west = world.add_tile_map(TileMapKey { x: -1, y: 0 });
        Self::load_west_tile_map(west);
        let east = world.add_tile_map(TileMapKey { x: 1, y: 0 });
        Self::load_east_tile_map(east);
        let north = world.add_tile_map(TileMapKey { x: 0, y: 1 });
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
        // In our world coordinate system, the y-coordinates increase from the bottom up.
        // But in memory, the y-coordinates increase from the top down. Therefore, when
        // we copy our hard-coded tile map arrays, we flip the coordinate of each row.
        let mut index = 0;
        for row_index in 0..row_count {
            for column_index in 0..column_count {
                let value = source[index];
                let destination_row_index = row_count - row_index - 1;
                destination[(destination_row_index, column_index)] = value;
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
        let updated_player_bounds = state.player().render_bounds().shifted(delta_x, delta_y);
        let width = state.width().get::<pixel>();
        let height = state.height().get::<pixel>();
        let world = state.world_mut();
        if let Some(updated_player) = world.try_navigate(updated_player_bounds, width, height) {
            state.player_mut().set_render_bounds(updated_player);
            return;
        }

        // Check that the player isn't trying to walk through a wall.
        let bound_height = updated_player_bounds.height() / 4f32;
        let player_bounds =
            updated_player_bounds.resized(bound_height, updated_player_bounds.width());
        if !world.is_traversable_rectangle(player_bounds) {
            return;
        }

        state.player_mut().set_render_bounds(updated_player_bounds);
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
        let frame_duration = state.frame_duration();
        let max_speed = Length::new::<meter>(3f32) / Time::new::<second>(1f32);
        let max_distance = frame_duration * max_speed;
        let max_distance_px = max_distance.get::<pixel>();
        delta_x *= max_distance_px;
        delta_y *= -max_distance_px;
        (delta_x, delta_y)
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
        controller.left_joystick().x_ratio()
    }

    #[inline]
    #[must_use]
    fn calculate_controller_delta_y(controller_state: &ControllerState) -> f32 {
        controller_state.left_joystick().y_ratio()
    }

    fn render_direct(state: &GameState, buffer: &mut [Color<u8>]) {
        let width = state.width();
        let height = state.height();
        let window_bounds = Rectangle::new(0f32, 0f32, height.get::<pixel>(), width.get::<pixel>());

        Self::render_tilemap(state, &window_bounds, buffer).unwrap_or_default(); // Ignore errors

        Self::render_player(state, &window_bounds, buffer).unwrap_or_default(); // Ignore errors
    }

    fn render_tilemap(
        state: &GameState,
        window_bounds: &Rectangle<f32>,
        buffer: &mut [Color<u8>],
    ) -> Result<()> {
        let world = state.world();
        let Some(tile_map) = world.tile_maps.get(&world.current_tile_map_key) else {
            return Err(ApplicationError::new("Fell out of the world"));
        };

        // When rendering the tile map, our goal is to keep the player relatively close to the
        // center of the screen. We also want to transition between tile maps smoothly, without
        // suddenly jumping the user to a completely new screen. The player can also see surrounding
        // tile maps and, without the presence of walls, they should be oblivious when they transit
        // from one tile map to the other. Each tile map has a key, which is an (X, Y) coordinate
        // of its relative position to other tile maps.
        //
        // Having relative tile map coordinates allows us to pick a maximum number of tiles up
        // and down, left and right, to render at one time. We determine the player's current tile
        // and count up to see if we need to start in a tile map above us. Similarly, we look to the
        // left to see if we need to start in a tile map to our left. If the player moves toward the
        // bottom or right of the screen, we want to spill over to the bottom and right tile map in
        // a similar fashion.
        //
        // It's possible a tile map will not have a neighbor to the left, right, top, or bottom.
        // When this happens, we switch our strategy, rendering more of the current tile map. This
        // means once the player gets past the center, they will no longer stay in the center and
        // start moving toward the outer edge. This avoids rendering a bunch of emptiness.
        let white = Color::from(Color::from_rgb(0xFF, 0xFF, 0xFF));
        let grey = Color::from(Color::from_rgb(0xCC, 0xCC, 0xCC));
        let black = Color::from(Color::from_rgb(0x00, 0x00, 0x00));
        let height = state.height();
        let tile_size = world.tile_size;
        let player_bounds = state.player().render_bounds();
        let player_center = Point2d::from_x_y(
            player_bounds.left() + player_bounds.width() / 2f32,
            player_bounds.bottom(),
        );
        for row_index in 0..world.rows {
            for column_index in 0..world.columns {
                let tile = tile_map[(row_index, column_index)];
                #[allow(clippy::cast_precision_loss)]
                let row_index = row_index as f32;
                #[allow(clippy::cast_precision_loss)]
                let column_index = column_index as f32;
                let left = column_index * tile_size;
                let bottom = row_index * tile_size;
                let tile_rectangle = Rectangle::new(
                    bottom.get::<pixel>(),
                    left.get::<pixel>(),
                    tile_size.get::<pixel>(),
                    tile_size.get::<pixel>(),
                );
                let color = if tile_rectangle.contains_point(player_center) {
                    black
                } else if tile == 0 {
                    grey
                } else {
                    white
                };
                let tile_rectangle = tile_rectangle.moved_to(
                    tile_rectangle.left(),
                    height.get::<pixel>() - tile_rectangle.top(),
                );
                let tile_rectangle = tile_rectangle.shifted(
                    world.x_offset.get::<pixel>(),
                    -world.y_offset.get::<pixel>(),
                );
                Self::render_rectangle(window_bounds, &tile_rectangle, color, buffer)?;
            }
        }
        Ok(())
    }

    fn render_player(
        state: &GameState,
        window_bounds: &Rectangle<f32>,
        buffer: &mut [Color<u8>],
    ) -> Result<()> {
        // We want the center of gravity to be the bottom middle of the rectangle.
        // So we render the rectangle above the y position and halfway past the x position.
        // The player's position is constrained while processing input.
        let world = state.world();
        let height = state.height();
        let player = state.player();
        let player_bounds = player.render_bounds();
        let player_bounds = player_bounds.moved_to(
            player_bounds.left(),
            height.get::<pixel>() - player_bounds.top(),
        );
        let player_bounds = player_bounds.shifted(
            world.x_offset.get::<pixel>(),
            -world.y_offset.get::<pixel>(),
        );
        Self::render_rectangle(window_bounds, &player_bounds, player.color(), buffer)
    }

    fn render_rectangle(
        window_bounds: &Rectangle<f32>,
        rectangle: &Rectangle<f32>,
        color: Color<f32>,
        buffer: &mut [Color<u8>],
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
        let color = Color::from(color);
        let mut index = rectangle.bottom() * pitch + rectangle.left();
        for _y in rectangle.bottom()..rectangle.top() {
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
