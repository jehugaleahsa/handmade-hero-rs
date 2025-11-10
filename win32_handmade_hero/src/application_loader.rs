use handmade_hero_interface::application::Application;
use handmade_hero_interface::application_error::{ApplicationError, Result};
use handmade_hero_interface::audio_context::AudioContext;
use handmade_hero_interface::initialize_context::InitializeContext;
use handmade_hero_interface::input_context::InputContext;
use handmade_hero_interface::render_context::RenderContext;
use libloading::{Library, Symbol, library_filename};
use std::ffi::OsString;
use std::os::windows::fs::MetadataExt;
use std::path::PathBuf;

pub struct ApplicationStub {
    application: Box<dyn Application>,
    // NOTE: Ensure _library appears after application, so these fields get dropped
    // in the correct order!
    _library: Library,
}

impl Application for ApplicationStub {
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

pub struct ApplicationLoader {
    plugin_directory: PathBuf,
    last_counter: usize,
    last_modified: Option<u64>,
    stub: Option<ApplicationStub>,
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

    pub fn load(&mut self, context: InitializeContext<'_>) -> Result<&mut ApplicationStub> {
        let normal_name = self
            .plugin_directory
            .join(library_filename("handmade_hero_plugin"));
        let metadata = std::fs::metadata(&normal_name).map_err(|e| {
            ApplicationError::wrap("Failed to get the application plugin file metadata", e)
        })?;

        let mut running_name = self.plugin_directory.join(self.current_running_name());
        let current_modified = metadata.last_write_time();
        if let Some(last_modified) = self.last_modified {
            if last_modified < current_modified {
                while Self::copy_plugin_library(&normal_name, &running_name).is_err() {
                    self.last_counter += 1;
                    running_name = self.plugin_directory.join(self.current_running_name());
                }
                self.stub = None;
                self.last_modified = Some(current_modified);
            }
        } else {
            Self::copy_plugin_library(&normal_name, &running_name)?;
            self.last_modified = Some(current_modified);
        }

        let application = self.stub.get_or_insert_with(|| {
            let library = unsafe {
                Library::new(&running_name).expect("Could not load the application library")
            };
            let creator: Symbol<'_, fn() -> Box<dyn Application>> = unsafe {
                library
                    .get(b"create_application")
                    .expect("Could not load the application implementation")
            };
            let application = creator();
            application.initialize(context);
            ApplicationStub {
                application,
                _library: library,
            }
        });
        Ok(application)
    }

    fn current_running_name(&self) -> OsString {
        Self::running_name(self.last_counter)
    }

    fn running_name(counter: usize) -> OsString {
        let running_name = format!("handmade_hero_plugin-running{counter}");
        library_filename(running_name)
    }

    fn copy_plugin_library(normal_file: &PathBuf, running_file: &PathBuf) -> Result<()> {
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
