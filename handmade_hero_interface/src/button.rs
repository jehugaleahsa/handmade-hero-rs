/// A digital button on a [`crate::controller_state::ControllerState`].
///
/// Naming the buttons lets a key mapping be data instead of a chain of `if` statements, and lets
/// code iterate every button without knowing the struct's fields.
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Button {
    A,
    B,
    X,
    Y,
    LeftShoulder,
    RightShoulder,
    Up,
    Down,
    Left,
    Right,
    Start,
    // Keep this the last variant; `COUNT` is derived from it.
    Back,
}

impl Button {
    /// The number of buttons, suitable for sizing an array indexed by [`Button::index`].
    pub const COUNT: usize = Button::Back as usize + 1;

    /// Every button, in index order.
    pub const ALL: [Button; Button::COUNT] = [
        Button::A,
        Button::B,
        Button::X,
        Button::Y,
        Button::LeftShoulder,
        Button::RightShoulder,
        Button::Up,
        Button::Down,
        Button::Left,
        Button::Right,
        Button::Start,
        Button::Back,
    ];

    /// The position of this button in an array of [`Button::COUNT`] elements.
    #[inline]
    #[must_use]
    pub const fn index(self) -> usize {
        self as usize
    }
}

#[cfg(test)]
mod tests {
    use crate::button::Button;

    #[test]
    fn test_all_is_in_index_order() {
        for (index, button) in Button::ALL.iter().enumerate() {
            assert_eq!(button.index(), index);
        }
    }
}
