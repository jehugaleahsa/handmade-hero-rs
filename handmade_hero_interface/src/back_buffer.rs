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

    #[allow(dead_code)]
    pub fn copy_from_buffer_repeating(
        &mut self,
        source: &BackBuffer,
        repeat_x: bool,
        repeat_y: bool,
    ) {
        let source_height = source.height_px();
        let source_width = source.width_px();
        let destination_height = self.height_px();
        let destination_width = self.width_px();
        let mut start_y = 0;
        while start_y < destination_height {
            let mut start_x = 0;
            while start_x < destination_width {
                self.copy_from_buffer(source, start_x, start_y);
                if repeat_x {
                    start_x += source_width;
                } else {
                    break;
                }
            }
            if repeat_y {
                start_y += source_height;
            } else {
                break;
            }
        }
    }

    pub fn copy_from_buffer(&mut self, source: &BackBuffer, x_offset: usize, y_offset: usize) {
        let source_width = source.width_px();
        let destination_width = self.width_px();
        let clipped_source_width = Self::clipped(source_width, x_offset, destination_width);
        if clipped_source_width == 0 {
            return;
        }

        let source_height = source.height_px();
        let destination_height = self.height_px();
        let clipped_source_height = Self::clipped(source_height, y_offset, destination_height);

        let source_pixels = source.pixels();
        let destination_pixels = self.pixels_mut();
        let mut source_row_offset = 0;
        let mut destination_row_offset = y_offset * destination_width + x_offset;
        for _ in 0..clipped_source_height {
            for (source_column_offset, destination_column_offset) in (source_row_offset..)
                .take(clipped_source_width)
                .zip(destination_row_offset..)
            {
                let source_pixel = source_pixels[source_column_offset];
                let destination_pixel = destination_pixels[destination_column_offset];
                let blended_pixel = Self::blend_pixel(source_pixel, destination_pixel);
                destination_pixels[destination_column_offset] = blended_pixel;
            }
            // Not clipped widths!
            source_row_offset += source_width;
            destination_row_offset += destination_width;
        }
    }

    fn clipped(start: usize, offset: usize, max: usize) -> usize {
        let end = start + offset;
        if end > max {
            let overflow = end - max;
            start.saturating_sub(overflow)
        } else {
            start
        }
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
