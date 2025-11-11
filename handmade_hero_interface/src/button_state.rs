use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct ButtonState {
    ended_down: bool,
    half_transition_count: u16,
}

impl ButtonState {
    #[inline]
    #[must_use]
    pub fn ended_down(self) -> bool {
        self.ended_down
    }

    #[inline]
    pub fn set_ended_down(&mut self, value: bool) {
        self.ended_down = value;
    }

    #[inline]
    #[must_use]
    pub fn half_transition_count(self) -> u16 {
        self.half_transition_count
    }

    #[inline]
    pub fn increment_half_transition_count(&mut self) {
        self.half_transition_count = self.half_transition_count.saturating_add(1);
    }

    #[inline]
    pub fn reset_half_transition_count(&mut self) {
        self.half_transition_count = 0;
    }

    pub fn clear(&mut self) {
        self.ended_down = false;
        self.half_transition_count = 0;
    }
}
