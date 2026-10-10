/// A width and height counted in screen pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Resolution {
    width_in_pixels: u32,
    height_in_pixels: u32,
}

impl Resolution {
    #[inline]
    #[must_use]
    pub fn new(width_in_pixels: u32, height_in_pixels: u32) -> Self {
        Self {
            width_in_pixels,
            height_in_pixels,
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
}
