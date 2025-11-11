use crate::rectangle::Rectangle;
use crate::sound_state::SoundState;
use crate::world::{TileMapKey, World};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use uom::si::length::{Length, meter};

#[derive(Debug, Serialize, Deserialize)]
pub struct GameState {
    width: u16,
    height: u16,
    sound: SoundState,
    player: Rectangle<f32>,
    frame_duration: Duration,
    world: World,
}

impl GameState {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        let sound = SoundState::new();
        let tile_size_pixels = 60u32;
        #[allow(clippy::cast_precision_loss)]
        let tile_size = tile_size_pixels as f32;
        let x_offset = -(tile_size / 1.6f32);
        let y_offset = -(tile_size / 3.75f32);
        let world = World {
            rows: World::TILE_ROWS,
            columns: World::TILE_COLUMNS,
            x_offset,
            y_offset,
            current_tile_map_id: TileMapKey::Hub,
            tile_maps: HashMap::new(),
            tile_size_meters: Length::new::<meter>(1.4f32),
            tile_size_pixels,
        };
        let player = Rectangle::new(0f32, 0f32, tile_size, tile_size * 0.75f32);
        Self {
            width: 0,
            height: 0,
            sound,
            player,
            frame_duration: Duration::default(),
            world,
        }
    }

    #[inline]
    #[must_use]
    pub fn sound(&self) -> &SoundState {
        &self.sound
    }

    #[inline]
    #[must_use]
    pub fn sound_mut(&mut self) -> &mut SoundState {
        &mut self.sound
    }

    #[inline]
    #[must_use]
    pub fn width(&self) -> u16 {
        self.width
    }

    #[inline]
    pub fn set_width(&mut self, value: u16) {
        self.width = value;
    }

    #[inline]
    #[must_use]
    pub fn height(&self) -> u16 {
        self.height
    }

    #[inline]
    pub fn set_height(&mut self, value: u16) {
        self.height = value;
    }

    #[inline]
    #[must_use]
    pub fn frame_duration(&self) -> Duration {
        self.frame_duration
    }

    #[inline]
    pub fn set_frame_duration(&mut self, value: Duration) {
        self.frame_duration = value;
    }

    #[inline]
    #[must_use]
    pub fn player(&self) -> Rectangle<f32> {
        self.player
    }

    #[inline]
    pub fn set_player(&mut self, player: Rectangle<f32>) {
        self.player = player;
    }

    #[inline]
    #[must_use]
    pub fn world(&self) -> &World {
        &self.world
    }

    #[inline]
    #[must_use]
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }
}

impl Default for GameState {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
