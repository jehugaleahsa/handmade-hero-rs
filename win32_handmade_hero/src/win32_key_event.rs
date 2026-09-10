use handmade_hero_interface::key::Key;
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyboardState, MAPVK_VSC_TO_VK_EX, MapVirtualKeyW, VIRTUAL_KEY, VK_0, VK_9, VK_A, VK_ADD,
    VK_APPS, VK_BACK, VK_CAPITAL, VK_CONTROL, VK_DECIMAL, VK_DELETE, VK_DIVIDE, VK_DOWN, VK_END,
    VK_ESCAPE, VK_F1, VK_F12, VK_HOME, VK_INSERT, VK_LCONTROL, VK_LEFT, VK_LMENU, VK_LSHIFT,
    VK_LWIN, VK_MENU, VK_MULTIPLY, VK_NEXT, VK_NUMLOCK, VK_NUMPAD0, VK_NUMPAD9, VK_OEM_1, VK_OEM_2,
    VK_OEM_3, VK_OEM_4, VK_OEM_5, VK_OEM_6, VK_OEM_7, VK_OEM_COMMA, VK_OEM_MINUS, VK_OEM_PERIOD,
    VK_OEM_PLUS, VK_PAUSE, VK_PRIOR, VK_RCONTROL, VK_RETURN, VK_RIGHT, VK_RMENU, VK_RSHIFT,
    VK_RWIN, VK_SCROLL, VK_SHIFT, VK_SNAPSHOT, VK_SPACE, VK_SUBTRACT, VK_TAB, VK_UP, VK_Z,
};
use windows::core::Result as Win32Result;

/// The parameters of a `WM_KEYDOWN`, `WM_KEYUP`, `WM_SYSKEYDOWN`, or `WM_SYSKEYUP` message.
///
/// `WPARAM` holds the virtual-key code. `LPARAM` packs several fields; the ones that matter here:
///
/// | Bits  | Meaning                                                                          |
/// |-------|----------------------------------------------------------------------------------|
/// | 16-23 | Scan code: the physical key, independent of layout                              |
/// | 24    | Extended key: right Control and right Alt, and the arrow keys as opposed to the numpad |
/// | 30    | Previous state: the key was already down before this message                     |
/// | 31    | Transition state: set on release, clear on press                                 |
///
/// Windows reports the modifier keys generically as `VK_SHIFT`, `VK_CONTROL`, and `VK_MENU`. The
/// scan code and the extended bit are what tell the left key from the right one.
#[derive(Debug, Copy, Clone)]
pub struct Win32KeyEvent {
    w_param: WPARAM,
    l_param: LPARAM,
}

impl Win32KeyEvent {
    #[inline]
    #[must_use]
    pub fn from_params(w_param: WPARAM, l_param: LPARAM) -> Self {
        Self { w_param, l_param }
    }

    #[inline]
    #[must_use]
    pub fn virtual_key(&self) -> VIRTUAL_KEY {
        #[expect(clippy::cast_possible_truncation)]
        VIRTUAL_KEY(self.w_param.0 as u16)
    }

    #[inline]
    #[must_use]
    pub fn scan_code(&self) -> u32 {
        let scan_code_mask = 0xFF << 16;
        #[expect(clippy::cast_possible_truncation)]
        #[expect(clippy::cast_sign_loss)]
        let scan_code = ((self.l_param.0 & scan_code_mask) >> 16) as u32;
        scan_code
    }

    #[inline]
    #[must_use]
    pub fn is_extended(&self) -> bool {
        let extended_key_mask = 1 << 24;
        (self.l_param.0 & extended_key_mask) != 0
    }

    #[inline]
    #[must_use]
    pub fn was_down(&self) -> bool {
        let previous_key_state_mask = 1 << 30;
        (self.l_param.0 & previous_key_state_mask) != 0
    }

    #[inline]
    #[must_use]
    pub fn is_down(&self) -> bool {
        let transition_state_mask = 1 << 31;
        (self.l_param.0 & transition_state_mask) == 0
    }

    /// Whether this message is a held key autorepeating rather than a change of state.
    ///
    /// A press has `was_down` clear and `is_down` set. A release has both set. Only an autorepeat
    /// has them agreeing on "down".
    #[inline]
    #[must_use]
    pub fn is_repeat(&self) -> bool {
        self.was_down() == self.is_down()
    }

