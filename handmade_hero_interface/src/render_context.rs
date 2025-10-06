use crate::application_state::ApplicationState;
use crate::pixel::Pixel;

#[derive(Debug)]
pub struct RenderContext<'a> {
    state: &'a mut ApplicationState,
    bitmap_buffer: &'a mut [Pixel],
}

impl<'a> RenderContext<'a> {
    #[inline]
    #[must_use]
    pub fn new(state: &'a mut ApplicationState, bitmap_buffer: &'a mut [Pixel]) -> Self {
        Self {
            state,
            bitmap_buffer,
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
        self.state.width()
    }

    #[inline]
    #[must_use]
    pub fn height(&self) -> u16 {
        self.state.height()
    }

    #[inline]
    #[must_use]
    pub fn player_x(&self) -> u16 {
        self.state.player_x()
    }

    #[inline]
    pub fn set_player_x(&mut self, value: u16) {
        self.state.set_player_x(value);
    }

    #[inline]
    #[must_use]
    pub fn player_y(&self) -> u16 {
        self.state.player_y()
    }

    #[inline]
    pub fn set_player_y(&mut self, value: u16) {
        self.state.set_player_y(value);
    }

    #[inline]
    #[must_use]
    pub fn jump_time(&self) -> f32 {
        self.state.jump_time()
    }
}
