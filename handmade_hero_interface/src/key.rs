/// A physical key on a keyboard, named after its label on a US layout.
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Key {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,

    Digit0,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,

    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,

    Up,
    Down,
    Left,
    Right,

    Escape,
    Space,
    Enter,
    Tab,
    Backspace,
    Insert,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,

    LeftShift,
    RightShift,
    LeftControl,
    RightControl,
    LeftAlt,
    RightAlt,
    /// The Windows key on a PC keyboard; Command on a Mac.
    LeftSuper,
    RightSuper,
    /// The context-menu key found between the right Super and right Control keys.
    Menu,
    CapsLock,

    Minus,
    Equal,
    LeftBracket,
    RightBracket,
    Backslash,
    Semicolon,
    Apostrophe,
    /// The backtick / tilde key.
    Grave,
    Comma,
    Period,
    Slash,

    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    NumpadAdd,
    NumpadSubtract,
    NumpadMultiply,
    NumpadDivide,
    NumpadDecimal,
    NumpadEnter,
    NumLock,

    PrintScreen,
    ScrollLock,
    // Keep this the last variant; `COUNT` is derived from it.
    Pause,
}

impl Key {
    /// The number of keys, suitable for sizing an array indexed by [`Key::index`].
    pub const COUNT: usize = Key::Pause.index() + 1;

    /// The letter keys in alphabetical order, so `LETTERS[0]` is A.
    pub const LETTERS: [Key; 26] = [
        Key::A,
        Key::B,
        Key::C,
        Key::D,
        Key::E,
        Key::F,
        Key::G,
        Key::H,
        Key::I,
        Key::J,
        Key::K,
        Key::L,
        Key::M,
        Key::N,
        Key::O,
        Key::P,
        Key::Q,
        Key::R,
        Key::S,
        Key::T,
        Key::U,
        Key::V,
        Key::W,
        Key::X,
        Key::Y,
        Key::Z,
    ];

    /// The digit keys along the top row, so `DIGITS[0]` is the 0 key.
    pub const DIGITS: [Key; 10] = [
        Key::Digit0,
        Key::Digit1,
        Key::Digit2,
        Key::Digit3,
        Key::Digit4,
        Key::Digit5,
        Key::Digit6,
        Key::Digit7,
        Key::Digit8,
        Key::Digit9,
    ];

    /// The function keys in order, so `FUNCTION_KEYS[0]` is F1.
    pub const FUNCTION_KEYS: [Key; 12] = [
        Key::F1,
        Key::F2,
        Key::F3,
        Key::F4,
        Key::F5,
        Key::F6,
        Key::F7,
        Key::F8,
        Key::F9,
        Key::F10,
        Key::F11,
        Key::F12,
    ];

    /// The numpad digit keys, so `NUMPAD_DIGITS[0]` is numpad 0.
    pub const NUMPAD_DIGITS: [Key; 10] = [
        Key::Numpad0,
        Key::Numpad1,
        Key::Numpad2,
        Key::Numpad3,
        Key::Numpad4,
        Key::Numpad5,
        Key::Numpad6,
        Key::Numpad7,
        Key::Numpad8,
        Key::Numpad9,
    ];

    /// The position of this key in an array of [`Key::COUNT`] elements.
    #[inline]
    #[must_use]
    pub const fn index(self) -> usize {
        self as usize
    }

    #[inline]
    #[must_use]
    pub const fn is_shift(self) -> bool {
        matches!(self, Key::LeftShift | Key::RightShift)
    }

    #[inline]
    #[must_use]
    pub const fn is_control(self) -> bool {
        matches!(self, Key::LeftControl | Key::RightControl)
    }

    #[inline]
    #[must_use]
    pub const fn is_alt(self) -> bool {
        matches!(self, Key::LeftAlt | Key::RightAlt)
    }
}

#[cfg(test)]
mod tests {
    use crate::key::Key;

    #[test]
    fn test_count_covers_every_key() {
        assert_eq!(Key::A.index(), 0);
        assert_eq!(Key::Pause.index(), Key::COUNT - 1);
    }

    #[test]
    fn test_lookup_tables_are_ordered() {
        assert_eq!(Key::LETTERS[0], Key::A);
        assert_eq!(Key::LETTERS[25], Key::Z);
        assert_eq!(Key::DIGITS[9], Key::Digit9);
        assert_eq!(Key::FUNCTION_KEYS[11], Key::F12);
        assert_eq!(Key::NUMPAD_DIGITS[5], Key::Numpad5);
    }
}
