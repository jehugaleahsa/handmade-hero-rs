mod application_plugin;

use crate::application_plugin::ApplicationPlugin;
use handmade_hero_interface::Application;

#[unsafe(no_mangle)]
#[must_use]
pub extern "Rust" fn create_application() -> Box<dyn Application> {
    Box::new(ApplicationPlugin)
}
