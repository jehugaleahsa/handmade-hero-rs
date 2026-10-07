use crate::application_error::Result;
use crate::display_settings::DisplaySettings;

/// How the game is displayed, kept by the platform from frame to frame.
#[derive(Debug)]
pub struct DisplayState {
    current: DisplaySettings,
    request: Option<DisplaySettings>,
    last_request: Option<Result<()>>,
}

impl DisplayState {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            current: DisplaySettings::Windowed,
            request: None,
            last_request: None,
        }
    }

    #[inline]
    #[must_use]
    pub fn current(&self) -> &DisplaySettings {
        &self.current
    }

    #[inline]
    pub fn set_current(&mut self, settings: DisplaySettings) {
        self.current = settings;
    }

    /// Whether the most recently applied request succeeded. `None` until one has been applied.
    #[inline]
    #[must_use]
    pub fn last_request(&self) -> Option<&Result<()>> {
        self.last_request.as_ref()
    }

    #[inline]
    pub fn set_last_request(&mut self, result: Result<()>) {
        self.last_request = Some(result);
    }

    /// Asks for different settings. A later request in the same frame replaces an earlier one.
    #[inline]
    pub fn request(&mut self, settings: DisplaySettings) {
        self.request = Some(settings);
    }

    #[inline]
    pub fn take_request(&mut self) -> Option<DisplaySettings> {
        self.request.take()
    }
}

impl Default for DisplayState {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
