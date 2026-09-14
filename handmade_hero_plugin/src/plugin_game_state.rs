use crate::player::Player;
use crate::plugin_audio_state::PluginAudioState;
use crate::tile_map_key::TileMapKey;
use crate::world::World;
use handmade_hero_interface::units::si::length::Length;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uom::si::length::meter;

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginGameState {
    player: Player,
    world: World,
    audio: PluginAudioState,
}

impl PluginGameState {
    #[must_use]
    pub fn new() -> Self {
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
        let current_tile_map_key = TileMapKey::from_x_y(0, 0);
        let player = Player::new(&world, current_tile_map_key);
        Self {
            player,
            world,
            audio: PluginAudioState::new(),
        }
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

    #[inline]
    #[must_use]
    pub fn audio_mut(&mut self) -> &mut PluginAudioState {
        &mut self.audio
    }
}

impl Default for PluginGameState {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
