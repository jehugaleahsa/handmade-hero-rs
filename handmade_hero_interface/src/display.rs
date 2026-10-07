use std::fmt::{Debug, Formatter};

use crate::{
    application_error::Result, display_settings::DisplaySettings, display_state::DisplayState,
    monitor::Monitor,
};

pub struct Display<'a> {
    list_monitors: &'a dyn Fn() -> Vec<Monitor>,
    state: &'a mut DisplayState,
}

impl<'a> Display<'a> {
    #[inline]
    #[must_use]
    pub fn new(list_monitors: &'a dyn Fn() -> Vec<Monitor>, state: &'a mut DisplayState) -> Self {
        Self {
            list_monitors,
            state,
        }
    }

    /// Builds a fresh copy of every monitor and its modes, so ask only when the list is needed,
    /// such as when a menu opens, rather than every frame.
    #[inline]
    #[must_use]
    pub fn monitors(&self) -> Vec<Monitor> {
        (self.list_monitors)()
    }

    #[inline]
    #[must_use]
    pub fn current(&self) -> &DisplaySettings {
        self.state.current()
    }

    #[inline]
    #[must_use]
    pub fn last_request(&self) -> Option<&Result<()>> {
        self.state.last_request()
    }

    #[inline]
    pub fn request(&mut self, settings: DisplaySettings) {
        self.state.request(settings);
    }
}

impl Debug for Display<'_> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Display")
            .field("state", &self.state)
            .finish_non_exhaustive()
    }
}
