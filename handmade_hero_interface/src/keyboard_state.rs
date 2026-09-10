use serde::{Deserialize, Serialize};
use smallvec::{SmallVec, smallvec};

use crate::button::Button;
use crate::button_state::ButtonState;
use crate::controller_state::ControllerState;
use crate::key::Key;
use crate::key_mapping::KeyMapping;

/// The raw state of every key, plus the controller those keys are mapped onto.
///
/// The platform layer feeds this one key at a time as key events arrive. The keyboard state is
/// the source of truth; the controller is derived from it, so the two can never disagree. That
/// is what fixes the "hold W, tap Up Arrow, character stops" bug: a release only lowers a button
/// when *no* key bound to that button is still held.
///
/// The mapping is not stored here. This type is serialized as part of the game's input so a
/// recording can be replayed, and the bindings are configuration rather than input. The caller
/// passes the mapping in with each key event instead.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardState {
    keys: SmallVec<[ButtonState; Key::COUNT]>,
    controller: ControllerState,
}

impl KeyboardState {
    #[must_use]
    pub fn new() -> Self {
        let keys = smallvec![ButtonState::default(); Key::COUNT];
        // A keyboard is always present, unlike a gamepad that may be unplugged.
        let mut controller = ControllerState::default();
        controller.set_enabled(true);
        Self { keys, controller }
    }

    #[inline]
    #[must_use]
    pub fn key(&self, key: Key) -> &ButtonState {
        &self.keys[key.index()]
    }

    #[inline]
    #[must_use]
    pub fn is_key_down(&self, key: Key) -> bool {
        self.key(key).ended_down()
    }

    /// Whether either Shift key is held.
    #[inline]
    #[must_use]
    pub fn is_shift_down(&self) -> bool {
        self.is_key_down(Key::LeftShift) || self.is_key_down(Key::RightShift)
    }

    /// Whether either Control key is held.
    #[inline]
    #[must_use]
    pub fn is_control_down(&self) -> bool {
        self.is_key_down(Key::LeftControl) || self.is_key_down(Key::RightControl)
    }

    /// Whether either Alt key is held.
    #[inline]
    #[must_use]
    pub fn is_alt_down(&self) -> bool {
        self.is_key_down(Key::LeftAlt) || self.is_key_down(Key::RightAlt)
    }

    /// The controller state mirroring the currently captured keyboard events.
    #[inline]
    #[must_use]
    pub fn as_controller(&self) -> &ControllerState {
        &self.controller
    }

    /// Applies a full snapshot of which keys are physically down.
    ///
    /// Use this when the window regains keyboard focus. Any key the user pressed or released
    /// while another window had focus never sent this window a message, so the tracked state is
    /// stale until it is reconciled with what the operating system reports.
    pub fn synchronize(
        &mut self,
        mapping: &KeyMapping,
        states: impl IntoIterator<Item = (Key, bool)>,
    ) {
        for (key, is_down) in states {
            self.track_key(mapping, key, is_down);
        }
    }

    /// Records that `key` is now down or up, and updates the button it drives, if any.
    ///
    /// The button is derived here, per event, rather than once per frame. A key that is pressed
    /// and released between two frames still leaves a half-transition count of two on its
    /// button, so a discrete action such as opening a menu does not miss the tap. Sampling the
    /// keys only at frame end would see the key up both times and record nothing.
    ///
    /// Repeated events for a key already in that state are harmless: neither the key nor its
    /// button counts a transition. The platform should still filter autorepeat when it can,
    /// purely to avoid the wasted work.
    pub fn track_key(&mut self, mapping: &KeyMapping, key: Key, is_down: bool) {
        self.keys[key.index()].track_down(is_down);
        if let Some(button) = mapping.button_for(key) {
            self.derive_button(mapping, button);
        }
    }

    /// A button is down while any key bound to it is down. `track_down` only counts a
    /// transition when that OR flips, so holding two keys for one button is one press.
    fn derive_button(&mut self, mapping: &KeyMapping, button: Button) {
        let any_down = mapping
            .keys_for(button)
            .any(|key| self.keys[key.index()].ended_down());
        self.controller.button_mut(button).track_down(any_down);
    }

    /// Releases every key, counting a transition for each one that was held.
    ///
    /// Use this when the window loses keyboard focus. The release for any key still held goes
    /// to whichever window has focus next, so without this the key stays down forever from the
    /// game's point of view.
    pub fn release_all(&mut self) {
        for key in &mut self.keys {
            key.track_down(false);
        }
        for button in Button::ALL {
            self.controller.button_mut(button).track_down(false);
        }
    }

    /// Zeros every half-transition count. Call once at the start of each frame.
    pub fn reset_counts(&mut self) {
        for key in &mut self.keys {
            key.reset_half_transition_count();
        }
        self.controller.reset_counts();
    }
}

impl Default for KeyboardState {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::button::Button;
    use crate::key::Key;
    use crate::key_mapping::KeyMapping;
    use crate::keyboard_state::KeyboardState;

    fn up_button(keyboard: &KeyboardState) -> (bool, u16) {
        let up = keyboard.as_controller().button(Button::Up);
        (up.ended_down(), up.half_transition_count())
    }

    #[test]
    fn test_single_key_press_and_release() {
        let mapping = KeyMapping::default();
        let mut keyboard = KeyboardState::default();

        keyboard.track_key(&mapping, Key::W, true);
        assert!(keyboard.is_key_down(Key::W));
        assert_eq!(up_button(&keyboard), (true, 1));

        keyboard.track_key(&mapping, Key::W, false);
        assert!(!keyboard.is_key_down(Key::W));
        assert_eq!(up_button(&keyboard), (false, 2));
    }

