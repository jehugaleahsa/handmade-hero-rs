mod application_loader;
mod performance_counter;
mod playback_recorder;

#[cfg(target_os = "windows")]
mod direct_sound;
#[cfg(target_os = "windows")]
mod direct_sound_buffer;
#[cfg(target_os = "windows")]
mod direct_sound_buffer_lock_guard;
#[cfg(target_os = "windows")]
mod win32_application;

use handmade_hero_interface::application_error::Result;

#[cfg(target_os = "windows")]
use win32_application::Win32Application;

fn main() -> Result<()> {
    #[cfg(target_os = "windows")]
    run_windows()
}

#[cfg(target_os = "windows")]
fn run_windows() -> Result<()> {
    let mut windows_application = Win32Application::new();
    windows_application.create_window(960, 540)?;
    windows_application.run()
}
