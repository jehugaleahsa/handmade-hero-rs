use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginAudioState {
    play_cursors: VecDeque<usize>,
    write_cursors: VecDeque<usize>,
    max_cursor_count: usize,
    theta: f32,
}

impl PluginAudioState {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        let play_cursors = VecDeque::new();
        let write_cursors = VecDeque::new();
        Self {
            play_cursors,
            write_cursors,
            max_cursor_count: 0,
            theta: 0f32,
        }
    }

    #[inline]
    pub fn play_cursors(&self) -> impl Iterator<Item = usize> {
        self.play_cursors.iter().copied()
    }

    #[inline]
    pub fn write_cursors(&self) -> impl Iterator<Item = usize> {
        self.write_cursors.iter().copied()
    }

    pub fn add_cursors(&mut self, play_cursor: usize, write_cursor: usize) {
        while self.play_cursors.len() >= self.max_cursor_count {
            // Both cursor collections are the same length!
            self.play_cursors.pop_front();
            self.write_cursors.pop_front();
        }
        self.play_cursors.push_back(play_cursor);
        self.write_cursors.push_back(write_cursor);
    }

    #[inline]
    pub fn set_max_cursor_count(&mut self, value: usize) {
        self.max_cursor_count = value;
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