    #[test]
    fn test_tap_within_a_frame_reaches_the_button() {
        // Press and release between two frames: the key ends up, but the tap must still show.
        let mapping = KeyMapping::default();
        let mut keyboard = KeyboardState::default();
        keyboard.track_key(&mapping, Key::Escape, true);
        keyboard.track_key(&mapping, Key::Escape, false);

        let start = keyboard.as_controller().button(Button::Start);
        assert!(!start.ended_down());
        assert_eq!(start.half_transition_count(), 2);
        assert!(start.was_pressed());
    }

    #[test]
    fn test_autorepeat_does_not_count_as_a_transition() {
        let mapping = KeyMapping::default();
        let mut keyboard = KeyboardState::default();
        keyboard.track_key(&mapping, Key::W, true);
        keyboard.track_key(&mapping, Key::W, true);
        keyboard.track_key(&mapping, Key::W, true);
        assert_eq!(up_button(&keyboard), (true, 1));
    }

    #[test]
    fn test_button_stays_down_while_any_bound_key_is_held() {
        // Hold W, tap Up Arrow, and the character must keep moving.
        let mapping = KeyMapping::default();
        let mut keyboard = KeyboardState::default();
        keyboard.track_key(&mapping, Key::W, true);
        keyboard.track_key(&mapping, Key::Up, true);
        assert_eq!(up_button(&keyboard), (true, 1));

        keyboard.track_key(&mapping, Key::Up, false);
        assert_eq!(up_button(&keyboard), (true, 1));

        keyboard.track_key(&mapping, Key::W, false);
        assert_eq!(up_button(&keyboard), (false, 2));
    }

    #[test]
    fn test_unbound_keys_do_not_touch_the_controller() {
        let mapping = KeyMapping::default();
        let mut keyboard = KeyboardState::default();
        keyboard.track_key(&mapping, Key::L, true);
        assert!(keyboard.is_key_down(Key::L));
        for button in Button::ALL {
            let state = keyboard.as_controller().button(button);
            assert!(!state.ended_down());
            assert_eq!(state.half_transition_count(), 0);
        }
    }

    #[test]
    fn test_modifiers_are_tracked_by_side_and_queried_together() {
        let mapping = KeyMapping::default();
        let mut keyboard = KeyboardState::default();
        assert!(!keyboard.is_control_down());

        keyboard.track_key(&mapping, Key::RightControl, true);
        assert!(keyboard.is_control_down());
        assert!(!keyboard.is_key_down(Key::LeftControl));
        assert!(!keyboard.is_shift_down());
        assert!(!keyboard.is_alt_down());

        keyboard.track_key(&mapping, Key::LeftShift, true);
        keyboard.track_key(&mapping, Key::LeftAlt, true);
        assert!(keyboard.is_shift_down());
        assert!(keyboard.is_alt_down());
    }

    #[test]
    fn test_reset_counts_keeps_held_keys_held() {
        let mapping = KeyMapping::default();
        let mut keyboard = KeyboardState::default();
        keyboard.track_key(&mapping, Key::W, true);
        keyboard.reset_counts();

        assert!(keyboard.is_key_down(Key::W));
        assert_eq!(keyboard.key(Key::W).half_transition_count(), 0);
        assert_eq!(up_button(&keyboard), (true, 0));
    }

    #[test]
    fn test_release_all_lowers_keys_and_buttons_with_a_transition() {
        let mapping = KeyMapping::default();
        let mut keyboard = KeyboardState::default();
        keyboard.track_key(&mapping, Key::W, true);
        keyboard.track_key(&mapping, Key::LeftControl, true);
        keyboard.reset_counts();

        keyboard.release_all();

        assert!(!keyboard.is_key_down(Key::W));
        assert!(!keyboard.is_control_down());
        assert_eq!(keyboard.key(Key::W).half_transition_count(), 1);
        assert_eq!(keyboard.key(Key::LeftControl).half_transition_count(), 1);
        assert_eq!(up_button(&keyboard), (false, 1));
        // Keys that were already up are untouched.
        assert_eq!(keyboard.key(Key::S).half_transition_count(), 0);
        let down = keyboard.as_controller().button(Button::Down);
        assert_eq!(down.half_transition_count(), 0);
    }

    #[test]
    fn test_synchronize_applies_a_snapshot() {
        let mapping = KeyMapping::default();
        let mut keyboard = KeyboardState::default();
        keyboard.track_key(&mapping, Key::W, true);
        keyboard.reset_counts();

        // While the window was unfocused the user let go of W and pressed D and Right Control.
        keyboard.synchronize(
            &mapping,
            [
                (Key::W, false),
                (Key::D, true),
                (Key::RightControl, true),
                (Key::S, false),
            ],
        );

        assert_eq!(up_button(&keyboard), (false, 1));
        let right = keyboard.as_controller().button(Button::Right);
        assert_eq!(
            (right.ended_down(), right.half_transition_count()),
            (true, 1)
        );
        assert!(keyboard.is_control_down());
        assert_eq!(keyboard.key(Key::S).half_transition_count(), 0);
    }

    #[test]
    fn test_custom_mapping_is_honored() {
        let mut mapping = KeyMapping::empty();
        mapping.bind(Key::Space, Button::A).unwrap();
        let mut keyboard = KeyboardState::new();

        keyboard.track_key(&mapping, Key::Space, true);
        assert!(keyboard.as_controller().button(Button::A).ended_down());
        // W means nothing under this mapping.
        keyboard.track_key(&mapping, Key::W, true);
        assert!(!keyboard.as_controller().button(Button::Up).ended_down());
    }

    #[test]
    fn test_keyboard_controller_is_always_enabled() {
        let keyboard = KeyboardState::default();
        assert!(keyboard.as_controller().enabled());
    }
}
