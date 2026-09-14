use std::fmt::{self, Debug, Formatter};

use serde::de::{DeserializeSeed, Deserializer, Error};

use crate::application::Application;
use crate::plugin_state::PluginState;

/// Deserializes a `Box<dyn PluginState>` by handing the work to the plugin.
///
/// Plain `Deserialize` cannot do this: the platform layer does not know the concrete type, and
/// `Deserialize` has no way to pass in the plugin that does. A `DeserializeSeed` is a
/// `Deserialize` that carries a value, and here the value is the plugin. Nothing about the
/// payload's structure is interpreted on this side; the seed erases the deserializer and hands
/// it straight across the plugin boundary.
#[derive(Clone, Copy)]
pub struct PluginStateSeed<'a> {
    application: &'a dyn Application,
}

impl<'a> PluginStateSeed<'a> {
    #[inline]
    #[must_use]
    pub fn new(application: &'a dyn Application) -> Self {
        Self { application }
    }
}

impl<'de> DeserializeSeed<'de> for PluginStateSeed<'_> {
    type Value = Box<dyn PluginState>;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        let mut erased = <dyn erased_serde::Deserializer<'de>>::erase(deserializer);
        self.application
            .deserialize_plugin_state(&mut erased)
            .map_err(D::Error::custom)
    }
}

impl Debug for PluginStateSeed<'_> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        // `dyn Application` has no `Debug`, so there is nothing to print but the name.
        formatter
            .debug_struct("PluginStateSeed")
            .finish_non_exhaustive()
    }
}
