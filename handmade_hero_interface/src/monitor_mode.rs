use uom::si::u32::Frequency;

#[derive(Debug, Clone)]
pub struct MonitorMode {
    refresh_rate: Frequency,
    current: bool,
}

impl MonitorMode {
    #[inline]
    #[must_use]
    pub fn new(refresh_rate: Frequency, current: bool) -> Self {
        Self {
            refresh_rate,
            current,
        }
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
}
