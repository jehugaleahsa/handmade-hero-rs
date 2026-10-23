use handmade_hero_interface::back_buffer::BackBuffer;

#[derive(Debug)]
pub struct Image {
    buffer: BackBuffer,
    x_offset_px: Option<isize>,
    y_offset_px: Option<isize>,
}

impl Image {
    #[inline]
    #[must_use]
    pub fn new(buffer: BackBuffer, x_offset_px: Option<isize>, y_offset_px: Option<isize>) -> Self {
        Self {
            buffer,
            x_offset_px,
            y_offset_px,
        }
    }

    #[inline]
    #[must_use]
    pub fn buffer(&self) -> &BackBuffer {
        &self.buffer
    }

    #[inline]
    #[must_use]
    pub fn x_offset_px(&self) -> Option<isize> {
        self.x_offset_px
    }

    #[inline]
    #[must_use]
    pub fn y_offset_px(&self) -> Option<isize> {
        self.y_offset_px
    }
}
