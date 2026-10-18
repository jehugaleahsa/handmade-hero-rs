use crate::chunk_key::ChunkKey;
use crate::dimensions::Dimensions;
use crate::player::Player;
use crate::tile_type::TileType;
use crate::viewport::Viewport;
use crate::world::World;
use crate::world_coordinate::WorldCoordinate;
use handmade_hero_interface::application::Application;
use handmade_hero_interface::application_error::{ApplicationError, Result};
use handmade_hero_interface::audio_context::AudioContext;
use handmade_hero_interface::back_buffer::BackBuffer;
use handmade_hero_interface::button::Button;
use handmade_hero_interface::button_state::ButtonState;
use handmade_hero_interface::color::Color;
use handmade_hero_interface::controller_state::ControllerState;
use handmade_hero_interface::display::Display;
use handmade_hero_interface::display_settings::DisplaySettings;
use handmade_hero_interface::game_state::GameState;
use handmade_hero_interface::initialize_context::InitializeContext;
use handmade_hero_interface::input_state::InputState;
use handmade_hero_interface::key::Key;
use handmade_hero_interface::monitor::Monitor;
use handmade_hero_interface::monitor_mode::MonitorMode;
use handmade_hero_interface::plugin_state::PluginState;
use handmade_hero_interface::rectangle::Rectangle;
use handmade_hero_interface::stereo_sample::StereoSample;
use handmade_hero_interface::units::si::frequency::Frequency;
use handmade_hero_interface::units::si::length::Length;
use handmade_hero_interface::units::si::time::Time;
use handmade_hero_interface::update_render_context::UpdateRenderContext;
use rand::prelude::*;
use std::f32;
use uom::num::Zero;
use uom::si::frequency::hertz;
use uom::si::length::meter;
use uom::si::ratio::ratio;
use uom::si::time::second;

use crate::plugin_game_state::PluginGameState;

#[derive(Debug)]
pub struct ApplicationPlugin {}

impl ApplicationPlugin {
    // For now, assume stairs are always on the same tile
    const STAIRS_X: u8 = 3;
    const STAIRS_Y: u8 = 3;

    #[unsafe(no_mangle)]
    #[must_use]
    pub extern "Rust" fn create_application() -> Box<dyn Application> {
        Box::new(Self {})
    }

    fn initialize_direct(
        #[allow(unused_variables)] game_state: &GameState,
        plugin_state: &mut PluginGameState,
    ) {
        #[cfg(feature = "audio_debug")]
        Self::initialize_audio_debugging(game_state, plugin_state);

        let world = plugin_state.world();
        let player = plugin_state.player();
        let new_coordinates = WorldCoordinate::new(plugin_state.world(), player.chunk_key());
        let tile_size = world.tile_size();
        // A new coordinate already sits at the center of its tile, so stepping a whole number
        // of tiles lands on the center of the middle tile.
        let x = f32::from(world.columns() / 2) * tile_size;
        let y = f32::from(world.rows() / 2) * tile_size;
        let new_coordinates = new_coordinates.shifted(x, y, tile_size);
        plugin_state.player_mut().set_coordinates(new_coordinates);
    }

    #[cfg(feature = "audio_debug")]
    fn initialize_audio_debugging(game_state: &GameState, plugin_state: &mut PluginGameState) {
        // Track how many audio cursors to render at one time
        let audio = plugin_state.audio_mut();
        let update_frequency = game_state.game_update_frequency().get::<hertz>();
        #[expect(clippy::cast_sign_loss)]
        #[expect(clippy::cast_possible_truncation)]
        let max_cursor_count = (update_frequency / 2.0).round() as usize;
        audio.set_max_cursor_count(max_cursor_count);
    }

    /// F11 switches between windowed and fullscreen on the monitor the window is on.
    fn process_display_hotkey(input_state: &InputState, display: &mut Display<'_>) {
        if !input_state.keyboard().key(Key::F11).was_pressed() {
            return;
        }
        let settings = match display.current() {
            DisplaySettings::Fullscreen { .. } => DisplaySettings::Windowed,
            DisplaySettings::Windowed => {
                let monitors = display.monitors();
                let Some(monitor) = monitors.iter().find(|monitor| monitor.current()) else {
                    return;
                };
                let current_refresh_rate = monitor.modes().find(|mode| mode.current());
                let Some(mode) = monitor
                    .modes()
                    .filter(|mode| {
                        current_refresh_rate.is_none_or(|m| mode.refresh_rate() == m.refresh_rate())
                    })
                    .max_by_key(|mode| Self::create_mode_key(mode))
                else {
                    return;
                };
                DisplaySettings::Fullscreen {
                    monitor_identifier: monitor.identifier().to_owned(),
                    mode: mode.clone(),
                }
            }
        };
        display.request(settings);
    }

