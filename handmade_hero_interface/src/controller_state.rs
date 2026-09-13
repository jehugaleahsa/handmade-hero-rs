use crate::button::Button;
use crate::button_state::ButtonState;
use crate::joystick_state::JoystickState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ControllerState {
    buttons: [ButtonState; Button::COUNT],
    left_joystick: JoystickState,
    right_joystick: JoystickState,
    left_trigger_ratio: f32,
    right_trigger_ratio: f32,
    enabled: bool,
}

impl ControllerState {
    #[inline]
    #[must_use]
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    #[inline]
    pub fn set_enabled(&mut self, value: bool) {
        self.enabled = value;
    }

    #[inline]
    #[must_use]
    pub fn a(&self) -> &ButtonState {
        self.button(Button::A)
    }

    #[inline]
    #[must_use]
    pub fn a_mut(&mut self) -> &mut ButtonState {
        self.button_mut(Button::A)
    }

    #[inline]
    #[must_use]
    pub fn b(&self) -> &ButtonState {
        self.button(Button::B)
    }

    #[inline]
    #[must_use]
    pub fn b_mut(&mut self) -> &mut ButtonState {
        self.button_mut(Button::B)
    }

    #[inline]
    #[must_use]
    pub fn x(&self) -> &ButtonState {
        self.button(Button::X)
    }

    #[inline]
    #[must_use]
    pub fn x_mut(&mut self) -> &mut ButtonState {
        self.button_mut(Button::X)
    }

    #[inline]
    #[must_use]
    pub fn y(&self) -> &ButtonState {
        self.button(Button::Y)
    }

    #[inline]
    #[must_use]
    pub fn y_mut(&mut self) -> &mut ButtonState {
        self.button_mut(Button::Y)
    }

    #[inline]
    #[must_use]
    pub fn start(&self) -> &ButtonState {
        self.button(Button::Start)
    }

    #[inline]
    #[must_use]
    pub fn start_mut(&mut self) -> &mut ButtonState {
        self.button_mut(Button::Start)
    }

    #[inline]
    #[must_use]
    pub fn back(&self) -> &ButtonState {
        self.button(Button::Back)
    }

    #[inline]
    #[must_use]
    pub fn back_mut(&mut self) -> &mut ButtonState {
        self.button_mut(Button::Back)
    }

    #[inline]
    #[must_use]
    pub fn up(&self) -> &ButtonState {
        self.button(Button::Up)
    }

    #[inline]
    #[must_use]
    pub fn up_mut(&mut self) -> &mut ButtonState {
        self.button_mut(Button::Up)
    }

    #[inline]
    #[must_use]
    pub fn down(&self) -> &ButtonState {
        self.button(Button::Down)
    }

    #[inline]
    #[must_use]
    pub fn down_mut(&mut self) -> &mut ButtonState {
        self.button_mut(Button::Down)
    }

    #[inline]
    #[must_use]
    pub fn left(&self) -> &ButtonState {
        self.button(Button::Left)
    }

    #[inline]
    #[must_use]
    pub fn left_mut(&mut self) -> &mut ButtonState {
        self.button_mut(Button::Left)
    }

    #[inline]
    #[must_use]
    pub fn right(&self) -> &ButtonState {
        self.button(Button::Right)
    }

    #[inline]
    #[must_use]
    pub fn right_mut(&mut self) -> &mut ButtonState {
        self.button_mut(Button::Right)
    }

    #[inline]
    #[must_use]
    pub fn left_shoulder(&self) -> &ButtonState {
        self.button(Button::LeftShoulder)
    }

    #[inline]
    #[must_use]
    pub fn left_shoulder_mut(&mut self) -> &mut ButtonState {
        self.button_mut(Button::LeftShoulder)
    }

    #[inline]
    #[must_use]
    pub fn right_shoulder(&self) -> &ButtonState {
        self.button(Button::RightShoulder)
    }

    #[inline]
    #[must_use]
    pub fn right_shoulder_mut(&mut self) -> &mut ButtonState {
        self.button_mut(Button::RightShoulder)
    }

    #[inline]
    #[must_use]
    pub fn left_trigger_ratio(&self) -> f32 {
        self.left_trigger_ratio
    }

    #[inline]
    pub fn set_left_trigger_ratio(&mut self, value: f32) {
        self.left_trigger_ratio = value;
    }

    #[inline]
    #[must_use]
    pub fn right_trigger_ratio(&self) -> f32 {
        self.right_trigger_ratio
    }

