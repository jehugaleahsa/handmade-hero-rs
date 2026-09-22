use crate::plugin_state_snapshot::PluginStateSnapshot;
use handmade_hero_interface::application::Application;
use handmade_hero_interface::application_error::{ApplicationError, Result};
use handmade_hero_interface::audio_context::AudioContext;
use handmade_hero_interface::initialize_context::InitializeContext;
use handmade_hero_interface::input_context::InputContext;
use handmade_hero_interface::plugin_state::PluginState;
use handmade_hero_interface::render_context::RenderContext;
use libloading::{Library, Symbol, library_filename};
use std::ffi::OsString;
use std::fmt::{self, Debug, Formatter};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::SystemTime;

pub struct ApplicationStub {
    application: Box<dyn Application>,
    // NOTE: Ensure library appears after application, so these fields get dropped
    // in the correct order!
    library: Library,
}

impl Debug for ApplicationStub {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ApplicationStub")
            .field("library", &self.library)
            .finish_non_exhaustive()
    }
}

impl Application for ApplicationStub {
    #[inline]
    fn name(&self) -> String {
        self.application.name()
    }

    #[inline]
    fn create_plugin_state(&self) -> Box<dyn PluginState> {
        self.application.create_plugin_state()
    }

    #[inline]
    fn deserialize_plugin_state(
        &self,
        deserializer: &mut dyn erased_serde::Deserializer<'_>,
    ) -> Result<Box<dyn PluginState>> {
        self.application.deserialize_plugin_state(deserializer)
    }

    #[inline]
    fn initialize(&self, context: InitializeContext<'_>) {
        self.application.initialize(context);
    }

    #[inline]
    fn process_input(&self, context: InputContext<'_>) {
        self.application.process_input(context);
    }

    #[inline]
    fn render(&self, context: RenderContext<'_>) {
        self.application.render(context);
    }

    #[inline]
    fn write_sound(&self, context: AudioContext<'_>) {
        self.application.write_sound(context);
    }
}

/// Loads the game plugin, hot reloads it when a newer build lands on disk, and carries the plugin
/// state across the reload.
///
/// The loader owns the one invariant that makes unloading safe: every object the plugin created
/// must be dropped before its library is. Their vtables live in the library, so touching one
/// afterward, even just to drop it, is undefined behavior.
///
/// The loaded plugin is handed out as an `Rc` rather than a borrow. That lets the loader live as
/// a field on the same struct as the plugin state without every frame turning into a borrow
/// fight, and a field is what makes the drop order at shutdown enforceable: Rust drops fields
/// in declaration order, so a loader declared after the plugin state outlives the objects whose
/// vtables point into its library.
#[derive(Debug)]
pub struct ApplicationLoader {
    plugin_directory: PathBuf,
    last_counter: usize,
    last_modified: Option<SystemTime>,
    stub: Option<Rc<ApplicationStub>>,
}

#[derive(Debug)]
pub enum LoadedApplication {
    /// The plugin is running with its plugin state intact, either because nothing changed or
    /// because the state was carried across a reload.
    Running(Rc<ApplicationStub>),
    /// The plugin was just loaded and has no plugin state. The caller must start a fresh game.
    Fresh(Rc<ApplicationStub>),
}

impl ApplicationLoader {
    #[inline]
    #[must_use]
    pub fn new(plugin_directory: impl Into<PathBuf>) -> Self {
        Self {
            plugin_directory: plugin_directory.into(),
            last_counter: 0,
            last_modified: None,
            stub: None,
        }
    }

    /// Returns the plugin for this frame, hot reloading it when a newer build is on disk.
    ///
    /// A reload carries the plugin state across in three steps, in an order that matters. While
    /// the old plugin is still loaded, the state is taken out of `plugin_state`, serialized, and
    /// dropped. Only then is the library unloaded. Once the new library is up, it rebuilds the
    /// state from the bytes and the result is written back through `plugin_state`. If the new
    /// build changed the state's layout so the bytes no longer fit, `plugin_state` is left empty,
    /// the result is [`LoadedApplication::Fresh`], and the caller starts over as if freshly
    /// launched.
    pub fn load(
        &mut self,
        plugin_state: &mut Option<Box<dyn PluginState>>,
    ) -> Result<LoadedApplication> {
        let snapshot = if self.is_outdated()? {
            // `take` moves the state out of the slot and into the closure, so it drops at the
            // end of the closure while its drop glue is still mapped into memory.
            let snapshot = plugin_state
                .take()
                .and_then(|state| PluginStateSnapshot::capture(state.as_ref()).ok());
            self.unload();
            snapshot
        } else {
            None
        };

        if let Some(ref stub) = self.stub {
            return Ok(LoadedApplication::Running(Rc::clone(stub)));
        }

        let application = self.load_library()?;
        *plugin_state = snapshot.and_then(|s| s.restore(application.as_ref()).ok());
        match plugin_state {
            Some(_) => Ok(LoadedApplication::Running(application)),
            None => Ok(LoadedApplication::Fresh(application)),
        }
    }

