use crate::monitor_mode::MonitorMode;

#[derive(Debug, Clone)]
pub struct Monitor {
    display_name: String,
    identifier: String,
    width_in_pixels: u32,
    height_in_pixels: u32,
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
        width_in_pixels: u32,
        height_in_pixels: u32,
        modes: Vec<MonitorMode>,
        primary: bool,
        current: bool,
    ) -> Self {
        Self {
            display_name,
            identifier,
            width_in_pixels,
            height_in_pixels,
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
    pub fn width_in_pixels(&self) -> u32 {
        self.width_in_pixels
    }

    #[inline]
    #[must_use]
    pub fn height_in_pixels(&self) -> u32 {
        self.height_in_pixels
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
