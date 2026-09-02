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
#[cfg(target_os = "windows")]
mod win32_controller;
#[cfg(target_os = "windows")]
mod win32_keyboard;
#[cfg(target_os = "windows")]
mod win32_mouse;
#[cfg(target_os = "windows")]
mod win32_utils;
#[cfg(target_os = "windows")]
mod win32_window;

use std::process::ExitCode;

use handmade_hero_interface::application_error::Result;

#[cfg(target_os = "windows")]
use win32_application::Win32Application;

fn main() -> ExitCode {
    #[cfg(target_os = "windows")]
    run_windows().unwrap_or(ExitCode::FAILURE)
}

#[cfg(target_os = "windows")]
fn run_windows() -> Result<ExitCode> {
    let mut windows_application = Win32Application::new();
    windows_application.create_window(960, 540)?;
    windows_application.run()
}
