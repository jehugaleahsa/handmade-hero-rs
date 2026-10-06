use uom::si::u32::Frequency;

use crate::units::si::length::Length;

#[derive(Debug)]
pub struct Monitor {
    width: Length,
    height: Length,
    refresh_rate: Frequency,
    primary: bool,
}

impl Monitor {
    #[inline]
    #[must_use]
    pub fn new(width: Length, height: Length, refresh_rate: Frequency, primary: bool) -> Self {
        Self {
            width,
            height,
            refresh_rate,
            primary,
        }
    }

    #[inline]
    #[must_use]
    pub fn width(&self) -> Length {
        self.width
    }

    #[inline]
    #[must_use]
    pub fn height(&self) -> Length {
        self.height
    }

    #[inline]
    #[must_use]
    pub fn refresh_rate(&self) -> Frequency {
        self.refresh_rate
    }

    #[inline]
    #[must_use]
    pub fn primary(&self) -> bool {
        self.primary
    }
}
