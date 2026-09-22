use crate::application_error::Result;
use crate::audio_context::AudioContext;
use crate::initialize_context::InitializeContext;
use crate::input_context::InputContext;
use crate::plugin_state::PluginState;
use crate::render_context::RenderContext;

pub trait Application {
    /// The game's display name, used for the window title.
    ///
    /// The platform asks for this before any state exists, so it can't depend on the plugin
    /// state or the back buffer.
    fn name(&self) -> String;

    /// Creates game-specific state.
    fn create_plugin_state(&self) -> Box<dyn PluginState>;

    /// Rebuilds game-specific state from a stream the platform layer serialized earlier.
    ///
    /// # Errors
    /// If the plugin state changes in an incompatible way between hot-reloads an error will be returned.
    fn deserialize_plugin_state(
        &self,
        deserializer: &mut dyn erased_serde::Deserializer<'_>,
    ) -> Result<Box<dyn PluginState>>;

    fn initialize(&self, context: InitializeContext<'_>);

    fn process_input(&self, context: InputContext<'_>);

    fn render(&self, context: RenderContext<'_>);

    fn write_sound(&self, context: AudioContext<'_>);
}
