/// A width and height counted in screen pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Resolution {
    width_px: u32,
    height_px: u32,
}

impl Resolution {
    #[inline]
    #[must_use]
    pub fn new(width_px: u32, height_px: u32) -> Self {
        Self {
            width_px,
            height_px,
        }
    }

    #[inline]
    #[must_use]
    pub fn width_px(&self) -> u32 {
        self.width_px
    }

    #[inline]
    #[must_use]
    pub fn height_px(&self) -> u32 {
        self.height_px
    }
}
