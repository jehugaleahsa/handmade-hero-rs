use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginAudioState {
    play_cursors: VecDeque<usize>,
    write_cursors: VecDeque<usize>,
    theta: f32,
}

impl PluginAudioState {
    const MAX_CURSOR_COUNT: usize = 30;

    #[inline]
    #[must_use]
    pub fn new() -> Self {
        let play_cursors = VecDeque::with_capacity(Self::MAX_CURSOR_COUNT);
        let write_cursors = VecDeque::with_capacity(Self::MAX_CURSOR_COUNT);
        Self {
            play_cursors,
            write_cursors,
            theta: 0f32,
        }
    }

    #[inline]
    #[must_use]
    pub fn play_cursors(&self) -> &[usize] {
        // The call to make_contiguous ensures the first slice contains all cursors
        let (first, _) = self.play_cursors.as_slices();
        first
    }

    #[inline]
    #[must_use]
    pub fn write_cursors(&self) -> &[usize] {
        // The call to make_contiguous ensures the first slice contains all cursors
        let (first, _) = self.write_cursors.as_slices();
        first
    }

    pub fn add_cursors(&mut self, play_cursor: usize, write_cursor: usize) {
        if self.play_cursors.len() == Self::MAX_CURSOR_COUNT {
            // Both cursor collections are the same length!
            self.play_cursors.pop_front();
            self.write_cursors.pop_front();
        }
        self.play_cursors.push_back(play_cursor);
        self.write_cursors.push_back(write_cursor);
        self.play_cursors.make_contiguous();
        self.write_cursors.make_contiguous();
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
