use std::fmt::{self, Debug, Formatter};

use serde::de::{DeserializeSeed, Deserializer, Error, Visitor};

use crate::application::Application;
use crate::plugin_state::PluginState;

/// Deserializes an `Option<Box<dyn PluginState>>` by handing the work to the plugin.
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
    type Value = Option<Box<dyn PluginState>>;

    #[inline]
    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        // The field is serialized as an `Option`, so ask for one. The format tells the visitor
        // whether it found `None` or `Some`, and hands over a deserializer for the payload.
        deserializer.deserialize_option(self)
    }
}

impl<'de> Visitor<'de> for PluginStateSeed<'_> {
    type Value = Option<Box<dyn PluginState>>;

    fn expecting(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str("optional plugin state")
    }

    #[inline]
    fn visit_none<E: Error>(self) -> Result<Self::Value, E> {
        Ok(None)
    }

    #[inline]
    fn visit_unit<E: Error>(self) -> Result<Self::Value, E> {
        // Some formats represent a missing option as a unit rather than calling `visit_none`.
        Ok(None)
    }

    fn visit_some<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        let mut erased = <dyn erased_serde::Deserializer<'de>>::erase(deserializer);
        let state = self.application.deserialize_plugin_state(&mut erased);
        state.map(Some).map_err(D::Error::custom)
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