    /// The platform-agnostic key this message is about, or `None` for keys the game ignores.
    #[must_use]
    pub fn key(&self) -> Option<Key> {
        let virtual_key = self.virtual_key();
        let key = match virtual_key {
            VK_SHIFT => {
                // Shift is the one modifier whose LPARAM carries no side information, so ask
                // Windows which specific key owns this scan code.
                let specific = unsafe { MapVirtualKeyW(self.scan_code(), MAPVK_VSC_TO_VK_EX) };
                if specific == u32::from(VK_RSHIFT.0) {
                    Key::RightShift
                } else {
                    Key::LeftShift
                }
            }
            VK_CONTROL => {
                if self.is_extended() {
                    Key::RightControl
                } else {
                    Key::LeftControl
                }
            }
            VK_MENU => {
                if self.is_extended() {
                    Key::RightAlt
                } else {
                    Key::LeftAlt
                }
            }
            VK_RETURN => {
                if self.is_extended() {
                    Key::NumpadEnter
                } else {
                    Key::Enter
                }
            }
            _ => return key_from_virtual_key(virtual_key),
        };
        Some(key)
    }
}

/// Snapshots which keys are physically down right now, for reconciling after a focus change.
///
/// Windows copies the asynchronous key state into this thread's synchronous state when focus
/// arrives, so at `WM_SETFOCUS` time this reflects the real keyboard, including keys pressed
/// while another window had focus. Unlike a key message, the snapshot reports left and right
/// modifiers as distinct virtual keys, so no scan-code lookup is needed.
///
/// # Errors
///
/// Fails if Windows cannot supply the keyboard state.
pub fn physical_key_states() -> Win32Result<impl Iterator<Item = (Key, bool)>> {
    let mut states = [0u8; 256];
    unsafe { GetKeyboardState(&mut states)? };
    let is_down_mask = 0x80; // The key is down if the high-order bit is set.
    let key_states = states
        .into_iter()
        .enumerate()
        .filter_map(move |(code, state)| {
            // The array has exactly 256 entries, so the index always fits a virtual-key code.
            let code = u16::try_from(code).ok()?;
            let key = key_from_virtual_key(VIRTUAL_KEY(code))?;
            Some((key, (state & is_down_mask) != 0))
        });
    Ok(key_states)
}

/// Translates a specific virtual-key code into a [`Key`].
///
/// The generic modifier codes map to `None` because they carry no side. A key message resolves
/// them with the scan code and extended bit first, and a keyboard snapshot reports the specific
/// left and right codes directly.
#[must_use]
pub fn key_from_virtual_key(virtual_key: VIRTUAL_KEY) -> Option<Key> {
    if let Some(key) = key_from_virtual_key_range(virtual_key) {
        return Some(key);
    }
    let key = match virtual_key {
        VK_UP => Key::Up,
        VK_DOWN => Key::Down,
        VK_LEFT => Key::Left,
        VK_RIGHT => Key::Right,

        VK_ESCAPE => Key::Escape,
        VK_SPACE => Key::Space,
        VK_RETURN => Key::Enter,
        VK_TAB => Key::Tab,
        VK_BACK => Key::Backspace,
        VK_INSERT => Key::Insert,
        VK_DELETE => Key::Delete,
        VK_HOME => Key::Home,
        VK_END => Key::End,
        VK_PRIOR => Key::PageUp,
        VK_NEXT => Key::PageDown,

        VK_LSHIFT => Key::LeftShift,
        VK_RSHIFT => Key::RightShift,
        VK_LCONTROL => Key::LeftControl,
        VK_RCONTROL => Key::RightControl,
        VK_LMENU => Key::LeftAlt,
        VK_RMENU => Key::RightAlt,
        VK_LWIN => Key::LeftSuper,
        VK_RWIN => Key::RightSuper,
        VK_APPS => Key::Menu,
        VK_CAPITAL => Key::CapsLock,

        // The OEM codes are named for their US-layout labels.
        VK_OEM_MINUS => Key::Minus,
        VK_OEM_PLUS => Key::Equal,
        VK_OEM_4 => Key::LeftBracket,
        VK_OEM_6 => Key::RightBracket,
        VK_OEM_5 => Key::Backslash,
        VK_OEM_1 => Key::Semicolon,
        VK_OEM_7 => Key::Apostrophe,
        VK_OEM_3 => Key::Grave,
        VK_OEM_COMMA => Key::Comma,
        VK_OEM_PERIOD => Key::Period,
        VK_OEM_2 => Key::Slash,

        VK_ADD => Key::NumpadAdd,
        VK_SUBTRACT => Key::NumpadSubtract,
        VK_MULTIPLY => Key::NumpadMultiply,
        VK_DIVIDE => Key::NumpadDivide,
        VK_DECIMAL => Key::NumpadDecimal,
        VK_NUMLOCK => Key::NumLock,

        VK_SNAPSHOT => Key::PrintScreen,
        VK_SCROLL => Key::ScrollLock,
        VK_PAUSE => Key::Pause,
        _ => return None,
    };
    Some(key)
}

