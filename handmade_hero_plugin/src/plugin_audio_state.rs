use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct FlipData {
    play_cursor: usize,
    write_cursor: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginAudioState {
    flips: VecDeque<FlipData>,
    max_cursor_count: usize,
    theta: f32,
}

impl PluginAudioState {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        let flips = VecDeque::new();
        Self {
            flips,
            max_cursor_count: 0,
            theta: 0f32,
        }
    }

    #[inline]
    #[must_use]
    pub fn flips_len(&self) -> usize {
        self.flips.len()
    }

    #[inline]
    pub fn flip_play_cursors(&self) -> impl Iterator<Item = usize> {
        self.flips.iter().map(|o| o.play_cursor)
    }

    #[inline]
    pub fn flip_write_cursors(&self) -> impl Iterator<Item = usize> {
        self.flips.iter().map(|o| o.write_cursor)
    }

    pub fn add_flip_data(&mut self, play_cursor: usize, write_cursor: usize) {
        while self.flips.len() >= self.max_cursor_count {
            self.flips.pop_front();
        }
        let flips = FlipData {
            play_cursor,
            write_cursor,
        };
        self.flips.push_back(flips);
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
