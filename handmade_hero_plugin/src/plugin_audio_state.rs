use std::collections::VecDeque;

use serde::{Deserialize, Serialize};
use uom::si::u32::Information;

#[derive(Debug, Serialize, Deserialize)]
struct OutputData {
    play_cursor: usize,
    write_cursor: usize,
    write_offset: usize,
    write_length: Information,
}

#[derive(Debug, Serialize, Deserialize)]
struct FlipData {
    play_cursor: usize,
    write_cursor: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginAudioState {
    output: Option<OutputData>,
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
            output: None,
            flips,
            max_cursor_count: 0,
            theta: 0f32,
        }
    }

    #[inline]
    pub fn output_play_cursor(&self) -> Option<usize> {
        self.output.as_ref().map(|o| o.play_cursor)
    }

    #[inline]
    pub fn output_write_cursor(&self) -> Option<usize> {
        self.output.as_ref().map(|o| o.write_cursor)
    }

    #[inline]
    pub fn output_write_offset(&self) -> Option<usize> {
        self.output.as_ref().map(|o| o.write_offset)
    }

    #[inline]
    pub fn output_write_length(&self) -> Option<Information> {
        self.output.as_ref().map(|o| o.write_length)
    }

    pub fn set_output_data(
        &mut self,
        play_cursor: usize,
        write_cursor: usize,
        write_offset: usize,
        write_length: Information,
    ) {
        let output = OutputData {
            play_cursor,
            write_cursor,
            write_offset,
            write_length,
        };
        self.output = Some(output);
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
