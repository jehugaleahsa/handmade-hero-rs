use handmade_hero_interface::{back_buffer::BackBuffer, vector_2d::Vector2d};

#[derive(Debug)]
pub struct Image {
    buffer: BackBuffer,
    offset_px: Vector2d<isize>,
}

impl Image {
    #[inline]
    #[must_use]
    pub fn new(buffer: BackBuffer, offset_px: Vector2d<isize>) -> Self {
        Self { buffer, offset_px }
    }

    #[inline]
    #[must_use]
    pub fn buffer(&self) -> &BackBuffer {
        &self.buffer
    }

    pub fn offset_px(&self) -> Vector2d<isize> {
        self.offset_px
    }
}
