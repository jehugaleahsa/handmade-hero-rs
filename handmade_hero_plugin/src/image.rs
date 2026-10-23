use handmade_hero_interface::back_buffer::BackBuffer;

#[derive(Debug)]
pub struct Image {
    buffer: BackBuffer,
    align_x: usize,
    align_y: usize,
}

impl Image {
    #[inline]
    #[must_use]
    pub fn new(buffer: BackBuffer, align_x: usize, align_y: usize) -> Self {
        Self {
            buffer,
            align_x,
            align_y,
        }
    }

    #[inline]
    #[must_use]
    pub fn buffer(&self) -> &BackBuffer {
        &self.buffer
    }

    #[inline]
    #[must_use]
    pub fn align_x(&self) -> usize {
        self.align_x
    }

    #[inline]
    #[must_use]
    pub fn align_y(&self) -> usize {
        self.align_y
    }
}
