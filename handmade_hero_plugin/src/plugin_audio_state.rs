use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginAudioState {
    play_cursors: [usize; 30],
    write_cursors: [usize; 30],
    cursor_index: usize,
    theta: f32,
}

impl PluginAudioState {
    const MAX_CURSOR_COUNT: usize = 30;

    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            play_cursors: [0; Self::MAX_CURSOR_COUNT],
            write_cursors: [0; Self::MAX_CURSOR_COUNT],
            cursor_index: 0,
            theta: 0f32,
        }
    }

    #[inline]
    #[must_use]
    pub fn play_cursors(&self) -> &[usize] {
        &self.play_cursors[0..self.cursor_index]
    }

    #[inline]
    #[must_use]
    pub fn write_cursors(&self) -> &[usize] {
        &self.write_cursors
    }

    #[inline]
    pub fn add_cursors(&mut self, play_cursor: usize, write_cursor: usize) {
        self.play_cursors[self.cursor_index] = play_cursor;
        self.write_cursors[self.cursor_index] = write_cursor;
        self.cursor_index += 1;
        if self.cursor_index == Self::MAX_CURSOR_COUNT {
            self.cursor_index = 0;
        }
    }

    #[inline]
    #[must_use]
    pub fn theta(&self) -> f32 {
        self.theta
    }

    #[inline]
    pub fn set_theta(&mut self, value: f32) {
        self.theta = value;
    }
}