    #[inline]
    pub fn set_right_trigger_ratio(&mut self, value: f32) {
        self.right_trigger_ratio = value;
    }

    #[inline]
    #[must_use]
    pub fn left_joystick(&self) -> &JoystickState {
        &self.left_joystick
    }

    #[inline]
    #[must_use]
    pub fn left_joystick_mut(&mut self) -> &mut JoystickState {
        &mut self.left_joystick
    }

    #[inline]
    #[must_use]
    pub fn right_joystick(&self) -> &JoystickState {
        &self.right_joystick
    }

    #[inline]
    #[must_use]
    pub fn right_joystick_mut(&mut self) -> &mut JoystickState {
        &mut self.right_joystick
    }

    /// Looks a button up by name, so callers can be written once for every button.
    #[inline]
    #[must_use]
    pub fn button(&self, button: Button) -> &ButtonState {
        let button_index = button.index();
        &self.buttons[button_index]
    }

    /// Looks a button up by name, so callers can be written once for every button.
    #[inline]
    #[must_use]
    pub fn button_mut(&mut self, button: Button) -> &mut ButtonState {
        let button_index = button.index();
        &mut self.buttons[button_index]
    }

    pub fn reset_counts(&mut self) {
        for button in &mut self.buttons {
            button.reset_half_transition_count();
        }
    }

    pub fn clear(&mut self) {
        for button in &mut self.buttons {
            button.clear();
        }
        self.left_trigger_ratio = 0.0;
        self.right_trigger_ratio = 0.0;
        self.left_joystick.clear();
        self.right_joystick.clear();
    }
}

#[cfg(test)]
mod tests {
    use crate::button::Button;
    use crate::controller_state::ControllerState;

    #[test]
    fn test_button_bindings() {
        let controller = ControllerState::default();
        assert!(core::ptr::eq(controller.a(), controller.button(Button::A)));
        assert!(core::ptr::eq(controller.b(), controller.button(Button::B)));
        assert!(core::ptr::eq(controller.x(), controller.button(Button::X)));
        assert!(core::ptr::eq(controller.y(), controller.button(Button::Y)));
        assert!(core::ptr::eq(
            controller.start(),
            controller.button(Button::Start)
        ));
        assert!(core::ptr::eq(
            controller.back(),
            controller.button(Button::Back)
        ));
        assert!(core::ptr::eq(
            controller.up(),
            controller.button(Button::Up)
        ));
        assert!(core::ptr::eq(
            controller.down(),
            controller.button(Button::Down)
        ));
        assert!(core::ptr::eq(
            controller.left(),
            controller.button(Button::Left)
        ));
        assert!(core::ptr::eq(
            controller.right(),
            controller.button(Button::Right)
        ));
        assert!(core::ptr::eq(
            controller.left_shoulder(),
            controller.button(Button::LeftShoulder)
        ));
        assert!(core::ptr::eq(
            controller.right_shoulder(),
            controller.button(Button::RightShoulder)
        ));
    }

    #[test]
    fn test_button_bindings_mut() {
        let mut controller = ControllerState::default();
        assert!(core::ptr::eq(
            controller.a_mut(),
            controller.button_mut(Button::A)
        ));
        assert!(core::ptr::eq(
            controller.b_mut(),
            controller.button_mut(Button::B)
        ));
        assert!(core::ptr::eq(
            controller.x_mut(),
            controller.button_mut(Button::X)
        ));
        assert!(core::ptr::eq(
            controller.y_mut(),
            controller.button_mut(Button::Y)
        ));
        assert!(core::ptr::eq(
            controller.start_mut(),
            controller.button_mut(Button::Start)
        ));
        assert!(core::ptr::eq(
            controller.back_mut(),
            controller.button_mut(Button::Back)
        ));
        assert!(core::ptr::eq(
            controller.up_mut(),
            controller.button_mut(Button::Up)
        ));
        assert!(core::ptr::eq(
            controller.down_mut(),
            controller.button_mut(Button::Down)
        ));
        assert!(core::ptr::eq(
            controller.left_mut(),
            controller.button_mut(Button::Left)
        ));
        assert!(core::ptr::eq(
            controller.right_mut(),
            controller.button_mut(Button::Right)
        ));
        assert!(core::ptr::eq(
            controller.left_shoulder_mut(),
            controller.button_mut(Button::LeftShoulder)
        ));
        assert!(core::ptr::eq(
            controller.right_shoulder_mut(),
            controller.button_mut(Button::RightShoulder)
        ));
    }
}
