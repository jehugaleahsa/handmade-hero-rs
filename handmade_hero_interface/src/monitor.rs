use uom::si::u32::Frequency;

use crate::units::si::length::Length;

#[derive(Debug, Clone)]
pub struct Monitor {
    display_name: String,
    identifier: String,
    width: Length,
    height: Length,
    refresh_rate: Frequency,
    primary: bool,
    current: bool,
}

impl Monitor {
    #[inline]
    #[must_use]
    pub fn new(
        display_name: String,
        identifier: String,
        width: Length,
        height: Length,
        refresh_rate: Frequency,
        primary: bool,
        current: bool,
    ) -> Self {
        Self {
            display_name,
            identifier,
            width,
            height,
            refresh_rate,
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

    #[inline]
    #[must_use]
    pub fn current(&self) -> bool {
        self.current
    }
}
