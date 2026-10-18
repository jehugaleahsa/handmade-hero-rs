use crate::chunk_key::ChunkKey;
use crate::player::Player;
use crate::plugin_audio_state::PluginAudioState;
use crate::world::World;
use handmade_hero_interface::units::si::length::Length;
use serde::{Deserialize, Serialize};
use uom::si::length::meter;

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginGameState {
    player: Player,
    world: World,
    audio: PluginAudioState,
    primary_controller_index: Option<usize>,
    zoom: f32,
}

impl PluginGameState {
    #[must_use]
    pub fn new() -> Self {
        let tile_size = Length::new::<meter>(1.4);
        let world = World::new(World::TILE_ROWS, World::TILE_COLUMNS, tile_size);
        let current_chunk_key = ChunkKey::at_x_y(0, 0);
        let player = Player::new(&world, current_chunk_key);
        Self {
            player,
            world,
            audio: PluginAudioState::new(),
            primary_controller_index: None,
            zoom: 1.0,
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
    #[allow(dead_code)]
    pub fn audio(&self) -> &PluginAudioState {
        &self.audio
    }

    #[inline]
    #[must_use]
    #[allow(dead_code)]
    pub fn audio_mut(&mut self) -> &mut PluginAudioState {
        &mut self.audio
    }

    #[inline]
    #[must_use]
    pub fn primary_controller_index(&self) -> Option<usize> {
        self.primary_controller_index
    }

    #[inline]
    pub fn set_primary_controller_index(&mut self, index: usize) {
        self.primary_controller_index = Some(index);
    }

    #[inline]
    pub fn clear_primary_controller_index(&mut self) {
        self.primary_controller_index = None;
    }

    #[inline]
    #[must_use]
    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    #[inline]
    pub fn set_zoom(&mut self, value: f32) {
        self.zoom = value;
    }
}

impl Default for PluginGameState {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
