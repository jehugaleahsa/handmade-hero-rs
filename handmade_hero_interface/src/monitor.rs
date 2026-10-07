use crate::{dimensions::Dimensions, monitor_mode::MonitorMode};

#[derive(Debug, Clone)]
pub struct Monitor {
    display_name: String,
    identifier: String,
    work_resolution: Dimensions,
    max_windowed_resolution: Dimensions,
    modes: Vec<MonitorMode>,
    primary: bool,
    current: bool,
}

impl Monitor {
    #[inline]
    #[must_use]
    pub fn new(
        display_name: String,
        identifier: String,
        work_resolution: Dimensions,
        max_windowed_resolution: Dimensions,
        modes: Vec<MonitorMode>,
        primary: bool,
        current: bool,
    ) -> Self {
        Self {
            display_name,
            identifier,
            work_resolution,
            max_windowed_resolution,
            modes,
            primary,
            current,
        }
    }

    #[inline]
    #[must_use]
    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    #[inline]
    #[must_use]
    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    #[inline]
    #[must_use]
    pub fn work_resolution(&self) -> Dimensions {
        self.work_resolution
    }

    #[inline]
    #[must_use]
    pub fn max_windowed_resolution(&self) -> Dimensions {
        self.max_windowed_resolution
    }

    #[inline]
    #[must_use]
    pub fn modes(&self) -> &[MonitorMode] {
        &self.modes
    }

    #[inline]
    #[must_use]
    pub fn current_mode(&self) -> Option<&MonitorMode> {
        self.modes.iter().find(|mode| mode.current())
    }

    #[inline]
    #[must_use]
    pub fn primary(&self) -> bool {
        self.primary
    }

    #[inline]
    #[must_use]
    pub fn current(&self) -> bool {
        self.current
    }
}
