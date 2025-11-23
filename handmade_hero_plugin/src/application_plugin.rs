use handmade_hero_interface::application::Application;
use handmade_hero_interface::application_error::Result;
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
use handmade_hero_interface::tile_map_key::TileMapKey;
use handmade_hero_interface::units::si::length::{Length, pixel};
use handmade_hero_interface::units::si::time::Time;
use handmade_hero_interface::world::World;
use handmade_hero_interface::world_coordinate::WorldCoordinate;
use std::cmp::Ordering;
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
        // Put the player somewhere in the middle
        let width = state.width().get::<pixel>();
        let height = state.height().get::<pixel>();
        let x = width / 2f32 + state.player().render_bounds().width() / 3f32;
        let y = height / 2f32;
        let tile_map_coordinates = state
            .world()
            .get_tile_map_coordinate(Point2d::from_x_y(x, y));
        let new_coordinates = WorldCoordinate::new(
            state.world(),
            state.player().tile_map_key(),
            tile_map_coordinates,
        );
        state.player_mut().set_coordinates(new_coordinates);

        // Load the world tile maps
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

        let world = state.world();
        let old_coordinates = state.player().coordinate();
        let new_coordinates = old_coordinates.shifted(delta_x, delta_y);
        if !world.is_traversable(&new_coordinates, state.player().collision_bounds()) {
            return;
        }

        state.player_mut().set_coordinates(new_coordinates);
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
        let world = state.world();
        let player_coordinate = state.player().coordinate();
        let tile_map_x = player_coordinate.tile_map_key().x();
        let tile_x = player_coordinate.tile_x();
        let tile_map_y = player_coordinate.tile_map_key().y();
        let tile_y = player_coordinate.tile_y();
        let (start_tile_map_x, start_tile_x) =
            Self::determine_start(tile_map_x, tile_x, world.columns());
        let (start_tile_map_y, start_tile_y) =
            Self::determine_start(tile_map_y, tile_y, world.rows());

        let tile_size = world.tile_size;
        let mut tile_map_y = start_tile_map_y;
        let mut tile_y = start_tile_y;
        for row_index in 0..world.rows {
            let mut tile_map_x = start_tile_map_x;
            let mut tile_x = start_tile_x;
            for column_index in 0..world.columns {
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

                let tile_map_key = TileMapKey {
                    x: tile_map_x,
                    y: tile_map_y,
                };
                let color = Self::determine_tile_color(
                    world,
                    player_coordinate,
                    tile_map_key,
                    tile_x,
                    tile_y,
                );
                let tile_rectangle = tile_rectangle.moved_to(
                    tile_rectangle.left(),
                    state.height().get::<pixel>() - tile_rectangle.top(),
                );
                let tile_rectangle = tile_rectangle.shifted(
                    world.x_offset.get::<pixel>(),
                    -world.y_offset.get::<pixel>(),
                );
                Self::render_rectangle(window_bounds, &tile_rectangle, color, buffer)?;

                tile_x += 1;
                if tile_x >= world.columns {
                    tile_map_x += 1;
                    tile_x = 0;
                }
            }

            tile_y += 1;
            if tile_y >= world.rows {
                tile_map_y += 1;
                tile_y = 0;
            }
        }
        Ok(())
    }

    fn determine_tile_color(
        world: &World,
        player_coordinate: &WorldCoordinate,
        tile_map_key: TileMapKey,
        tile_x: usize,
        tile_y: usize,
    ) -> Color<f32> {
        let tile = world
            .get_tile_map(tile_map_key)
            .map(|tm| tm[(tile_y, tile_x)]);
        if let Some(tile) = tile {
            if tile_map_key == player_coordinate.tile_map_key()
                && tile_y == player_coordinate.tile_y()
                && tile_x == player_coordinate.tile_x()
            {
                Color::from(Color::from_rgb(0x00, 0x00, 0x00)) // black
            } else if tile == 0 {
                Color::from(Color::from_rgb(0xCC, 0xCC, 0xCC)) // grey
            } else {
                Color::from(Color::from_rgb(0xFF, 0xFF, 0xFF)) // white
            }
        } else {
            Color::from(Color::from_rgb(0x00, 0x00, 0x00)) // black
        }
    }

    fn render_player(
        state: &GameState,
        window_bounds: &Rectangle<f32>,
        buffer: &mut [Color<u8>],
    ) -> Result<()> {
        let world = state.world();
        let player = state.player();
        let player_coordinate = player.coordinate();
        let tile_size = world.tile_size().get::<pixel>();
        let x_offset = Self::determine_player_offset(
            player_coordinate.tile_map_key().x(),
            player_coordinate.tile_x(),
            tile_size,
            world.columns(),
        );
        let y_offset = Self::determine_player_offset(
            player_coordinate.tile_map_key().y(),
            player_coordinate.tile_y(),
            tile_size,
            world.rows(),
        );

        let player_bounds = player.render_bounds();
        let player_bounds = player_bounds.shifted(x_offset, y_offset);
        let height = state.height();
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

    #[allow(clippy::cast_precision_loss)]
    #[allow(clippy::cast_possible_wrap)]
    fn determine_player_offset(
        tile_map: isize,
        tile: usize,
        tile_size: f32,
        max_tile: usize,
    ) -> f32 {
        let (start_tile_map, start_tile) = Self::determine_start(tile_map, tile, max_tile);
        let tile_map_diff = tile_map - start_tile_map;
        let tile_map_diff = tile_map_diff as f32 * max_tile as f32 * tile_size;
        let tile_diff = tile as isize - start_tile as isize;
        let tile_diff = tile_diff as f32 * tile_size;
        tile_map_diff + tile_diff
    }

    #[allow(clippy::cast_possible_wrap)]
    #[allow(clippy::cast_sign_loss)]
    fn determine_start(tile_map: isize, tile: usize, max_tile: usize) -> (isize, usize) {
        let current_tile_map = tile_map;
        let tile = tile as isize;
        let max_tile = max_tile as isize;
        let middle = max_tile / 2;
        match tile.cmp(&middle) {
            Ordering::Less => (current_tile_map - 1, (max_tile - (middle - tile)) as usize),
            Ordering::Greater => (current_tile_map, (tile - middle) as usize),
            Ordering::Equal => (current_tile_map, 0),
        }
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
