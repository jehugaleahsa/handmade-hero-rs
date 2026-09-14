use handmade_hero_interface::application::Application;
use handmade_hero_interface::application_error::{ApplicationError, Result};
use handmade_hero_interface::game_state::GameState;
use handmade_hero_interface::game_state_seed::GameStateSeed;

/// The whole game state as bytes, so it can outlive the plugin that produced it.
///
/// A hot reload swaps the plugin library out from under the game. The plugin-owned parts of the
/// state cannot survive that directly: their vtables point into the old library. Their data can
/// survive, however. The old plugin serializes the state before it goes away, and the new plugin
/// deserializes it once it is loaded. The assumption is the state's shape changing cannot be
/// hot-swapped. The same machinery the recorder uses for playback carries the game across the
/// reload.
#[derive(Debug)]
pub struct GameStateSnapshot {
    bytes: Vec<u8>,
}

impl GameStateSnapshot {
    /// Serializes the state. The plugin that created its plugin-owned parts must still be loaded.
    pub fn capture(state: &GameState) -> Result<Self> {
        let bytes = bincode::serde::encode_to_vec(state, bincode::config::standard())
            .map_err(|e| ApplicationError::wrap("Could not serialize the game state", e))?;
        let snapshot = Self { bytes };
        Ok(snapshot)
    }

    /// Rebuilds the state with the given plugin.
    pub fn restore(&self, application: &dyn Application) -> Result<GameState> {
        let seed = GameStateSeed::new(application);
        let (state, _) =
            bincode::serde::seed_decode_from_slice(seed, &self.bytes, bincode::config::standard())
                .map_err(|e| ApplicationError::wrap("Could not deserialize the game state", e))?;
        Ok(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use handmade_hero_interface::audio_context::AudioContext;
    use handmade_hero_interface::initialize_context::InitializeContext;
    use handmade_hero_interface::input_context::InputContext;
    use handmade_hero_interface::plugin_state::PluginState;
    use handmade_hero_interface::render_context::RenderContext;
    use serde::{Deserialize, Serialize};
    use uom::si::f32::Time;
    use uom::si::time::second;

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

    fn populated_state() -> GameState {
        let mut state = GameState::new();
        state.set_frame_duration(Time::new::<second>(1.0 / 30.0));
        state.set_plugin_state(Box::new(FakeGameState {
            player_x: 42,
            visited: vec!["hub".into(), "north".into()],
            theta: 1.25,
        }));
        state
    }

    #[test]
    fn round_trips_shared_and_plugin_state() {
        let original = populated_state();

        let snapshot = GameStateSnapshot::capture(&original).expect("capture");
        let restored = snapshot.restore(&FakeApplication).expect("restore");

        assert_eq!(restored.frame_duration(), original.frame_duration());
        assert_eq!(
            restored.audio().channel_count(),
            original.audio().channel_count()
        );
        assert_eq!(
            restored.plugin_state::<FakeGameState>(),
            original.plugin_state::<FakeGameState>()
        );
    }

    #[test]
    fn round_trips_without_plugin_state() {
        let original = GameState::new();

        let snapshot = GameStateSnapshot::capture(&original).expect("capture");
        let restored = snapshot.restore(&FakeApplication).expect("restore");

        assert!(restored.plugin_state::<FakeGameState>().is_none());
    }

    #[test]
    fn restore_fails_when_plugin_layout_changed() {
        let snapshot = GameStateSnapshot::capture(&populated_state()).expect("capture");

        let restored = snapshot.restore(&IncompatibleApplication);

        assert!(restored.is_err());
    }
}
