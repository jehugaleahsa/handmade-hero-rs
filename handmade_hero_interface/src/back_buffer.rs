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

    pub fn copy_from_buffer(&mut self, source: &BackBuffer, x_offset: isize, y_offset: isize) {
        let source_width = source.width_px();
        let destination_width = self.width_px();

        let Some((min_x, max_x, source_offset_x)) =
            Self::clamp(x_offset, source_width, destination_width)
        else {
            return;
        };
        let Some((min_y, max_y, source_offset_y)) =
            Self::clamp(y_offset, source.height_px(), self.height_px())
        else {
            return;
        };
        if min_x == max_x {
            return;
        }

        let mut source_row = source_offset_y * source_width + source_offset_x;
        let mut destination_row = min_y * destination_width + min_x;

        let source_pixels = source.pixels();
        let destination_pixels = self.pixels_mut();
        for _ in min_y..max_y {
            let mut source_column_offset = source_row;
            let mut destination_column_offset = destination_row;
            #[expect(clippy::explicit_counter_loop)] // <-- Suggestion is too complicated
            for _ in min_x..max_x {
                let source_pixel = source_pixels[source_column_offset];
                let destination_pixel = destination_pixels[destination_column_offset];
                let blended_pixel = Self::blend_pixel(source_pixel, destination_pixel);
                destination_pixels[destination_column_offset] = blended_pixel;
                source_column_offset += 1;
                destination_column_offset += 1;
            }
            source_row += source_width;
            destination_row += destination_width;
        }
    }

    fn clamp(
        delta: isize,
        source_max: usize,
        destination_max: usize,
    ) -> Option<(usize, usize, usize)> {
        let mut offset = 0;
        let mut start = 0;
        let mut end = source_max;
        if delta < 0 {
            offset = delta.unsigned_abs();
            if offset > source_max {
                return None;
            }
            end -= offset;
        } else {
            start = delta.unsigned_abs();
            end += start;
        }
        if end > destination_max {
            end = destination_max;
        }
        Some((start, end, offset))
    }

    fn blend_pixel(a: Color<u8>, b: Color<u8>) -> Color<u8> {
        let alpha = f32::from(a.alpha()) / f32::from(u8::MAX);
        let red = Self::blend_channel(a.red(), b.red(), alpha);
        let green = Self::blend_channel(a.green(), b.green(), alpha);
        let blue = Self::blend_channel(a.blue(), b.blue(), alpha);
        Color::from_rgba(red, green, blue, u8::MAX)
    }

    #[inline]
    #[expect(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    fn blend_channel(a: u8, b: u8, t: f32) -> u8 {
        let blended = (1.0 - t) * f32::from(b) + t * f32::from(a);
        blended as u8
    }
}