    #[inline]
    #[must_use]
    fn create_mode_key(mode: &MonitorMode) -> (u32, u32, u32) {
        let resolution = mode.resolution();
        (
            resolution.width_px(),
            resolution.height_px(),
            mode.refresh_rate().get::<hertz>(),
        )
    }

    fn calculate_delta_x_y(
        input_state: &InputState,
        game_state: &GameState,
        plugin_state: &PluginGameState,
        controller: Option<&ControllerState>,
    ) -> (Length, Length) {
        let Some((delta_x, delta_y, controller)) =
            Self::calculate_input_delta_x_y(input_state, controller)
        else {
            return (Length::zero(), Length::zero());
        };
        let frame_duration = game_state.frame_duration();
        let is_running = plugin_state.player().running()
            || (controller.keyboard() && input_state.keyboard().is_shift_down());
        let max_speed = if is_running {
            Length::new::<meter>(10.0) / Time::new::<second>(1.0)
        } else {
            Length::new::<meter>(3.0) / Time::new::<second>(1.0)
        };
        let max_distance = frame_duration * max_speed;
        let delta_x = delta_x * max_distance;
        let delta_y = delta_y * max_distance;
        (delta_x, delta_y)
    }

    fn calculate_input_delta_x_y<'a>(
        input_state: &'a InputState,
        controller: Option<&'a ControllerState>,
    ) -> Option<(f32, f32, &'a ControllerState)> {
        let keyboard = input_state.keyboard().as_controller();
        let delta_x = Self::calculate_keyboard_delta_x(keyboard);
        let delta_y = Self::calculate_keyboard_delta_y(keyboard);
        if delta_x != 0.0 || delta_y != 0.0 {
            return Some((delta_x, delta_y, keyboard));
        }
        if let Some(controller) = controller {
            let delta_x = Self::calculate_controller_delta_x(controller);
            let delta_y = Self::calculate_controller_delta_y(controller);
            return Some((delta_x, delta_y, controller));
        }
        None
    }

    #[inline]
    #[must_use]
    fn calculate_keyboard_delta_x(keyboard: &ControllerState) -> f32 {
        Self::calculate_input_delta(*keyboard.right(), *keyboard.left())
    }

    #[inline]
    #[must_use]
    fn calculate_keyboard_delta_y(keyboard: &ControllerState) -> f32 {
        Self::calculate_input_delta(*keyboard.up(), *keyboard.down())
    }

    #[must_use]
    fn calculate_input_delta(positive: ButtonState, negative: ButtonState) -> f32 {
        if positive.ended_down() {
            if negative.ended_down() { 0.0 } else { 1.0 }
        } else if negative.ended_down() {
            -1.0
        } else {
            0.0
        }
    }

    #[inline]
    #[must_use]
    fn calculate_controller_delta_x(controller: &ControllerState) -> f32 {
        if controller.left().ended_down() {
            -1.0
        } else if controller.right().ended_down() {
            1.0
        } else {
            controller.left_joystick().x_ratio()
        }
    }

    #[inline]
    #[must_use]
    fn calculate_controller_delta_y(controller: &ControllerState) -> f32 {
        if controller.up().ended_down() {
            1.0
        } else if controller.down().ended_down() {
            -1.0
        } else {
            controller.left_joystick().y_ratio()
        }
    }

    fn update_render_direct(
        game_state: &GameState,
        plugin_state: &mut PluginGameState,
        input_state: &InputState,
        buffer: &mut BackBuffer,
    ) {
        let controller = Self::detect_primary_controller(plugin_state, input_state);

        if let Some(controller) = controller {
            if controller.left_thumb().was_released() {
                plugin_state.player_mut().toggle_running();
            }

            let zoom = Self::calculate_zoom(game_state, plugin_state.zoom(), controller);
            plugin_state.set_zoom(zoom);
        }

        let viewport = Viewport::fit(
            plugin_state.world(),
            buffer.width_px(),
            buffer.height_px(),
            plugin_state.zoom(),
        );

        Self::ensure_chunks(plugin_state, &viewport);

        if let Some(new_coordinates) =
            Self::find_new_player_coordinates(game_state, plugin_state, input_state, controller)
            && Self::is_traversable(plugin_state, &new_coordinates)
        {
            let player_coordinate = plugin_state.player().coordinate();
            let new_coordinates = if let Some(stairs) =
                Self::find_stairs(plugin_state.world(), &new_coordinates)
                && !player_coordinate.is_same_tile(&new_coordinates, false)
            {
                let player_z = player_coordinate.chunk_key().z();
                if player_z == 0 && stairs == TileType::UpStairs {
                    new_coordinates.elevated(1)
                } else if player_z == 1 && stairs == TileType::DownStairs {
                    new_coordinates.elevated(-1)
                } else {
                    new_coordinates
                }
            } else {
                new_coordinates
            };
            plugin_state.player_mut().set_coordinates(new_coordinates);
        }

        let _ = Self::render_chunk(plugin_state, &viewport, buffer);

        let _ = Self::render_player(plugin_state, &viewport, buffer);

        #[cfg(feature = "audio_debug")]
        let _ = Self::render_audio(game_state, plugin_state, &viewport, buffer);

        #[cfg(feature = "mouse_debug")]
        let _ = Self::render_mouse(input_state, &viewport, buffer);
    }

    /// Scales the zoom by the right joystick's vertical tilt.
    ///
    /// Zooming multiplies rather than adds, so each second the stick is held changes the zoom
    /// by the same ratio at every level, and tilting the other way exactly undoes it.
    #[must_use]
    fn calculate_zoom(
        game_state: &GameState,
        previous_zoom: f32,
        controller: &ControllerState,
    ) -> f32 {
        const MIN_ZOOM: f32 = 0.25;
        const MAX_ZOOM: f32 = 2.0;
        // A fully tilted stick doubles or halves the zoom this many times per second.
        const DOUBLINGS_PER_SECOND: f32 = 1.0;

        let tilt = controller.right_joystick().y_ratio();
        let seconds = game_state.frame_duration().get::<second>();
        let zoom = previous_zoom * (DOUBLINGS_PER_SECOND * tilt * seconds).exp2();
        zoom.clamp(MIN_ZOOM, MAX_ZOOM)
    }

    fn detect_primary_controller<'a>(
        plugin_state: &mut PluginGameState,
        input_state: &'a InputState,
    ) -> Option<&'a ControllerState> {
        if let Some(primary_index) = plugin_state.primary_controller_index()
            && let Some(primary_controller) = input_state.controller_at(primary_index)
        {
            if primary_controller.enabled() {
                // The primary controller is already detected and enabled
                return Some(primary_controller);
            }
            plugin_state.clear_primary_controller_index();
            return None;
        }
        // NOTE: In the future, when we support multiple players, we won't want to allow players
        // to "steal" another player's character when they get disconnected. Once a player gets
        // assigned a slot, they are stuck to until their controller is disconnected.
        for (index, controller) in input_state.controllers().enumerate() {
            if controller.enabled() && Self::is_input_detected(controller) {
                plugin_state.set_primary_controller_index(index);
                return Some(controller);
            }
        }
        None
    }

    /// For local development purposes, we detect inputs by checking for any sort of input.
    fn is_input_detected(controller: &ControllerState) -> bool {
        let button_pressed = Button::ALL
            .iter()
            .map(|b| controller.button(*b))
            .any(|b| b.half_transition_count() > 0);
        if button_pressed {
            return true;
        }
        let left_joystick = controller.left_joystick();
        if left_joystick.x_ratio() != 0.0 || left_joystick.y_ratio() != 0.0 {
            return true;
        }
        let right_joystick = controller.right_joystick();
        if right_joystick.x_ratio() != 0.0 || right_joystick.y_ratio() != 0.0 {
            return true;
        }
        controller.left_trigger_ratio() != 0.0 || controller.right_trigger_ratio() != 0.0
    }

    fn ensure_chunks(plugin_state: &mut PluginGameState, viewport: &Viewport) {
        let center = Self::view_center(plugin_state);
        let Dimensions { width, height } = viewport.bounds();
        // We give ourselves an extra tile of margin on either side
        let tile_size = plugin_state.world().tile_size();
        let margin_width = (width / 2.0) + tile_size;
        let margin_height = (height / 2.0) + tile_size;
        let bottom_left = center
            .shifted(-margin_width, -margin_height, tile_size)
            .chunk_key();
        let top_right = center
            .shifted(margin_width, margin_height, tile_size)
            .chunk_key();
        let world = plugin_state.world_mut();
        world.ensure_chunks(bottom_left, top_right, Self::initialize_chunk);

        // For now, let's just also create tiles on both floors
        let player_z = plugin_state.player().coordinate().chunk_key().z();
        let z_delta = Self::find_delta_z(player_z);
        let bottom_left = bottom_left.shifted(0, 0, z_delta);
        let top_right = top_right.shifted(0, 0, z_delta);
        let world = plugin_state.world_mut();
        world.ensure_chunks(bottom_left, top_right, Self::initialize_chunk);
    }

    fn initialize_chunk(world: &mut World, key: ChunkKey) {
        let chunk = world.get_chunk(key).unwrap();
        let rows = chunk.rows();
        let columns = chunk.columns();
        debug_assert!(rows > 2);
        debug_assert!(columns > 2);

        let mut generator = rand::rng();

        let has_stairs = Self::has_stairs(world, key, || generator.random());
        if has_stairs
            && let Some(value) = world
                .get_chunk_mut(key)
                .unwrap()
                .get_mut(Self::STAIRS_X, Self::STAIRS_Y)
        {
            *value = if key.z() == 0 {
                TileType::UpStairs
            } else {
                TileType::DownStairs
            };
        }

        let [top_door, bottom_door, left_door, right_door] =
            Self::pick_doors(world, key, &mut generator);

        // Top and bottom
        let top = 0;
        let bottom = rows - 1;
        let middle_column = columns / 2;
        let column_door_indexes = (middle_column - 1)..=(middle_column + 1);
        for column in 0..columns {
            if let Some(value) = world.get_chunk_mut(key).unwrap().get_mut(top, column) {
                if column_door_indexes.contains(&column) && top_door {
                    *value = TileType::Door;
                } else {
                    *value = TileType::Wall;
                }
            }
            if let Some(value) = world.get_chunk_mut(key).unwrap().get_mut(bottom, column) {
                if column_door_indexes.contains(&column) && bottom_door {
                    *value = TileType::Door;
                } else {
                    *value = TileType::Wall;
                }
            }
        }

        // Left and right
        let left = 0;
        let right = columns - 1;
        let middle_row = rows / 2;
        let row_door_indexes = (middle_row - 1)..=(middle_row + 1);
        for row in 1..bottom {
            if let Some(value) = world.get_chunk_mut(key).unwrap().get_mut(row, left) {
                if row_door_indexes.contains(&row) && left_door {
                    *value = TileType::Door;
                } else {
                    *value = TileType::Wall;
                }
            }
            if let Some(value) = world.get_chunk_mut(key).unwrap().get_mut(row, right) {
                if row_door_indexes.contains(&row) && right_door {
                    *value = TileType::Door;
                } else {
                    *value = TileType::Wall;
                }
            }
        }
    }

    fn pick_doors(world: &World, key: ChunkKey, generator: &mut ThreadRng) -> [bool; 4] {
        const TOP_INDEX: usize = 0;
        const BOTTOM_INDEX: usize = 1;
        const LEFT_INDEX: usize = 2;
        const RIGHT_INDEX: usize = 3;
        const DOOR_CHANCE: [f64; 4] = [0.5, 0.65, 0.8, 1.0];
        let mut indexes = [0, 1, 2, 3];
        indexes.shuffle(generator);
        let mut doors = [false; 4];
        let mut wall_count = 0;
        for index in indexes {
            let pick = || generator.random_bool(DOOR_CHANCE[wall_count]);
            let is_door = match index {
                TOP_INDEX => Self::top_door(world, key, pick),
                BOTTOM_INDEX => Self::bottom_door(world, key, pick),
                LEFT_INDEX => Self::left_door(world, key, pick),
                RIGHT_INDEX => Self::right_door(world, key, pick),
                _ => unreachable!(),
            };
            if is_door {
                doors[index] = true;
            } else {
                wall_count += 1;
            }
        }
        doors
    }

    fn has_stairs(world: &World, key: ChunkKey, generator: impl FnOnce() -> bool) -> bool {
        let z_delta = Self::find_delta_z(key.z());
        let neighbor = world.get_chunk(key.shifted(0, 0, z_delta));
        let neighbor_has_stairs = neighbor
            .and_then(|n| n.get(Self::STAIRS_X, Self::STAIRS_Y))
            .map(TileType::is_stairs);
        neighbor_has_stairs.unwrap_or_else(generator)
    }

    fn find_delta_z(current: i16) -> i16 {
        // For now, we will only support two floors. If we're on the top floor, we see if
        // the bottom floor already has stairs and mirror that. Reversed for the bottom floor.
        // If this chunk is generated first, we randomly decide if there are stairs or not.
        if current == 0 {
            1
        } else if current == 1 {
            -1
        } else {
            unreachable!("There should only ever be two floors");
        }
    }

    fn top_door(world: &World, key: ChunkKey, generator: impl FnOnce() -> bool) -> bool {
        let bottom = world.rows() - 1;
        let middle_column = world.columns() / 2;
        let top_neighbor = world.get_chunk(key.shifted(0, -1, 0));
        let top_neighbor_has_door = top_neighbor
            .and_then(|n| n.get(bottom, middle_column))
            .map(|t| t == TileType::Door);
        top_neighbor_has_door.unwrap_or_else(generator)
    }

    fn bottom_door(world: &World, key: ChunkKey, generator: impl FnOnce() -> bool) -> bool {
        let top = 0;
        let middle_column = world.columns() / 2;
        let bottom_neighbor = world.get_chunk(key.shifted(0, 1, 0));
        let bottom_neighbor_has_door = bottom_neighbor
            .and_then(|n| n.get(top, middle_column))
            .map(|t| t == TileType::Door);
        bottom_neighbor_has_door.unwrap_or_else(generator)
    }

    fn left_door(world: &World, key: ChunkKey, generator: impl FnOnce() -> bool) -> bool {
        let right = world.columns() - 1;
        let middle_row = world.rows() / 2;
        let left_neighbor = world.get_chunk(key.shifted(-1, 0, 0));
        let left_neighbor_has_door = left_neighbor
            .and_then(|n| n.get(middle_row, right))
            .map(|t| t == TileType::Door);
        left_neighbor_has_door.unwrap_or_else(generator)
    }

    fn right_door(world: &World, key: ChunkKey, generator: impl FnOnce() -> bool) -> bool {
        let left = 0;
        let middle_row = world.rows() / 2;
        let right_neighbor = world.get_chunk(key.shifted(1, 0, 0));
        let right_neighbor_has_door = right_neighbor
            .and_then(|n| n.get(middle_row, left))
            .map(|t| t == TileType::Door);
        right_neighbor_has_door.unwrap_or_else(generator)
    }

    /// The world position drawn at the center of the screen.
    fn view_center(plugin_state: &PluginGameState) -> &WorldCoordinate {
        // NOTE: In the future, we won't want to center on the player. Instead, we'll want
        // to center on the camera position.
        plugin_state.player().coordinate()
    }

    fn find_new_player_coordinates(
        game_state: &GameState,
        plugin_state: &PluginGameState,
        input_state: &InputState,
        controller: Option<&ControllerState>,
    ) -> Option<WorldCoordinate> {
        let (delta_x, delta_y) =
            Self::calculate_delta_x_y(input_state, game_state, plugin_state, controller);
        if delta_x.is_zero() && delta_y.is_zero() {
            return None;
        }

        let world = plugin_state.world();
        let old_coordinates = plugin_state.player().coordinate();
        let new_coordinates = old_coordinates.shifted(delta_x, delta_y, world.tile_size());
        Some(new_coordinates)
    }

    fn is_traversable(plugin_state: &PluginGameState, coordinates: &WorldCoordinate) -> bool {
        let world = plugin_state.world();
        let collision_deltas = Player::collision_bound_deltas(world.tile_size());
        world.is_traversable(coordinates, collision_deltas)
    }

    fn find_stairs(world: &World, coordinates: &WorldCoordinate) -> Option<TileType> {
        world
            .get_chunk(coordinates.chunk_key())
            .and_then(|c| c.get(coordinates.tile_y(), coordinates.tile_x()))
            .filter(|t| t.is_stairs())
    }

    fn render_chunk(
        plugin_state: &PluginGameState,
        viewport: &Viewport,
        buffer: &mut BackBuffer,
    ) -> Result<()> {
        let pixels = buffer.pixels_mut();

        let world = plugin_state.world();
        let tile_size = world.tile_size();
        let half_tile_size = tile_size / 2.0;
        let player_coordinate = plugin_state.player().coordinate();
        let center = Self::view_center(plugin_state);
        let (center_x_offset, center_y_offset) = center.tile_offset().to_tuple();
        let Dimensions { width, height } = viewport.bounds();
        let tiles_x = Self::tiles_to_edge(width / 2.0, tile_size);
        let tiles_y = Self::tiles_to_edge(height / 2.0, tile_size);
        for delta_y in -tiles_y..=tiles_y {
            let delta_y = f32::from(delta_y) * tile_size;
            for delta_x in -tiles_x..=tiles_x {
                let delta_x = f32::from(delta_x) * tile_size;
                let tile = center.shifted(delta_x, delta_y, tile_size);
                let left = delta_x - center_x_offset - half_tile_size;
                let bottom = delta_y - center_y_offset - half_tile_size;
                let tile_rectangle = Rectangle::new(bottom, left, tile_size, tile_size);
                let tile_rectangle = viewport.to_buffer_rectangle(&tile_rectangle);
                let color = Self::determine_tile_color(
                    world,
                    player_coordinate,
                    tile.chunk_key(),
                    tile.tile_x(),
                    tile.tile_y(),
                );
                Self::render_rectangle(viewport, &tile_rectangle, Color::from(color), pixels)?;
            }
        }
        Ok(())
    }

    /// Counts the tiles, beyond the center, needed to reach a screen edge `distance`
    /// from the center.
    #[expect(clippy::cast_possible_truncation)]
    fn tiles_to_edge(distance: Length, tile_size: Length) -> i16 {
        (distance / tile_size).get::<ratio>().ceil() as i16
    }

    fn determine_tile_color(
        world: &World,
        player_coordinate: &WorldCoordinate,
        chunk_key: ChunkKey,
        tile_x: u8,
        tile_y: u8,
    ) -> Color<f32> {
        let tile = world.get_chunk(chunk_key).map(|c| c[(tile_y, tile_x)]);
        if let Some(tile) = tile {
            if chunk_key == player_coordinate.chunk_key()
                && tile_y == player_coordinate.tile_y()
                && tile_x == player_coordinate.tile_x()
            {
                return Color::from(Color::from_rgb(0x00, 0x00, 0x00));
            } else if tile == TileType::Floor {
                return Color::from(Color::from_rgb(0xCC, 0xCC, 0xCC));
            } else if tile == TileType::Door {
                return Color::from(Color::from_rgb(0xBB, 0xBB, 0xBB));
            } else if tile == TileType::Wall {
                return Color::from(Color::from_rgb(0xFF, 0xFF, 0xFF));
            } else if tile.is_stairs() {
                return Color::from(Color::from_rgb(0xAA, 0xAA, 0xAA));
            }
        }
        // Fallback color
        Color::from(Color::from_rgb(0xCC, 0x00, 0xCC))
    }

    fn render_player(
        plugin_state: &PluginGameState,
        viewport: &Viewport,
        buffer: &mut BackBuffer,
    ) -> Result<()> {
        let pixels = buffer.pixels_mut();
        let world = plugin_state.world();
        let player = plugin_state.player();
        let player_bounds = Player::render_bounds(world.tile_size());
        let player_bounds = viewport.to_buffer_rectangle(&player_bounds);
        Self::render_rectangle(
            viewport,
            &player_bounds,
            Color::from(player.color()),
            pixels,
        )
    }

    fn render_rectangle(
        viewport: &Viewport,
        rectangle: &Rectangle<f32>,
        color: Color<u8>,
        pixels: &mut [Color<u8>],
    ) -> Result<()> {
        let window_bounds = viewport.bounds_px();
        let rectangle = rectangle.bound_to(&window_bounds);
        let rectangle = rectangle.round_to_usize()?;
        if rectangle.width() == 0 {
            // Avoid iterating over rows when there are no columns.
            return Ok(());
        }

        #[expect(clippy::cast_sign_loss)]
        #[expect(clippy::cast_possible_truncation)]
        let pitch = window_bounds.width() as usize;
        let mut index = rectangle.bottom() * pitch + rectangle.left();
        for _y in rectangle.bottom()..rectangle.top() {
            let row = index;
            for _x in rectangle.left()..rectangle.right() {
                pixels[index] = color;
                index += 1;
            }
            index = row + pitch;
        }
        Ok(())
    }

    #[cfg(feature = "audio_debug")]
    fn render_audio(
        game_state: &GameState,
        plugin_state: &PluginGameState,
        viewport: &Viewport,
        buffer: &mut BackBuffer,
    ) -> Result<()> {
        use uom::si::information::byte;

        let Some(buffer_length) = game_state.audio().buffer_length() else {
            return Ok(());
        };
        let window_bounds = viewport.bounds_px();
        let padding_x = 50.0;
        let padding_y = 50.0;
        let height = 100.0;
        let line_bottom = window_bounds.top() - padding_y - height;
        let play_color = Color::from_rgb(0xFF, 0x00, 0x00);
        let write_color = Color::from_rgb(0x00, 0xFF, 0x00);
        let boundary_color = Color::from_rgb(0xFF, 0xFF, 0x00);
        let pixels = buffer.pixels_mut();

        let window_width = window_bounds.width();
        let buffer_length_bytes = buffer_length.get::<byte>();
        #[expect(clippy::cast_precision_loss)]
        let chunk = (window_width - 2.0 * padding_x) / buffer_length_bytes as f32;
        let audio = plugin_state.audio();
        for (index, (play_cursor, write_cursor)) in audio
            .flip_play_cursors()
            .zip(audio.flip_write_cursors())
            .enumerate()
        {
            let bottom = if index + 1 == audio.flips_len() {
                line_bottom - height * 3.0
            } else {
                line_bottom
            };
            #[expect(clippy::cast_precision_loss)]
            let x = padding_x + chunk * play_cursor as f32;
            let line = Rectangle::new(bottom, x, height, 1.0);
            let line = viewport.flip_to_buffer(&line);
            Self::render_rectangle(viewport, &line, play_color, pixels)?;

            #[expect(clippy::cast_precision_loss)]
            let x = padding_x + chunk * write_cursor as f32;
            let line = Rectangle::new(bottom, x, height, 1.0);
            let line = viewport.flip_to_buffer(&line);
            Self::render_rectangle(viewport, &line, write_color, pixels)?;
        }

        if let Some(play_cursor) = game_state.audio().output_play_cursor() {
            let bottom = line_bottom - height;
            #[expect(clippy::cast_precision_loss)]
            let x = padding_x + chunk * play_cursor as f32;
            let line = Rectangle::new(bottom, x, height, 1.0);
            let line = viewport.flip_to_buffer(&line);
            Self::render_rectangle(viewport, &line, play_color, pixels)?;
        }

        if let Some(write_cursor) = game_state.audio().output_write_cursor() {
            let bottom = line_bottom - height;
            #[expect(clippy::cast_precision_loss)]
            let x = padding_x + chunk * write_cursor as f32;
            let line = Rectangle::new(bottom, x, height, 1.0);
            let line = viewport.flip_to_buffer(&line);
            Self::render_rectangle(viewport, &line, write_color, pixels)?;
        }

        if let Some(expected_frame_boundary) = game_state.audio().expected_frame_boundary() {
            let bottom = line_bottom - height;
            #[expect(clippy::cast_precision_loss)]
            let x = padding_x + chunk * expected_frame_boundary as f32;
            let line = Rectangle::new(bottom, x, height, 1.0);
            let line = viewport.flip_to_buffer(&line);
            Self::render_rectangle(viewport, &line, boundary_color, pixels)?;
        }

        if let Some(write_offset) = game_state.audio().output_write_offset() {
            let bottom = line_bottom - 2.0 * height;
            #[expect(clippy::cast_precision_loss)]
            let x = padding_x + chunk * write_offset as f32;
            let line = Rectangle::new(bottom, x, height, 1.0);
            let line = viewport.flip_to_buffer(&line);
            Self::render_rectangle(viewport, &line, play_color, pixels)?;

            if let Some(write_length) = game_state.audio().output_write_length() {
                #[expect(clippy::cast_precision_loss)]
                let write_end = write_offset as f32 + write_length.get::<byte>() as f32;
                let x = padding_x + chunk * write_end;
                let line = Rectangle::new(bottom, x, height, 1.0);
                let line = viewport.flip_to_buffer(&line);
                Self::render_rectangle(viewport, &line, write_color, pixels)?;
            }
        }

        Ok(())
    }

    #[cfg(feature = "mouse_debug")]
    fn render_mouse(
        input_state: &InputState,
        viewport: &Viewport,
        buffer: &mut BackBuffer,
    ) -> Result<()> {
        let mouse = input_state.mouse();
        let pixels = buffer.pixels_mut();

        // The position does not need translated into world-relative coordinates
        #[expect(clippy::cast_precision_loss)]
        let bottom = mouse.y() as f32;
        #[expect(clippy::cast_precision_loss)]
        let left = mouse.x() as f32;
        let length = 10.0;
        let position = Rectangle::new(bottom - length / 2.0, left - length / 2.0, length, length);
        let color = Color::from_rgb(0xFF, 0x00, 0x00);
        Self::render_rectangle(viewport, &position, color, pixels)?;

        for (index, button) in mouse.buttons().enumerate() {
            if button.ended_down() {
                let bottom = 100.0;
                #[expect(clippy::cast_precision_loss)]
                let offset = index as f32 * 25.0 + 100.0;
                let indicator = Rectangle::new(bottom, offset, length, length);
                let indicator = viewport.flip_to_buffer(&indicator);
                Self::render_rectangle(viewport, &indicator, color, pixels)?;
            }
        }

        Ok(())
    }

    fn write_sound_direct(
        game_state: &GameState,
        plugin_state: &mut PluginGameState,
        sound_buffer: &mut [StereoSample],
    ) {
        const C_HERTZ: f32 = 261.63;
        const TWO_PI: f32 = 2.0 * f32::consts::PI;

        let audio_state = game_state.audio();
        let frequency = audio_state.frequency();
        let volume = audio_state.volume();

        #[expect(clippy::cast_sign_loss)]
        #[expect(clippy::cast_possible_truncation)]
        let tone = Frequency::new::<hertz>(C_HERTZ as u32);
        let period = if tone == Frequency::zero() {
            0
        } else {
            (frequency / tone).get::<ratio>()
        };

        let plugin_audio_state = plugin_state.audio_mut();
        let mut theta = plugin_audio_state.theta();
        for outbound in sound_buffer {
            #[expect(clippy::cast_precision_loss)]
            let step = if period == 0 {
                0.0
            } else {
                1.0 / period as f32
            };
            theta += TWO_PI * step;
            if theta >= TWO_PI {
                theta -= TWO_PI;
            }
            let value = f32::from(volume) * theta.sin();
            #[expect(clippy::cast_possible_truncation)]
            let value = value as i16;
            let sample = StereoSample::from_left_right(value, value);
            *outbound = sample;
        }
        plugin_audio_state.set_theta(theta);
    }

    #[cfg(feature = "audio_debug")]
    fn capture_audio_flip(game_state: &GameState, plugin_state: &mut PluginGameState) {
        let audio = game_state.audio();
        if let Some(play_cursor) = audio.flip_play_cursor()
            && let Some(write_cursor) = audio.flip_write_cursor()
        {
            plugin_state
                .audio_mut()
                .add_flip_data(play_cursor, write_cursor);
        }
    }
}

