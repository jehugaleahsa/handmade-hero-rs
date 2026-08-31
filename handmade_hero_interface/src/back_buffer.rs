use std::cmp::Ordering;
use std::ffi::c_void;

use crate::application_error::{ApplicationError, Result};
use crate::units::si::length::pixel;
use crate::{color::Color, units::si::length::Length};

#[derive(Debug, Default)]
pub struct BackBuffer {
    pixels: Vec<Color<u8>>,
    height: Length,
    width: Length,
}

impl BackBuffer {
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

    /// # Errors
    /// An error is returned if the buffer width and height, when multiplied together, do not
    /// fit within a usize.
    pub fn resize(&mut self, width: Length, height: Length) -> Result<()> {
        #[expect(clippy::cast_possible_truncation)]
        #[expect(clippy::cast_sign_loss)]
        let width_pixels = width.get::<pixel>() as usize;
        #[expect(clippy::cast_possible_truncation)]
        #[expect(clippy::cast_sign_loss)]
        let height_pixels = height.get::<pixel>() as usize;
        let pixel_count = width_pixels
            .checked_mul(height_pixels)
            .ok_or_else(|| ApplicationError::new("The pixel count did not fit in a usize"))?;
        match pixel_count.cmp(&self.pixels.len()) {
            Ordering::Greater => self.pixels.resize(pixel_count, Color::default()),
            Ordering::Less => self.pixels.truncate(pixel_count),
            Ordering::Equal => {}
        }
        self.width = width;
        self.height = height;
        Ok(())
    }

    #[inline]
    #[must_use]
    pub fn bitmap(&self) -> *const c_void {
        self.pixels.as_ptr().cast::<c_void>()
    }

    #[inline]
    pub fn pixels_mut(&mut self) -> &mut [Color<u8>] {
        self.pixels.as_mut_slice()
    }
}
