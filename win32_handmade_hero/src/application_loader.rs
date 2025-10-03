use handmade_hero_interface::application_state::ApplicationState;
use handmade_hero_interface::pixel::Pixel;
use handmade_hero_interface::stereo_sample::StereoSample;
use handmade_hero_interface::Application;
use libloading::{library_filename, Library, Symbol};
use std::marker::PhantomData;

pub struct ApplicationStub<'loader> {
    application: Box<dyn Application>,
    loader: PhantomData<&'loader ApplicationLoader>,
}

impl Application for ApplicationStub<'_> {
    #[inline]
    fn render(
        &self,
        state: &mut ApplicationState,
        bitmap_buffer: &mut [Pixel],
        width: u16,
        height: u16,
    ) {
        self.application.render(state, bitmap_buffer, width, height);
    }

    #[inline]
    fn write_sound(&self, state: &mut ApplicationState, sound_buffer: &mut [StereoSample]) {
        self.application.write_sound(state, sound_buffer);
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
