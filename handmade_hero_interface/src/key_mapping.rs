use smallvec::SmallVec;

use crate::button::Button;
use crate::key::Key;
use crate::key_mapping_error::KeyMappingError;

/// A realistic maximum number of keys mapped to a single button, with headroom.
const KEYS_PER_BUTTON_WATERMARK: usize = 4;
type KeysForButtonVec = SmallVec<[Key; KEYS_PER_BUTTON_WATERMARK]>;

/// A many-to-one binding from keys to controller buttons.
///
/// A key drives at most one button. A button may be driven by several keys, which is what lets
/// W and Up Arrow both mean "up". Both directions are stored so each lookup is a single array
/// index: the platform asks which button a key drives when a key event arrives, and the
/// aggregation step asks which keys drive that button so it can OR their states together.
#[derive(Debug, Clone)]
pub struct KeyMapping {
    button_for_key: [Option<Button>; Key::COUNT],
    keys_for_button: [KeysForButtonVec; Button::COUNT],
}

impl KeyMapping {
    /// A mapping with no bindings at all.
    /// NOTE: In the future, we should be able to make this const, but `core::array::from_fn`
    /// can't be called from a const context yet.
    #[inline]
    #[must_use]
    pub fn empty() -> Self {
        const BUTTON_COUNT: usize = Button::COUNT;
        Self {
            button_for_key: [None; Key::COUNT],
            keys_for_button: core::array::from_fn::<_, BUTTON_COUNT, _>(|_| {
                KeysForButtonVec::new()
            }),
        }
    }

    /// Makes `key` drive `button`.
    ///
    /// # Errors
    ///
    /// Fails if the key already drives a button. The mapping is unchanged on failure.
    pub fn bind(&mut self, key: Key, button: Button) -> Result<(), KeyMappingError> {
        if let Some(button) = self.button_for_key[key.index()] {
            return Err(KeyMappingError::KeyAlreadyBound { key, button });
        }
        let slots = &mut self.keys_for_button[button.index()];
        slots.push(key);
        self.button_for_key[key.index()] = Some(button);
        Ok(())
    }

    /// Removes whatever binding `key` has. Unbound keys are ignored.
    pub fn unbind(&mut self, key: Key) {
        let Some(button) = self.button_for_key[key.index()].take() else {
            return;
        };
        let keys = &mut self.keys_for_button[button.index()];
        keys.retain(|x| *x != key);
    }

    /// The button `key` drives, if any.
    #[inline]
    #[must_use]
    pub fn button_for(&self, key: Key) -> Option<Button> {
        self.button_for_key[key.index()]
    }

    /// Every key that drives `button`.
    #[inline]
    pub fn keys_for(&self, button: Button) -> impl Iterator<Item = Key> {
        self.keys_for_button[button.index()].iter().copied()
    }
}

impl Default for KeyMapping {
    /// The bindings the game ships with: WASD and the arrow keys for the D-pad, Q and E for the
    /// shoulders, and Escape for Start.
    fn default() -> Self {
        default_mapping()
    }
}

fn default_mapping() -> KeyMapping {
    const BINDINGS: [(Key, Button); 11] = [
        (Key::W, Button::Up),
        (Key::Up, Button::Up),
        (Key::A, Button::Left),
        (Key::Left, Button::Left),
        (Key::S, Button::Down),
        (Key::Down, Button::Down),
        (Key::D, Button::Right),
        (Key::Right, Button::Right),
        (Key::Q, Button::LeftShoulder),
        (Key::E, Button::RightShoulder),
        (Key::Escape, Button::Start),
    ];

    let mut mapping = KeyMapping::empty();
    for (key, button) in BINDINGS {
        // The table above is fixed, so a failure here is a programming error in the table itself.
        mapping
            .bind(key, button)
            .expect("The default key mapping must bind each key exactly once");
    }
    mapping
}

#[cfg(test)]
mod tests {
    use crate::button::Button;
    use crate::key::Key;
    use crate::key_mapping::KeyMapping;
    use crate::key_mapping_error::KeyMappingError;

    #[test]
    fn test_bind_is_visible_in_both_directions() {
        let mut mapping = KeyMapping::empty();
        mapping.bind(Key::W, Button::Up).unwrap();
        mapping.bind(Key::Up, Button::Up).unwrap();

        assert_eq!(mapping.button_for(Key::W), Some(Button::Up));
        assert_eq!(mapping.button_for(Key::Up), Some(Button::Up));
        assert_eq!(mapping.button_for(Key::S), None);
        let keys: Vec<Key> = mapping.keys_for(Button::Up).collect();
        assert_eq!(keys, vec![Key::W, Key::Up]);
        assert_eq!(mapping.keys_for(Button::Down).count(), 0);
    }

    #[test]
    fn test_key_cannot_drive_two_buttons() {
        let mut mapping = KeyMapping::empty();
        mapping.bind(Key::W, Button::Up).unwrap();
        let result = mapping.bind(Key::W, Button::Down);
        assert_eq!(
            result,
            Err(KeyMappingError::KeyAlreadyBound {
                key: Key::W,
                button: Button::Up
            })
        );
        assert_eq!(mapping.keys_for(Button::Down).count(), 0);
    }

    #[test]
    fn test_unbind_frees_the_slot() {
        let mut mapping = KeyMapping::empty();
        mapping.bind(Key::W, Button::Up).unwrap();
        mapping.bind(Key::Up, Button::Up).unwrap();
        mapping.unbind(Key::W);

        assert_eq!(mapping.button_for(Key::W), None);
        let keys: Vec<Key> = mapping.keys_for(Button::Up).collect();
        assert_eq!(keys, vec![Key::Up]);
        mapping.bind(Key::W, Button::Down).unwrap();
        assert_eq!(mapping.button_for(Key::W), Some(Button::Down));
    }

    #[test]
    fn test_default_mapping_matches_the_original_hard_coded_keys() {
        let mapping = KeyMapping::default();
        assert_eq!(mapping.button_for(Key::W), Some(Button::Up));
        assert_eq!(mapping.button_for(Key::Up), Some(Button::Up));
        assert_eq!(mapping.button_for(Key::A), Some(Button::Left));
        assert_eq!(mapping.button_for(Key::Left), Some(Button::Left));
        assert_eq!(mapping.button_for(Key::S), Some(Button::Down));
        assert_eq!(mapping.button_for(Key::Down), Some(Button::Down));
        assert_eq!(mapping.button_for(Key::D), Some(Button::Right));
        assert_eq!(mapping.button_for(Key::Right), Some(Button::Right));
        assert_eq!(mapping.button_for(Key::Q), Some(Button::LeftShoulder));
        assert_eq!(mapping.button_for(Key::E), Some(Button::RightShoulder));
        assert_eq!(mapping.button_for(Key::Escape), Some(Button::Start));
        assert_eq!(mapping.button_for(Key::L), None);
    }
}