/// Handles the virtual-key codes Windows assigns in contiguous runs, where arithmetic beats a
/// table: letters, digits, function keys, and numpad digits.
fn key_from_virtual_key_range(virtual_key: VIRTUAL_KEY) -> Option<Key> {
    let code = virtual_key.0;
    let ranges: [(VIRTUAL_KEY, VIRTUAL_KEY, &[Key]); 4] = [
        (VK_A, VK_Z, &Key::LETTERS),
        (VK_0, VK_9, &Key::DIGITS),
        (VK_F1, VK_F12, &Key::FUNCTION_KEYS),
        (VK_NUMPAD0, VK_NUMPAD9, &Key::NUMPAD_DIGITS),
    ];
    ranges
        .into_iter()
        .find(|(first, last, _)| (first.0..=last.0).contains(&code))
        .and_then(|(first, _, keys)| keys.get(usize::from(code - first.0)).copied())
}

#[cfg(test)]
mod tests {
    use crate::win32_key_event::{Win32KeyEvent, key_from_virtual_key};
    use handmade_hero_interface::key::Key;
    use windows::Win32::Foundation::{LPARAM, WPARAM};
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        VIRTUAL_KEY, VK_5, VK_A, VK_CONTROL, VK_F10, VK_MENU, VK_NUMPAD7, VK_OEM_3, VK_RETURN,
        VK_SHIFT, VK_W, VK_Z,
    };

    const EXTENDED: isize = 1 << 24;
    const WAS_DOWN: isize = 1 << 30;
    const RELEASE: isize = 1 << 31;

    fn event(virtual_key: VIRTUAL_KEY, l_param: isize) -> Win32KeyEvent {
        Win32KeyEvent::from_params(WPARAM(virtual_key.0 as usize), LPARAM(l_param))
    }

    #[test]
    fn test_press_release_and_repeat_flags() {
        assert!(event(VK_W, 0).is_down());
        assert!(!event(VK_W, 0).is_repeat());

        assert!(!event(VK_W, WAS_DOWN | RELEASE).is_down());
        assert!(!event(VK_W, WAS_DOWN | RELEASE).is_repeat());

        assert!(event(VK_W, WAS_DOWN).is_down());
        assert!(event(VK_W, WAS_DOWN).is_repeat());
    }

    #[test]
    fn test_scan_code_and_extended_bit_are_decoded() {
        let key_event = event(VK_CONTROL, (0x1D << 16) | EXTENDED);
        assert_eq!(key_event.scan_code(), 0x1D);
        assert!(key_event.is_extended());
        assert!(!event(VK_CONTROL, 0x1D << 16).is_extended());
    }

    #[test]
    fn test_contiguous_ranges_translate_by_arithmetic() {
        assert_eq!(key_from_virtual_key(VK_A), Some(Key::A));
        assert_eq!(key_from_virtual_key(VK_Z), Some(Key::Z));
        assert_eq!(key_from_virtual_key(VK_5), Some(Key::Digit5));
        assert_eq!(key_from_virtual_key(VK_F10), Some(Key::F10));
        assert_eq!(key_from_virtual_key(VK_NUMPAD7), Some(Key::Numpad7));
        assert_eq!(key_from_virtual_key(VK_OEM_3), Some(Key::Grave));
    }

    #[test]
    fn test_generic_modifiers_have_no_side_without_a_message() {
        assert_eq!(key_from_virtual_key(VK_SHIFT), None);
        assert_eq!(key_from_virtual_key(VK_CONTROL), None);
        assert_eq!(key_from_virtual_key(VK_MENU), None);
        assert_eq!(key_from_virtual_key(VIRTUAL_KEY(0xFF)), None);
    }

    #[test]
    fn test_extended_bit_picks_the_right_hand_modifier() {
        assert_eq!(event(VK_CONTROL, 0).key(), Some(Key::LeftControl));
        assert_eq!(event(VK_CONTROL, EXTENDED).key(), Some(Key::RightControl));
        assert_eq!(event(VK_MENU, 0).key(), Some(Key::LeftAlt));
        assert_eq!(event(VK_MENU, EXTENDED).key(), Some(Key::RightAlt));
        assert_eq!(event(VK_RETURN, 0).key(), Some(Key::Enter));
        assert_eq!(event(VK_RETURN, EXTENDED).key(), Some(Key::NumpadEnter));
    }

    #[test]
    fn test_shift_side_comes_from_the_scan_code() {
        // 0x2A and 0x36 are the fixed PC scan codes for the left and right Shift keys.
        assert_eq!(event(VK_SHIFT, 0x2A << 16).key(), Some(Key::LeftShift));
        assert_eq!(event(VK_SHIFT, 0x36 << 16).key(), Some(Key::RightShift));
    }
}
