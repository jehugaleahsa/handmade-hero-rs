use std::any::Any;

use serde::Serialize;
use uom::num::Zero;

use crate::{
    audio_state::AudioState, plugin_state::PluginState, sample::Sample,
    stereo_sample::StereoSample, units::si::time::Time,
};

/// Only `Serialize` is derived. Deserializing needs the plugin to build the `plugin` field.
#[derive(Debug, Serialize)]
pub struct GameState {
    pub(crate) frame_duration: Time,
    pub(crate) audio: AudioState,
    pub(crate) plugin: Option<Box<dyn PluginState>>,
}

impl GameState {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        let sample = StereoSample::default();
        let channel_count = sample.channel_count();
        let channel_size = sample.channel_size();
        Self {
            frame_duration: Time::zero(),
            audio: AudioState::new(channel_count, channel_size),
            plugin: None,
        }
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
    pub fn audio(&self) -> &AudioState {
        &self.audio
    }

    #[inline]
    #[must_use]
    pub fn audio_mut(&mut self) -> &mut AudioState {
        &mut self.audio
    }

    /// The plugin's state as its concrete type, or `None` when no plugin state has been set or
    /// it is some other type.
    #[inline]
    #[must_use]
    pub fn plugin_state<P: Any>(&self) -> Option<&P> {
        let state: &dyn Any = self.plugin.as_deref()?;
        state.downcast_ref()
    }

    #[inline]
    #[must_use]
    pub fn plugin_state_mut<P: Any>(&mut self) -> Option<&mut P> {
        let state: &mut dyn Any = self.plugin.as_deref_mut()?;
        state.downcast_mut()
    }

    #[inline]
    pub fn set_plugin_state(&mut self, value: Box<dyn PluginState>) {
        self.plugin = Some(value);
    }

    /// Removes the plugin state so it can be dropped on the caller's schedule, which matters
    /// when the plugin library is about to be unloaded.
    #[inline]
    pub fn take_plugin_state(&mut self) -> Option<Box<dyn PluginState>> {
        self.plugin.take()
    }
}

impl Default for GameState {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
