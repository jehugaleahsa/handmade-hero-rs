use uom::si::u32::Frequency;

#[derive(Debug, Clone)]
pub struct MonitorMode {
    width_in_pixels: u32,
    height_in_pixels: u32,
    refresh_rate: Frequency,
    current: bool,
}

impl MonitorMode {
    #[inline]
    #[must_use]
    pub fn new(
        width_in_pixels: u32,
        height_in_pixels: u32,
        refresh_rate: Frequency,
        current: bool,
    ) -> Self {
        Self {
            width_in_pixels,
            height_in_pixels,
            refresh_rate,
            current,
        }
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
    pub fn refresh_rate(&self) -> Frequency {
        self.refresh_rate
    }

    #[inline]
    #[must_use]
    pub fn current(&self) -> bool {
        self.current
    }
}
