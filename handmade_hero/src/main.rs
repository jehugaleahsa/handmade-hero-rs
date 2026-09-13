mod application_loader;
mod playback_recorder;

#[cfg(target_os = "windows")]
mod win32;

use std::process::ExitCode;

use handmade_hero_interface::application_error::Result;

fn main() -> ExitCode {
    #[cfg(target_os = "windows")]
    run_windows().unwrap_or(ExitCode::FAILURE)
}

#[cfg(target_os = "windows")]
fn run_windows() -> Result<ExitCode> {
    use win32::win32_application::Win32Application;

    let mut windows_application = Win32Application::new();
    windows_application.create_window(960, 540)?;
    windows_application.run()
}
