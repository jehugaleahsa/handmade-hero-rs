mod application;
mod application_error;
mod performance_counter;
mod pixel;

#[cfg(target_os = "windows")]
mod direct_sound;
#[cfg(target_os = "windows")]
mod direct_sound_buffer;
#[cfg(target_os = "windows")]
mod direct_sound_buffer_lock_guard;
#[cfg(target_os = "windows")]
mod win32_application;

use crate::application::Application;
use application_error::Result;

#[cfg(target_os = "windows")]
use win32_application::Win32Application;

fn main() -> Result<()> {
    let mut application = Application::new();
    application.resize_bitmap(720, 480);
    #[cfg(target_os = "windows")]
    run_windows(application)
}

#[cfg(target_os = "windows")]
fn run_windows(application: Application) -> Result<()> {
    let mut windows_application = Win32Application::new(application);
    windows_application.create_window(720, 480)?;
    windows_application.run()
}
