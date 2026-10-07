use crate::button_state::ButtonState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MouseState {
    x: i32,
    y: i32,
    vertical_wheel_delta: i32,
    horizontal_wheel_delta: i32,
    buttons: [ButtonState; MouseState::BUTTON_COUNT],
}

impl MouseState {
    const BUTTON_COUNT: usize = 5;
    const LEFT_BUTTON_INDEX: usize = 0;
    const MIDDLE_BUTTON_INDEX: usize = 1;
    const RIGHT_BUTTON_INDEX: usize = 2;
    const X1_BUTTON_INDEX: usize = 3;
    const X2_BUTTON_INDEX: usize = 4;

    #[inline]
    #[must_use]
    pub fn x(&self) -> i32 {
        self.x
    }

    #[inline]
    pub fn set_x(&mut self, value: i32) {
        self.x = value;
    }

    #[inline]
    #[must_use]
    pub fn y(&self) -> i32 {
        self.y
    }

    #[inline]
    pub fn set_y(&mut self, value: i32) {
        self.y = value;
    }

    #[inline]
    #[must_use]
    pub fn left(&self) -> &ButtonState {
        &self.buttons[Self::LEFT_BUTTON_INDEX]
    }

    #[inline]
    #[must_use]
    pub fn left_mut(&mut self) -> &mut ButtonState {
        &mut self.buttons[Self::LEFT_BUTTON_INDEX]
    }

    #[inline]
    #[must_use]
    pub fn middle(&self) -> &ButtonState {
        &self.buttons[Self::MIDDLE_BUTTON_INDEX]
    }

    #[inline]
    #[must_use]
    pub fn middle_mut(&mut self) -> &mut ButtonState {
        &mut self.buttons[Self::MIDDLE_BUTTON_INDEX]
    }

    #[inline]
    #[must_use]
    pub fn right(&self) -> &ButtonState {
        &self.buttons[Self::RIGHT_BUTTON_INDEX]
    }

    #[inline]
    #[must_use]
    pub fn right_mut(&mut self) -> &mut ButtonState {
        &mut self.buttons[Self::RIGHT_BUTTON_INDEX]
    }

    /// The first side button, which is usually "back".
    #[inline]
    #[must_use]
    pub fn x1(&self) -> &ButtonState {
        &self.buttons[Self::X1_BUTTON_INDEX]
    }

    #[inline]
    #[must_use]
    pub fn x1_mut(&mut self) -> &mut ButtonState {
        &mut self.buttons[Self::X1_BUTTON_INDEX]
    }

    /// The second side button, which is usually "forward".
    #[inline]
    #[must_use]
    pub fn x2(&self) -> &ButtonState {
        &self.buttons[Self::X2_BUTTON_INDEX]
    }

    #[inline]
    #[must_use]
    pub fn x2_mut(&mut self) -> &mut ButtonState {
        &mut self.buttons[Self::X2_BUTTON_INDEX]
    }

    #[inline]
    pub fn buttons(&self) -> impl Iterator<Item = &ButtonState> {
        self.buttons.iter()
    }

    #[inline]
    #[must_use]
    pub fn is_any_down(&self) -> bool {
        self.buttons.iter().any(|button| button.ended_down())
    }

    pub fn release_all(&mut self) {
        for button in &mut self.buttons {
            button.track_down(false);
        }
    }

    #[inline]
    #[must_use]
    pub fn vertical_wheel_delta(&self) -> i32 {
        self.vertical_wheel_delta
    }

    #[inline]
    pub fn set_vertical_wheel_delta(&mut self, value: i32) {
        self.vertical_wheel_delta = value;
    }

    #[inline]
    #[must_use]
    pub fn horizontal_wheel_delta(&self) -> i32 {
        self.horizontal_wheel_delta
    }

    #[inline]
    pub fn set_horizontal_wheel_delta(&mut self, value: i32) {
        self.horizontal_wheel_delta = value;
    }

    pub fn clear(&mut self) {
        for button in &mut self.buttons {
            button.reset_half_transition_count();
        }
        self.x = 0;
        self.y = 0;
        self.vertical_wheel_delta = 0;
        self.horizontal_wheel_delta = 0;
    }
}

#[cfg(test)]
mod tests {
    use crate::mouse_state::MouseState;

    #[test]
    fn test_no_button_down_by_default() {
        let mouse = MouseState::default();
        assert!(!mouse.is_any_down());
    }

    #[test]
    fn test_side_button_counts_as_down() {
        let mut mouse = MouseState::default();
        mouse.x2_mut().track_down(true);
        assert!(mouse.is_any_down());
    }

    #[test]
    fn test_release_all_releases_held_buttons() {
        let mut mouse = MouseState::default();
        mouse.left_mut().track_down(true);
        mouse.x1_mut().track_down(true);
        mouse.clear();
        mouse.release_all();
        assert!(!mouse.is_any_down());
        assert!(mouse.left().was_released());
        assert!(mouse.x1().was_released());
        assert!(!mouse.right().was_released());
    }
}