    /// Whether the plugin on disk is newer than the one that is loaded.
    fn is_outdated(&self) -> Result<bool> {
        let Some(last_modified) = self.last_modified else {
            // Nothing is loaded, so there is nothing to be out of date.
            return Ok(false);
        };
        let current_modified = self.plugin_last_modified()?;
        let is_outdated = last_modified < current_modified;
        Ok(is_outdated)
    }

    /// Unloads the plugin library. See the type-level docs for what must happen first.
    #[inline]
    fn unload(&mut self) {
        self.stub = None;
    }

    /// Copies the plugin from disk and loads the copy.
    fn load_library(&mut self) -> Result<Rc<ApplicationStub>> {
        let last_modified = self.plugin_last_modified()?;
        let running_name = self.copy_plugin_library()?;
        self.last_modified = Some(last_modified);

        let stub = Self::load_stub(&running_name);
        let stub = Rc::new(stub);
        self.stub = Some(Rc::clone(&stub));
        Ok(stub)
    }

    fn load_stub(running_name: &Path) -> ApplicationStub {
        let library =
            unsafe { Library::new(running_name).expect("Could not load the application library") };
        let creator: Symbol<'_, fn() -> Box<dyn Application>> = unsafe {
            library
                .get(b"create_application")
                .expect("Could not load the application implementation")
        };
        let application = creator();
        ApplicationStub {
            application,
            library,
        }
    }

    /// The last write time of the plugin on disk. The build script renames the previous build
    /// while the new one compiles, so either name counts.
    fn plugin_last_modified(&self) -> Result<SystemTime> {
        let normal_name = self.normal_name();
        let old_name = self
            .plugin_directory
            .join(library_filename("handmade_hero_plugin_old"));
        let metadata = std::fs::metadata(&normal_name)
            .or_else(|_| std::fs::metadata(&old_name))
            .map_err(|e| {
                ApplicationError::wrap("Failed to get the application plugin file metadata", e)
            })?;
        let last_modified = metadata.modified().map_err(|e| {
            ApplicationError::wrap("Failed to get the application plugin file write time", e)
        })?;
        Ok(last_modified)
    }

    /// Copies the plugin to a private name so the compiler can overwrite the original while the
    /// copy is loaded.
    ///
    /// The first copy must succeed. Later copies retry under a fresh name because Windows can
    /// hold the previous copy open briefly after it is unloaded.
    fn copy_plugin_library(&mut self) -> Result<PathBuf> {
        let normal_name = self.normal_name();
        let mut running_name = self.plugin_directory.join(self.current_running_name());
        if self.last_modified.is_none() {
            Self::copy_file(&normal_name, &running_name)?;
        } else {
            while Self::copy_file(&normal_name, &running_name).is_err() {
                self.last_counter += 1;
                running_name = self.plugin_directory.join(self.current_running_name());
            }
        }
        Ok(running_name)
    }

    fn normal_name(&self) -> PathBuf {
        self.plugin_directory
            .join(library_filename("handmade_hero_plugin"))
    }

    fn current_running_name(&self) -> OsString {
        Self::running_name(self.last_counter)
    }

    fn running_name(counter: usize) -> OsString {
        let running_name = format!("handmade_hero_plugin-running{counter}");
        library_filename(running_name)
    }

    fn copy_file(normal_file: &PathBuf, running_file: &PathBuf) -> Result<()> {
        std::fs::copy(normal_file, running_file)
            .map_err(|e| ApplicationError::wrap("Failed to copy the application plugin", e))
            .map(|_| ())
    }
}

impl Drop for ApplicationLoader {
    fn drop(&mut self) {
        self.stub = None;
        for counter in 0..=self.last_counter {
            let running_name = self.plugin_directory.join(Self::running_name(counter));
            std::fs::remove_file(running_name).unwrap_or_default(); // Okay to fail
        }
    }
}
