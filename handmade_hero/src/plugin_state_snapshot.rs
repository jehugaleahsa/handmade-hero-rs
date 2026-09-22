use handmade_hero_interface::application::Application;
use handmade_hero_interface::application_error::{ApplicationError, Result};
use handmade_hero_interface::plugin_state::PluginState;
use handmade_hero_interface::plugin_state_seed::PluginStateSeed;

/// The plugin's state as bytes, so it can outlive the plugin that produced it.
///
/// A hot reload swaps the plugin library out from under the game. The plugin state cannot
/// survive that directly: its vtable points into the old library. Its data can survive, however.
/// The old plugin serializes the state before it goes away, and the new plugin deserializes it
/// once it is loaded. The assumption is the state's shape changing cannot be hot-swapped.
///
/// Only the plugin state needs this treatment. The platform-owned `GameState` has no vtables,
/// so it simply stays in place across the reload.
#[derive(Debug)]
pub struct PluginStateSnapshot {
    bytes: Vec<u8>,
}

impl PluginStateSnapshot {
    /// Serializes the state. The plugin that created it must still be loaded.
    pub fn capture(state: &dyn PluginState) -> Result<Self> {
        let bytes = bincode::serde::encode_to_vec(state, bincode::config::standard())
            .map_err(|e| ApplicationError::wrap("Could not serialize the plugin state", e))?;
        let snapshot = Self { bytes };
        Ok(snapshot)
    }

    /// Rebuilds the state with the given plugin.
    pub fn restore(&self, application: &dyn Application) -> Result<Box<dyn PluginState>> {
        let seed = PluginStateSeed::new(application);
        let (state, _) =
            bincode::serde::seed_decode_from_slice(seed, &self.bytes, bincode::config::standard())
                .map_err(|e| ApplicationError::wrap("Could not deserialize the plugin state", e))?;
        Ok(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use handmade_hero_interface::audio_context::AudioContext;
    use handmade_hero_interface::initialize_context::InitializeContext;
    use handmade_hero_interface::input_context::InputContext;
    use handmade_hero_interface::render_context::RenderContext;
    use serde::{Deserialize, Serialize};

    /// Stands in for the plugin's game state. Being `Serialize + Debug + 'static` is all it takes
    /// to be a `PluginState`.
    #[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
    struct FakeGameState {
        player_x: u32,
        visited: Vec<String>,
        theta: f32,
    }

    /// The smallest `Application` that can round-trip its own state.
    struct FakeApplication;

    impl Application for FakeApplication {
        fn name(&self) -> String {
            String::from("Fake")
        }

        fn create_plugin_state(&self) -> Box<dyn PluginState> {
            Box::new(FakeGameState::default())
        }

        fn deserialize_plugin_state(
            &self,
            deserializer: &mut dyn erased_serde::Deserializer<'_>,
        ) -> Result<Box<dyn PluginState>> {
            let state: FakeGameState = erased_serde::deserialize(deserializer)
                .map_err(|e| ApplicationError::wrap("bad game state", e))?;
            Ok(Box::new(state))
        }

        fn initialize(&self, _: InitializeContext<'_>) {}
        fn process_input(&self, _: InputContext<'_>) {}
        fn render(&self, _: RenderContext<'_>) {}
        fn write_sound(&self, _: AudioContext<'_>) {}
    }

    /// A plugin whose game state has a different shape, as after a hot reload changed it.
    ///
    /// bincode carries no field names or types, so a mismatch is only *guaranteed* to fail when
    /// the bytes cannot be read as the new type at all. A change that keeps the bytes readable,
    /// such as swapping two same-sized integers, decodes silently into the wrong values. A plugin
    /// that needs to catch those must version its own payload.
    struct IncompatibleApplication;

    #[derive(Debug, Serialize, Deserialize)]
    struct IncompatibleGameState {
        // Where the snapshot holds a `u32`, this expects a length-prefixed string. The integer is
        // taken as the length and there are not enough bytes left to satisfy it.
        name: String,
        visited: Vec<String>,
    }

    impl Application for IncompatibleApplication {
        fn name(&self) -> String {
            String::from("Incompatible")
        }

        fn create_plugin_state(&self) -> Box<dyn PluginState> {
            unimplemented!()
        }

        fn deserialize_plugin_state(
            &self,
            deserializer: &mut dyn erased_serde::Deserializer<'_>,
        ) -> Result<Box<dyn PluginState>> {
            let state: IncompatibleGameState = erased_serde::deserialize(deserializer)
                .map_err(|e| ApplicationError::wrap("bad game state", e))?;
            Ok(Box::new(state))
        }

        fn initialize(&self, _: InitializeContext<'_>) {}
        fn process_input(&self, _: InputContext<'_>) {}
        fn render(&self, _: RenderContext<'_>) {}
        fn write_sound(&self, _: AudioContext<'_>) {}
    }

    fn populated_state() -> FakeGameState {
        FakeGameState {
            player_x: 42,
            visited: vec!["hub".into(), "north".into()],
            theta: 1.25,
        }
    }

    #[test]
    fn round_trips_plugin_state() {
        let original = populated_state();

        let snapshot = PluginStateSnapshot::capture(&original).expect("capture");
        let restored = snapshot.restore(&FakeApplication).expect("restore");

        assert_eq!(restored.downcast_ref::<FakeGameState>(), Some(&original));
    }

    #[test]
    fn restore_fails_when_plugin_layout_changed() {
        let snapshot = PluginStateSnapshot::capture(&populated_state()).expect("capture");

        let restored = snapshot.restore(&IncompatibleApplication);

        assert!(restored.is_err());
    }
}
