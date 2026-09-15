mod application_loader;
mod playback_recorder;
mod plugin_state_snapshot;

#[cfg(target_os = "windows")]
mod win32;

#[cfg(target_os = "windows")]
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;

use handmade_hero_interface::application_error::{ApplicationError, Result};

use crate::application_loader::ApplicationLoader;

fn main() -> ExitCode {
    let Ok(exe_directory) = exe_directory() else {
        return ExitCode::FAILURE;
    };

    let mut loader = ApplicationLoader::new(&exe_directory);

    #[cfg(target_os = "windows")]
    run_windows(&exe_directory, &mut loader).unwrap_or(ExitCode::FAILURE)
}

#[cfg(target_os = "windows")]
fn run_windows(
    exe_directory: &Path,
    application_loader: &mut ApplicationLoader,
) -> Result<ExitCode> {
    use win32::win32_application::Win32Application;

    let mut windows_application = Win32Application::new(exe_directory);
    windows_application.run(application_loader, 960, 540)
}

/// The directory the executable lives in. The plugin is loaded from here and recordings are
/// written here, whichever platform is running.
fn exe_directory() -> Result<PathBuf> {
    let current_exe_path = std::env::current_exe()
        .map_err(|e| ApplicationError::wrap("Failed to retrieve the current executable path", e))?;
    let current_directory = current_exe_path.parent().ok_or_else(|| {
        ApplicationError::new("Failed to retrieve the current executable parent directory")
    })?;
    Ok(current_directory.to_path_buf())
}
