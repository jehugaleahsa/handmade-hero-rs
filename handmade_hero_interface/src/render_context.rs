use crate::application_state::ApplicationState;
use crate::pixel::Pixel;

#[derive(Debug)]
pub struct RenderContext<'a> {
    state: &'a mut ApplicationState,
    bitmap_buffer: &'a mut [Pixel],
    width: u16,
    height: u16,
}

impl<'a> RenderContext<'a> {
    #[inline]
    #[must_use]
    pub fn new(
        state: &'a mut ApplicationState,
        bitmap_buffer: &'a mut [Pixel],
        width: u16,
        height: u16,
    ) -> Self {
        Self {
            state,
            bitmap_buffer,
            width,
            height,
        }
    }

    #[inline]
    #[must_use]
    pub fn x_offset(&self) -> u16 {
        self.state.x_offset()
    }

    #[inline]
    #[must_use]
    pub fn y_offset(&self) -> u16 {
        self.state.y_offset()
    }

    #[inline]
    pub fn set_pixel(&mut self, index: usize, pixel: Pixel) {
        self.bitmap_buffer[index] = pixel;
    }

    #[inline]
    #[must_use]
    pub fn width(&self) -> u16 {
        self.width
    }

    #[inline]
    #[must_use]
    pub fn height(&self) -> u16 {
        self.height
    }
}
