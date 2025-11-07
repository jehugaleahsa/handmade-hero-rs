use crate::point_2d::Point2d;
use crate::sound_state::SoundState;
use bincode::{Decode, Encode};
use std::time::Duration;

#[derive(Debug, Encode, Decode)]
pub struct GameState {
    width: u16,
    height: u16,
    sound: SoundState,
    player: Point2d,
    frame_duration: Duration,
}

impl GameState {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        let sound = SoundState::new();
        let player = Point2d::default();
        Self {
            width: 0,
            height: 0,
            sound,
            player,
            frame_duration: Duration::default(),
        }
    }

    #[inline]
    #[must_use]
    pub fn sound(&self) -> &SoundState {
        &self.sound
    }

    #[inline]
    #[must_use]
    pub fn sound_mut(&mut self) -> &mut SoundState {
        &mut self.sound
    }

    #[inline]
    #[must_use]
    pub fn width(&self) -> u16 {
        self.width
    }

    #[inline]
    pub fn set_width(&mut self, value: u16) {
        self.width = value;
    }

    #[inline]
    #[must_use]
    pub fn height(&self) -> u16 {
        self.height
    }

    #[inline]
    pub fn set_height(&mut self, value: u16) {
        self.height = value;
    }

    #[inline]
    #[must_use]
    pub fn frame_duration(&self) -> Duration {
        self.frame_duration
    }

    #[inline]
    pub fn set_frame_duration(&mut self, value: Duration) {
        self.frame_duration = value;
    }

    #[inline]
    #[must_use]
    pub fn player(&self) -> Point2d {
        self.player
    }

    #[inline]
    pub fn set_player(&mut self, player: Point2d) {
        self.player = player;
    }
}

impl Default for GameState {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
