use crate::rectangle::Rectangle;
use crate::sound_state::SoundState;
use crate::units::si::length::{Length, pixel};
use crate::units::si::time::Time;
use crate::world::{TileMapKey, World};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uom::num::Zero;
use uom::si::length::meter;

#[derive(Debug, Serialize, Deserialize)]
pub struct GameState {
    width: Length,
    height: Length,
    sound: SoundState,
    player: Rectangle<f32>,
    frame_duration: Time,
    world: World,
}

impl GameState {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        let sound = SoundState::new();
        let tile_size = Length::new::<meter>(1.4f32);
        let x_offset = -(tile_size / 1.6f32);
        let y_offset = -(tile_size / 3.5f32);
        let world = World {
            rows: World::TILE_ROWS,
            columns: World::TILE_COLUMNS,
            x_offset,
            y_offset,
            current_tile_map_id: TileMapKey::Hub,
            tile_maps: HashMap::new(),
            tile_size,
        };
        let player = Rectangle::new(
            0f32,
            0f32,
            (tile_size * 0.9f32).get::<pixel>(),
            (tile_size * 0.75f32).get::<pixel>(),
        );
        Self {
            width: Length::zero(),
            height: Length::zero(),
            sound,
            player,
            frame_duration: Time::zero(),
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
    pub fn width(&self) -> Length {
        self.width
    }

    #[inline]
    pub fn set_width(&mut self, value: Length) {
        self.width = value;
    }

    #[inline]
    #[must_use]
    pub fn height(&self) -> Length {
        self.height
    }

    #[inline]
    pub fn set_height(&mut self, value: Length) {
        self.height = value;
    }

    #[inline]
    #[must_use]
    pub fn frame_duration(&self) -> Time {
        self.frame_duration
    }

    #[inline]
    pub fn set_frame_duration(&mut self, value: Time) {
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
