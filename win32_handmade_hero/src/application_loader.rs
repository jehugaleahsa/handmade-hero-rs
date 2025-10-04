use handmade_hero_interface::Application;
use handmade_hero_interface::audio_context::AudioContext;
use handmade_hero_interface::render_context::RenderContext;
use libloading::{Library, Symbol, library_filename};
use std::marker::PhantomData;

pub struct ApplicationStub<'loader> {
    application: Box<dyn Application>,
    loader: PhantomData<&'loader ApplicationLoader>,
}

impl Application for ApplicationStub<'_> {
    #[inline]
    fn render(&self, context: &mut RenderContext<'_>) {
        self.application.render(context);
    }

    #[inline]
    fn write_sound(&self, context: &mut AudioContext<'_>) {
        self.application.write_sound(context);
    }
}

#[derive(Debug)]
pub struct ApplicationLoader {
    library: Library,
}

impl ApplicationLoader {
    pub fn new() -> Self {
        let normal_name = library_filename("handmade_hero_plugin");
        let running_name = library_filename("handmade_hero_plugin-running");
        std::fs::copy(&normal_name, &running_name).unwrap_or(0); // Process holding on after free
        let library =
            unsafe { Library::new(&running_name).expect("Could not load the application library") };
        Self { library }
    }

    pub fn load(&self) -> ApplicationStub<'_> {
        let creator: Symbol<'_, fn() -> Box<dyn Application>> = unsafe {
            self.library
                .get(b"create_application")
                .expect("Could not load the application implementation")
        };
        let application = creator();
        ApplicationStub {
            application,
            loader: PhantomData,
        }
    }
}
