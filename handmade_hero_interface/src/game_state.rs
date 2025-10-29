use crate::sound_state::SoundState;
use bincode::{Decode, Encode};

#[derive(Debug, Encode, Decode)]
pub struct GameState {
    width: u16,
    height: u16,
    sound: SoundState,
}

impl GameState {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        let sound = SoundState::new();
        Self {
            width: 0,
            height: 0,
            sound,
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
}

impl Default for GameState {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