impl Application for ApplicationPlugin {
    #[inline]
    fn name(&self) -> String {
        String::from("Handmade Hero")
    }

    fn suggest_frame_duration(&self, _monitors: &[Monitor]) -> Option<Time> {
        None
    }

    #[inline]
    fn create_plugin_state(&self) -> Box<dyn PluginState> {
        Box::new(PluginGameState::new())
    }

    fn deserialize_plugin_state(
        &self,
        deserializer: &mut dyn erased_serde::Deserializer<'_>,
    ) -> Result<Box<dyn PluginState>> {
        // This is the only place that knows the bytes describe a `PluginGameState`, which is
        // why deserialization has to happen on this side of the plugin boundary.
        let plugin_state: PluginGameState = erased_serde::deserialize(deserializer)
            .map_err(|e| ApplicationError::wrap("Could not deserialize the game state", e))?;
        Ok(Box::new(plugin_state))
    }

    fn initialize(&self, context: InitializeContext<'_>) {
        let InitializeContext {
            game_state,
            plugin_state,
            ..
        } = context;
        if let Some(plugin_state) = plugin_state.downcast_mut::<PluginGameState>() {
            Self::initialize_direct(game_state, plugin_state);
        }
    }

    fn update_render(&self, context: UpdateRenderContext<'_>) {
        let UpdateRenderContext {
            game_state,
            plugin_state,
            input_state,
            buffer,
            mut display,
            ..
        } = context;
        Self::process_display_hotkey(input_state, &mut display);
        if let Some(plugin_state) = plugin_state.downcast_mut::<PluginGameState>() {
            Self::update_render_direct(game_state, plugin_state, input_state, buffer);
        }
    }

    fn write_sound(&self, context: AudioContext<'_>) {
        let AudioContext {
            game_state,
            plugin_state,
            mut sound_buffer,
            ..
        } = context;
        let Some(plugin_state) = plugin_state.downcast_mut::<PluginGameState>() else {
            return;
        };
        let Some(samples) = sound_buffer.as_samples_mut::<StereoSample>() else {
            return;
        };
        Self::write_sound_direct(game_state, plugin_state, samples);

        #[cfg(feature = "audio_debug")]
        Self::capture_audio_flip(game_state, plugin_state);
    }
}
