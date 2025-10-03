use libloading::{library_filename, Library, Symbol};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct ApplicationStub<'loader> {
    //pub render: fn(&mut Application, bitmap_buffer: &mut [Pixel]) -> (),
    //pub write_sound: fn(&mut Application, sound_buffer: &mut [StereoSample]) -> (),
    execute_symbol: Symbol<'loader, fn()>,
    loader: PhantomData<&'loader ApplicationLoader>,
}

impl ApplicationStub<'_> {
    #[inline]
    pub fn execute(&self) {
        let symbol = &self.execute_symbol;
        symbol();
    }
}

#[derive(Debug)]
pub struct ApplicationLoader {
    library: Library,
}

impl ApplicationLoader {
    pub fn new() -> Self {
        std::fs::copy("handmade_hero.dll", "handmade_hero-running.dll")
            .expect("Failed to copy the application library");
        let library = unsafe {
            Library::new(library_filename("handmade_hero-running"))
                .expect("Could not load the application library")
        };
        Self { library }
    }

    pub fn load(&self) -> ApplicationStub<'_> {
        let execute = unsafe {
            self.library
                .get(b"execute")
                .expect("Function 'execute' not found")
        };
        ApplicationStub {
            execute_symbol: execute,
            loader: PhantomData,
        }
    }

    #[inline]
    pub fn close(self) {
        self.library
            .close()
            .expect("Failed to unload the application library");
    }
}
