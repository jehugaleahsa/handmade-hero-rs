use crate::joystick_transition::JoystickTransition;

#[derive(Debug, Default, Copy, Clone)]
pub struct JoystickState {
    x: JoystickTransition,
    y: JoystickTransition,
}

impl JoystickState {
    #[inline]
    #[must_use]
    pub fn x(&self) -> &JoystickTransition {
        &self.x
    }

    #[inline]
    #[must_use]
    pub fn x_mut(&mut self) -> &mut JoystickTransition {
        &mut self.x
    }

    #[inline]
    #[must_use]
    pub fn y(&self) -> &JoystickTransition {
        &self.y
    }

    #[inline]
    #[must_use]
    pub fn y_mut(&mut self) -> &mut JoystickTransition {
        &mut self.y
    }
}
