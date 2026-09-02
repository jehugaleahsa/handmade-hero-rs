use crate::player::Player;
use crate::sample::Sample;
use crate::sound_state::SoundState;
use crate::stereo_sample::StereoSample;
use crate::tile_map_key::TileMapKey;
use crate::units::si::length::Length;
use crate::units::si::time::Time;
use crate::world::World;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uom::num::Zero;
use uom::si::length::meter;

#[derive(Debug, Serialize, Deserialize)]
pub struct GameState {
    sound: SoundState,
    player: Player,
    frame_duration: Time,
    world: World,
}

impl GameState {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        let sample = StereoSample::default();
        let sound = SoundState::new(sample.channel_count(), sample.channel_size());
        let tile_size = Length::new::<meter>(1.4f32);
        let x_offset = -(tile_size / 1.6f32);
        let y_offset = -(tile_size / 3.5f32);
        let world = World {
            rows: World::TILE_ROWS,
            columns: World::TILE_COLUMNS,
            x_offset,
            y_offset,
            tile_maps: HashMap::new(),
            tile_size,
        };
        let current_tile_map_key = TileMapKey { x: 0, y: 0 };
        let player = Player::new(&world, current_tile_map_key);
        Self {
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
    pub fn frame_duration(&self) -> Time {
        self.frame_duration
    }

    #[inline]
    pub fn set_frame_duration(&mut self, value: Time) {
        self.frame_duration = value;
    }

    #[inline]
    #[must_use]
    pub fn player(&self) -> &Player {
        &self.player
    }

    #[inline]
    #[must_use]
    pub fn player_mut(&mut self) -> &mut Player {
        &mut self.player
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
