use serde::{Deserialize, Serialize};

/// What happened to one button over the course of a frame.
///
/// The game only samples input once per frame, but presses and releases happen between samples.
/// Storing just "is it down right now" would lose a tap that started and finished inside a single
/// frame. So a button records two things: where it ended up, and how many times it changed state
/// along the way. Each press or release is one half transition; a full tap is two.
///
/// Reading the two fields together recovers what happened:
///
/// | `half_transition_count` | `ended_down` | Meaning                                     |
/// |-------------------------|--------------|---------------------------------------------|
/// | 0                       | any          | No change. Held if `ended_down`, else idle. |
/// | 1                       | true         | Pressed this frame and still held.          |
/// | 1                       | false        | Released this frame.                        |
/// | 2                       | false        | Tapped entirely within this frame.          |
///
/// Game code rarely needs the table. **Continuous** actions like moving ask [`ended_down`]:
/// the question is "is it held right now". **Discrete** actions like jumping or toggling a menu
/// ask [`was_pressed`]: the question is "did a press happen this frame", and checking
/// `ended_down` instead would fire every frame the key is held.
///
/// [`ended_down`]: ButtonState::ended_down
/// [`was_pressed`]: ButtonState::was_pressed
#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct ButtonState {
    ended_down: bool,
    half_transition_count: u16,
}

impl ButtonState {
    /// Whether the button was down when the frame ended. The check for continuous actions.
    #[inline]
    #[must_use]
    pub fn ended_down(self) -> bool {
        self.ended_down
    }

    /// Whether the button went down at least once this frame. The check for discrete actions.
    ///
    /// A count above one means the button both rose and fell, so a press is in there somewhere.
    /// A count of exactly one is a press only if the button ended down; otherwise it was a
    /// release of a button held from the previous frame.
    #[inline]
    #[must_use]
    pub fn was_pressed(self) -> bool {
        self.half_transition_count > 1 || (self.half_transition_count == 1 && self.ended_down)
    }

    /// Whether the button came up at least once this frame. The mirror of [`was_pressed`].
    ///
    /// [`was_pressed`]: ButtonState::was_pressed
    #[inline]
    #[must_use]
    pub fn was_released(self) -> bool {
        self.half_transition_count > 1 || (self.half_transition_count == 1 && !self.ended_down)
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

    /// Records the button's current physical state, counting a transition only if it changed.
    ///
    /// Feeding the same state twice is a no-op, which is what makes it safe to call from both
    /// event-driven sources like the keyboard and polled sources like a gamepad.
    pub fn track_down(&mut self, is_pressed: bool) {
        let was_pressed = self.ended_down();
        if was_pressed != is_pressed {
            self.increment_half_transition_count();
        }
        self.set_ended_down(is_pressed);
    }
}

#[cfg(test)]
mod tests {
    use crate::button_state::ButtonState;

    #[test]
    fn test_idle_button() {
        let button = ButtonState::default();
        assert!(!button.ended_down());
        assert!(!button.was_pressed());
        assert!(!button.was_released());
    }

    #[test]
    fn test_pressed_and_still_held() {
        let mut button = ButtonState::default();
        button.track_down(true);
        assert!(button.ended_down());
        assert!(button.was_pressed());
        assert!(!button.was_released());
    }

    #[test]
    fn test_held_across_a_frame_boundary_is_not_a_new_press() {
        let mut button = ButtonState::default();
        button.track_down(true);
        button.reset_half_transition_count();
        // The key is still physically down; an autorepeat or a poll reports it again.
        button.track_down(true);
        assert!(button.ended_down());
        assert!(!button.was_pressed());
        assert!(!button.was_released());
    }

    #[test]
    fn test_released_this_frame() {
        let mut button = ButtonState::default();
        button.track_down(true);
        button.reset_half_transition_count();
        button.track_down(false);
        assert!(!button.ended_down());
        assert!(!button.was_pressed());
        assert!(button.was_released());
    }

    #[test]
    fn test_tapped_within_a_single_frame() {
        let mut button = ButtonState::default();
        button.track_down(true);
        button.track_down(false);
        assert!(!button.ended_down());
        assert_eq!(button.half_transition_count(), 2);
        assert!(button.was_pressed());
        assert!(button.was_released());
    }

    #[test]
    fn test_released_and_pressed_again_within_a_single_frame() {
        let mut button = ButtonState::default();
        button.track_down(true);
        button.reset_half_transition_count();
        button.track_down(false);
        button.track_down(true);
        assert!(button.ended_down());
        assert_eq!(button.half_transition_count(), 2);
        assert!(button.was_pressed());
        assert!(button.was_released());
    }

    #[test]
    fn test_reset_keeps_the_ended_state() {
        let mut button = ButtonState::default();
        button.track_down(true);
        button.reset_half_transition_count();
        assert!(button.ended_down());
        assert_eq!(button.half_transition_count(), 0);
    }
}
