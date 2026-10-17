use crate::dimensions::Dimensions;
use crate::player::Player;
use crate::tile_map_coordinate::TileMapCoordinate;
use crate::tile_map_key::TileMapKey;
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
use std::cmp::Ordering;
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
        let new_coordinates = WorldCoordinate::new(plugin_state.world(), player.tile_map_key());
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
        plugin_state: &mut PluginGameState,
    ) -> (Length, Length) {
        let controller = plugin_state
            .primary_controller_index()
            .and_then(|i| input_state.controller_at(i));
        if controller.is_some_and(|c| c.left_thumb().was_released()) {
            plugin_state.player_mut().toggle_running();
        }
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
        Self::detect_primary_controller(plugin_state, input_state);
        let viewport = Viewport::fit(plugin_state.world(), buffer.width_px(), buffer.height_px());
        Self::ensure_tile_maps(plugin_state, &viewport);

        if let Some(new_coordinates) =
            Self::find_new_player_coordinates(game_state, plugin_state, input_state)
            && Self::is_traversable(plugin_state, &new_coordinates)
        {
            plugin_state.player_mut().set_coordinates(new_coordinates);
        }

        let player_coordinate = plugin_state.player().coordinate();
        let start_coordinate =
            Self::determine_start_coordinate(plugin_state.world(), player_coordinate);

        let _ = Self::render_tilemap(plugin_state, &viewport, &start_coordinate, buffer);

        let _ = Self::render_player(plugin_state, &viewport, &start_coordinate, buffer);

        #[cfg(feature = "audio_debug")]
        let _ = Self::render_audio(game_state, plugin_state, &viewport, buffer);

        #[cfg(feature = "mouse_debug")]
        let _ = Self::render_mouse(input_state, &viewport, buffer);
    }

    fn detect_primary_controller(plugin_state: &mut PluginGameState, input_state: &InputState) {
        if let Some(primary_index) = plugin_state.primary_controller_index()
            && let Some(primary_controller) = input_state.controller_at(primary_index)
        {
            if primary_controller.enabled() {
                // The primary controller is already detected and enabled
                return;
            }
            plugin_state.clear_primary_controller_index();
        }
        // NOTE: In the future, when we support multiple players, we won't want to allow players
        // to "steal" another player's character when they get disconnected. Once a player gets
        // assigned a slot, they are stuck to until their controller is disconnected.
        for (index, controller) in input_state.controllers().enumerate() {
            if controller.enabled() && Self::is_input_detected(controller) {
                plugin_state.set_primary_controller_index(index);
            }
        }
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
        if left_joystick.x_ratio() > 0.0 || left_joystick.y_ratio() > 0.0 {
            return true;
        }
        let right_joystick = controller.right_joystick();
        if right_joystick.x_ratio() > 0.0 || right_joystick.y_ratio() > 0.0 {
            return true;
        }
        controller.left_trigger_ratio() > 0.0 || controller.right_trigger_ratio() > 0.0
    }

    fn ensure_tile_maps(plugin_state: &mut PluginGameState, viewport: &Viewport) {
        // NOTE: In the future, we won't want to center on the player. Instead, we'll want
        // to center on the camera position.
        let center = plugin_state.player().coordinate();
        let Dimensions { width, height } = viewport.bounds();
        // We give ourselves an extra tile of margin on either side
        let tile_size = plugin_state.world().tile_size();
        let margin_width = (width / 2.0) + tile_size;
        let margin_height = (height / 2.0) + tile_size;
        let bottom_left = center
            .shifted(-margin_width, -margin_height, tile_size)
            .tile_map_key();
        let top_right = center
            .shifted(margin_width, margin_height, tile_size)
            .tile_map_key();
        let world = plugin_state.world_mut();
        world.ensure_tile_maps(bottom_left, top_right);
    }

    fn find_new_player_coordinates(
        game_state: &GameState,
        plugin_state: &mut PluginGameState,
        input_state: &InputState,
    ) -> Option<WorldCoordinate> {
        let (delta_x, delta_y) = Self::calculate_delta_x_y(input_state, game_state, plugin_state);
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

    fn render_tilemap(
        plugin_state: &PluginGameState,
        viewport: &Viewport,
        start_coordinate: &WorldCoordinate,
        buffer: &mut BackBuffer,
    ) -> Result<()> {
        let pixels = buffer.pixels_mut();

        let world = plugin_state.world();
        let rows = world.rows();
        let columns = world.columns();
        let tile_size = world.tile_size();
        let player_coordinate = plugin_state.player().coordinate().clone();
        let (player_x_offset, player_y_offset) = player_coordinate.tile_offset().to_tuple();

        // The player's offset within their tile can expose part of a tile past either edge of
        // the screen, so the grid starts one tile early and runs one tile late.
        let start_coordinate = start_coordinate.shifted(-tile_size, -tile_size, tile_size);
        let mut tile_map_y = start_coordinate.tile_map_y();
        let mut tile_y = start_coordinate.tile_y();
        for row_index in 0..=rows {
            let row_index = f32::from(row_index);
            // Subtracting a tile undoes the early start, keeping the original start coordinate's
            // tile flush with the screen's edge.
            let row_offset = row_index * tile_size - tile_size;
            let mut tile_map_x = start_coordinate.tile_map_x();
            let mut tile_x = start_coordinate.tile_x();
            for column_index in 0..=columns {
                let column_index = f32::from(column_index);
                let column_offset = column_index * tile_size - tile_size;
                let left = column_offset - player_x_offset;
                let bottom = row_offset - player_y_offset;
                let tile_rectangle = Rectangle::new(bottom, left, tile_size, tile_size);

                let tile_map_key = TileMapKey::at_x_y(tile_map_x, tile_map_y);
                let color = Self::determine_tile_color(
                    plugin_state.world(),
                    &player_coordinate,
                    tile_map_key,
                    tile_x,
                    tile_y,
                );
                let tile_rectangle = viewport.to_buffer_rectangle(&tile_rectangle);
                Self::render_rectangle(viewport, &tile_rectangle, Color::from(color), pixels)?;

                tile_x += 1;
                if tile_x >= columns {
                    tile_map_x += 1;
                    tile_x = 0;
                }
            }

            tile_y += 1;
            if tile_y >= rows {
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
        tile_x: u8,
        tile_y: u8,
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
        plugin_state: &PluginGameState,
        viewport: &Viewport,
        start_coordinate: &WorldCoordinate,
        buffer: &mut BackBuffer,
    ) -> Result<()> {
        let pixels = buffer.pixels_mut();

        let world = plugin_state.world();
        let player = plugin_state.player();
        let player_coordinate = player.coordinate();
        let x_offset = Self::determine_player_offset(
            start_coordinate.tile_map_x(),
            start_coordinate.tile_x(),
            player_coordinate.tile_map_x(),
            player_coordinate.tile_x(),
            world.tile_size(),
            world.columns(),
        );
        let y_offset = Self::determine_player_offset(
            start_coordinate.tile_map_y(),
            start_coordinate.tile_y(),
            player_coordinate.tile_map_y(),
            player_coordinate.tile_y(),
            world.tile_size(),
            world.rows(),
        );

        // The offsets reach the low edge of the player's tile, but the player stands at the
        // tile's center.
        let half_tile_size = world.tile_size() / 2.0;
        let player_bounds = Player::render_bounds(world.tile_size());
        let player_bounds =
            player_bounds.shifted(x_offset + half_tile_size, y_offset + half_tile_size);
        let player_bounds = viewport.to_buffer_rectangle(&player_bounds);
        Self::render_rectangle(
            viewport,
            &player_bounds,
            Color::from(player.color()),
            pixels,
        )
    }

    fn determine_player_offset(
        start_tile_map: i16,
        start_tile: u8,
        tile_map: i16,
        tile: u8,
        tile_size: Length,
        max_tile: u8,
    ) -> Length {
        let tile_map_diff = f32::from(tile_map) - f32::from(start_tile_map);
        let tile_map_diff = tile_map_diff * f32::from(max_tile) * tile_size;
        let tile_diff = f32::from(tile) - f32::from(start_tile);
        let tile_diff = tile_diff * tile_size;
        tile_map_diff + tile_diff
    }

    fn determine_start_coordinate(
        world: &World,
        player_coordinate: &WorldCoordinate,
    ) -> WorldCoordinate {
        let tile_map_x = player_coordinate.tile_map_x();
        let tile_x = player_coordinate.tile_x();
        let (start_tile_map_x, start_tile_x) =
            Self::determine_start(tile_map_x, tile_x, world.columns());

        let tile_map_y = player_coordinate.tile_map_y();
        let tile_y = player_coordinate.tile_y();
        let (start_tile_map_y, start_tile_y) =
            Self::determine_start(tile_map_y, tile_y, world.rows());

        let start_tile_map_key = TileMapKey::at_x_y(start_tile_map_x, start_tile_map_y);
        let tile_map_coordinate = TileMapCoordinate::at_x_y(start_tile_x, start_tile_y);
        WorldCoordinate::new(world, start_tile_map_key).within_tile_map(tile_map_coordinate)
    }

    fn determine_start(tile_map: i16, tile: u8, max_tile: u8) -> (i16, u8) {
        let middle = max_tile / 2;
        match tile.cmp(&middle) {
            Ordering::Less => (tile_map - 1, max_tile - (middle - tile)),
            Ordering::Greater => (tile_map, tile - middle),
            Ordering::Equal => (tile_map, 0),
        }
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
