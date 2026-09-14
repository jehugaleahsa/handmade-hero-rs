use std::fmt::{self, Debug, Formatter};

use serde::Deserialize;
use serde::de::{DeserializeSeed, Deserializer, Error, MapAccess, SeqAccess, Visitor};

use crate::application::Application;
use crate::game_state::GameState;
use crate::plugin_state_seed::PluginStateSeed;
use crate::units::si::time::Time;

/// The field names, in declaration order, exactly as the derived `Serialize` writes them.
const FIELDS: &[&str] = &["frame_duration", "audio", "plugin"];

/// Deserializes a [`GameState`], routing the `plugin` field through the plugin.
///
/// This is the code `#[derive(Deserialize)]` would have generated, written out by hand so the
/// last field can be deserialized with a [`PluginStateSeed`] instead of `Deserialize`. The
/// other fields still use their derived `Deserialize`.
#[derive(Clone, Copy)]
pub struct GameStateSeed<'a> {
    application: &'a dyn Application,
}

impl<'a> GameStateSeed<'a> {
    #[inline]
    #[must_use]
    pub fn new(application: &'a dyn Application) -> Self {
        Self { application }
    }

    #[inline]
    fn plugin_seed(&self) -> PluginStateSeed<'a> {
        PluginStateSeed::new(self.application)
    }
}

impl<'de> DeserializeSeed<'de> for GameStateSeed<'_> {
    type Value = GameState;

    #[inline]
    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_struct("GameState", FIELDS, self)
    }
}

/// Field names as an enum so a self-describing format can deliver them in any order.
#[derive(Debug, Deserialize)]
#[serde(field_identifier, rename_all = "snake_case")]
enum Field {
    FrameDuration,
    Audio,
    Plugin,
}

impl<'de> Visitor<'de> for GameStateSeed<'_> {
    type Value = GameState;

    fn expecting(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str("struct GameState")
    }

    /// Compact formats like bincode carry no field names. They deliver a struct as a sequence of
    /// values in declaration order, so this is the path the recorder actually takes.
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let frame_duration = seq
            .next_element()?
            .ok_or_else(|| Error::invalid_length(0, &self))?;
        let audio = seq
            .next_element()?
            .ok_or_else(|| Error::invalid_length(1, &self))?;
        // `next_element_seed` is the seeded twin of `next_element`: same slot in the sequence,
        // but the seed decides how to read it.
        let plugin = seq
            .next_element_seed(self.plugin_seed())?
            .ok_or_else(|| Error::invalid_length(2, &self))?;
        let state = GameState {
            frame_duration,
            audio,
            plugin,
        };
        Ok(state)
    }

    /// Self-describing formats like JSON deliver a struct as key/value pairs in any order. bincode
    /// never calls this, but a correct `Visitor` handles both so the state stays format-agnostic.
    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut frame_duration: Option<Time> = None;
        let mut audio = None;
        let mut plugin = None;
        while let Some(key) = map.next_key()? {
            match key {
                Field::FrameDuration => {
                    if frame_duration.is_some() {
                        return Err(Error::duplicate_field("frame_duration"));
                    }
                    frame_duration = Some(map.next_value()?);
                }
                Field::Audio => {
                    if audio.is_some() {
                        return Err(Error::duplicate_field("audio"));
                    }
                    audio = Some(map.next_value()?);
                }
                Field::Plugin => {
                    if plugin.is_some() {
                        return Err(Error::duplicate_field("plugin"));
                    }
                    plugin = Some(map.next_value_seed(self.plugin_seed())?);
                }
            }
        }
        let frame_duration =
            frame_duration.ok_or_else(|| Error::missing_field("frame_duration"))?;
        let audio = audio.ok_or_else(|| Error::missing_field("audio"))?;
        let plugin = plugin.ok_or_else(|| Error::missing_field("plugin"))?;
        let state = GameState {
            frame_duration,
            audio,
            plugin,
        };
        Ok(state)
    }
}

impl Debug for GameStateSeed<'_> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        // `dyn Application` has no `Debug`, so there is nothing to print but the name.
        formatter
            .debug_struct("GameStateSeed")
            .finish_non_exhaustive()
    }
}
