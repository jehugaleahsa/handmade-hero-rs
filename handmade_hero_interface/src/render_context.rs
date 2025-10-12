use crate::game_state::GameState;
use crate::pixel::Pixel;

#[derive(Debug)]
pub struct RenderContext<'a> {
    state: &'a mut GameState,
    bitmap_buffer: &'a mut [Pixel],
}

impl<'a> RenderContext<'a> {
    #[inline]
    #[must_use]
    pub fn new(state: &'a mut GameState, bitmap_buffer: &'a mut [Pixel]) -> Self {
        Self {
            state,
            bitmap_buffer,
        }
    }

    #[inline]
    #[must_use]
    pub fn height(&self) -> u16 {
        self.state.height()
    }

    #[inline]
    #[must_use]
    pub fn width(&self) -> u16 {
        self.state.width()
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
    #[must_use]
    pub fn player_x(&self) -> u16 {
        self.state.player_x()
    }

    #[inline]
    #[must_use]
    pub fn player_y(&self) -> u16 {
        self.state.player_y()
    }

    #[inline]
    #[must_use]
    pub fn jump_time(&self) -> f32 {
        self.state.jump_time()
    }

    #[inline]
    pub fn set_pixel(&mut self, index: usize, pixel: Pixel) {
        self.bitmap_buffer[index] = pixel;
    }
}
