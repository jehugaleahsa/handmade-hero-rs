use std::cmp::Ordering;
use std::ffi::c_void;

use crate::application_error::{ApplicationError, Result};
use crate::color::Color;

#[derive(Debug, Default)]
pub struct BackBuffer {
    pixels: Vec<Color<u8>>,
    height_px: usize,
    width_px: usize,
}

impl BackBuffer {
    #[inline]
    #[must_use]
    pub fn width_px(&self) -> usize {
        self.width_px
    }

    #[inline]
    #[must_use]
    pub fn height_px(&self) -> usize {
        self.height_px
    }

    /// # Errors
    /// An error is returned if the buffer width and height, when multiplied together, do not
    /// fit within a usize.
    pub fn resize(&mut self, width_px: usize, height_px: usize) -> Result<()> {
        let pixel_count = width_px
            .checked_mul(height_px)
            .ok_or_else(|| ApplicationError::new("The pixel count did not fit in a usize"))?;
        match pixel_count.cmp(&self.pixels.len()) {
            Ordering::Greater => self.pixels.resize(pixel_count, Color::default()),
            Ordering::Less => self.pixels.truncate(pixel_count),
            Ordering::Equal => {}
        }
        self.width_px = width_px;
        self.height_px = height_px;
        Ok(())
    }

    #[inline]
    #[must_use]
    pub fn bitmap(&self) -> *const c_void {
        self.pixels.as_ptr().cast::<c_void>()
    }

    #[inline]
    #[must_use]
    pub fn pixels(&self) -> &[Color<u8>] {
        self.pixels.as_ref()
    }

    #[inline]
    #[must_use]
    pub fn pixels_mut(&mut self) -> &mut [Color<u8>] {
        self.pixels.as_mut_slice()
    }
}
