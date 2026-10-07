use uom::si::u32::Frequency;

use crate::dimensions::Dimensions;

#[derive(Debug, Clone)]
pub struct MonitorMode {
    resolution: Dimensions,
    refresh_rate: Frequency,
    current: bool,
}

impl MonitorMode {
    #[inline]
    #[must_use]
    pub fn new(resolution: Dimensions, refresh_rate: Frequency, current: bool) -> Self {
        Self {
            resolution,
            refresh_rate,
            current,
        }
    }

    #[inline]
    #[must_use]
    pub fn resolution(&self) -> Dimensions {
        self.resolution
    }

    #[inline]
    #[must_use]
    pub fn refresh_rate(&self) -> Frequency {
        self.refresh_rate
    }

    #[inline]
    #[must_use]
    pub fn current(&self) -> bool {
        self.current
    }

    /// Whether both describe the same resolution and refresh rate, regardless of which one is
    /// current.
    #[inline]
    #[must_use]
    pub fn matches(&self, other: &MonitorMode) -> bool {
        self.resolution == other.resolution && self.refresh_rate == other.refresh_rate
    }
}
