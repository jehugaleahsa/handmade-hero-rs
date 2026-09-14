use std::any::Any;
use std::fmt::Debug;

/// Game-specific state the platform layer carries without understanding.
///
/// The platform layer needs exactly three things from this state: to drop it, to serialize it,
/// and to hand it back to the plugin. Dropping and serializing go through the vtable, so the
/// plugin's code does the work. Handing it back goes through `Any`: the plugin upcasts to
/// `dyn Any` and downcasts to its concrete type.
///
/// Every vtable for a `dyn PluginState` lives inside the plugin library. Dropping, serializing,
/// or downcasting one after the library is unloaded jumps into unmapped memory, so the platform
/// layer must drop these before it unloads the plugin.
pub trait PluginState: erased_serde::Serialize + Any + Debug {}

/// Any serializable, debuggable, `'static` type is plugin state. The plugin derives `Serialize`
/// and gets this for free.
impl<T: serde::Serialize + Any + Debug> PluginState for T {}

// `erased_serde::Serialize` is object safe but is not `serde::Serialize`. This macro bridges the
// gap by implementing `serde::Serialize` for `dyn PluginState`, which lets `Box<dyn PluginState>`
// take part in a derived `Serialize` like any other field.
erased_serde::serialize_trait_object!(PluginState);
