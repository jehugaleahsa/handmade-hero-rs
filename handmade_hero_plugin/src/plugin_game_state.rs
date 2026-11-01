use std::collections::HashSet;

use crate::chunk_key::ChunkKey;
use crate::entity::Entity;
use crate::plugin_audio_state::PluginAudioState;
use crate::world::World;
use crate::world_coordinate::WorldCoordinate;
use handmade_hero_interface::units::si::length::Length;
use serde::{Deserialize, Serialize};
use uom::si::length::meter;

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginGameState {
    entities: Vec<Entity>,
    world: World,
    camera_position: WorldCoordinate,
    audio: PluginAudioState,
    primary_controller_index: Option<usize>,
    player_disabled: HashSet<usize>,
    controller_player_indexes: Vec<Option<usize>>,
    zoom: f32,
}

impl PluginGameState {
    #[must_use]
    pub fn new() -> Self {
        let tile_size = Length::new::<meter>(1.4);
        let world = World::new(World::TILE_ROWS, World::TILE_COLUMNS, tile_size);
        let current_chunk_key = ChunkKey::at_x_y_z(0, 0, 0);
        let camera_position = WorldCoordinate::new(&world, current_chunk_key).centered();
        Self {
            entities: Vec::new(),
            world,
            camera_position,
            audio: PluginAudioState::new(),
            primary_controller_index: None,
            controller_player_indexes: Vec::new(),
            player_disabled: HashSet::new(),
            zoom: 1.0,
        }
    }

    #[inline]
    #[must_use]
    pub fn entity(&self, index: usize) -> Option<&Entity> {
        self.entities.get(index)
    }

    #[inline]
    #[must_use]
    pub fn entity_mut(&mut self, index: usize) -> Option<&mut Entity> {
        self.entities.get_mut(index)
    }

    #[inline]
    #[must_use]
    pub fn camera_position(&self) -> &WorldCoordinate {
        &self.camera_position
    }

    #[inline]
    #[must_use]
    pub fn camera_position_mut(&mut self) -> &mut WorldCoordinate {
        &mut self.camera_position
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
    pub fn players(&self) -> impl Iterator<Item = &Entity> {
        self.controller_player_indexes
            .iter()
            .filter_map(|o| o.and_then(|i| self.entity(i)))
    }

    #[inline]
    #[must_use]
    pub fn player_index_by_controller_index(&self, controller_index: usize) -> Option<usize> {
        self.controller_player_indexes
            .get(controller_index)
            .and_then(|i| *i)
    }

    #[inline]
    #[must_use]
    pub fn player_by_controller_index(&self, controller_index: usize) -> Option<&Entity> {
        if let Some(index) = self.player_index_by_controller_index(controller_index) {
            self.entity(index)
        } else {
            None
        }
    }

    #[inline]
    #[must_use]
    pub fn player_by_controller_index_mut(
        &mut self,
        controller_index: usize,
    ) -> Option<&mut Entity> {
        if let Some(index) = self.player_index_by_controller_index(controller_index) {
            self.entity_mut(index)
        } else {
            None
        }
    }

    pub fn map_controller_to_player_index(&mut self, controller_index: usize, entity_index: usize) {
        if controller_index >= self.controller_player_indexes.len() {
            self.controller_player_indexes
                .resize(controller_index + 1, None);
        }
        self.controller_player_indexes[controller_index] = Some(entity_index);
    }

    #[inline]
    pub fn add_player(&mut self, player: Entity) -> usize {
        let index = self.entities.len();
        self.entities.push(player);
        index
    }

    #[inline]
    #[must_use]
    pub fn is_player_disabled(&self, player_index: usize) -> bool {
        self.player_disabled.contains(&player_index)
    }

    #[inline]
    pub fn enable_player(&mut self, player_index: usize) {
        self.player_disabled.remove(&player_index);
    }

    #[inline]
    pub fn disable_player(&mut self, player_index: usize) {
        self.player_disabled.insert(player_index);
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
