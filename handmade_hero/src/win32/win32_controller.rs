use windows::Win32::{
    Foundation::ERROR_SUCCESS,
    UI::Input::XboxController::{
        XINPUT_GAMEPAD, XINPUT_GAMEPAD_A, XINPUT_GAMEPAD_B, XINPUT_GAMEPAD_BACK,
        XINPUT_GAMEPAD_BUTTON_FLAGS, XINPUT_GAMEPAD_DPAD_DOWN, XINPUT_GAMEPAD_DPAD_LEFT,
        XINPUT_GAMEPAD_DPAD_RIGHT, XINPUT_GAMEPAD_DPAD_UP, XINPUT_GAMEPAD_LEFT_SHOULDER,
        XINPUT_GAMEPAD_LEFT_THUMB_DEADZONE, XINPUT_GAMEPAD_RIGHT_SHOULDER,
        XINPUT_GAMEPAD_RIGHT_THUMB_DEADZONE, XINPUT_GAMEPAD_START,
        XINPUT_GAMEPAD_TRIGGER_THRESHOLD, XINPUT_GAMEPAD_X, XINPUT_GAMEPAD_Y, XINPUT_STATE,
        XInputGetState, XUSER_MAX_COUNT,
    },
};

#[derive(Debug)]
pub enum Win32ControllerState {
    Enabled(Win32Controller),
    Disabled,
}

#[derive(Debug)]
pub struct Win32Controller {
    gamepad: XINPUT_GAMEPAD,
}

impl Win32Controller {
    #[inline]
    #[must_use]
    pub fn max_controller_count() -> usize {
        usize::try_from(XUSER_MAX_COUNT).unwrap_or(0)
    }

    #[must_use]
    pub fn from_index(index: usize) -> Win32ControllerState {
        let Ok(index_u32) = u32::try_from(index) else {
            return Win32ControllerState::Disabled;
        };
        let mut controller_state = XINPUT_STATE::default();
        let result = unsafe { XInputGetState(index_u32, &raw mut controller_state) };
        if result != ERROR_SUCCESS.0 {
            return Win32ControllerState::Disabled;
        }
        let XINPUT_STATE {
            Gamepad: gamepad, ..
        } = controller_state;
        let controller = Win32Controller { gamepad };
        Win32ControllerState::Enabled(controller)
    }

    #[inline]
    #[must_use]
    pub fn is_a(&self) -> bool {
        self.is_pressed(XINPUT_GAMEPAD_A)
    }

    #[inline]
    #[must_use]
    pub fn is_b(&self) -> bool {
        self.is_pressed(XINPUT_GAMEPAD_B)
    }

    #[inline]
    #[must_use]
    pub fn is_x(&self) -> bool {
        self.is_pressed(XINPUT_GAMEPAD_X)
    }

    #[inline]
    #[must_use]
    pub fn is_y(&self) -> bool {
        self.is_pressed(XINPUT_GAMEPAD_Y)
    }

    #[inline]
    #[must_use]
    pub fn is_start(&self) -> bool {
        self.is_pressed(XINPUT_GAMEPAD_START)
    }

    #[inline]
    #[must_use]
    pub fn is_back(&self) -> bool {
        self.is_pressed(XINPUT_GAMEPAD_BACK)
    }

    #[inline]
    #[must_use]
    pub fn is_dpad_up(&self) -> bool {
        self.is_pressed(XINPUT_GAMEPAD_DPAD_UP)
    }

    #[inline]
    #[must_use]
    pub fn is_dpad_down(&self) -> bool {
        self.is_pressed(XINPUT_GAMEPAD_DPAD_DOWN)
    }

    #[inline]
    #[must_use]
    pub fn is_dpad_left(&self) -> bool {
        self.is_pressed(XINPUT_GAMEPAD_DPAD_LEFT)
    }

    #[inline]
    #[must_use]
    pub fn is_dpad_right(&self) -> bool {
        self.is_pressed(XINPUT_GAMEPAD_DPAD_RIGHT)
    }

    #[inline]
    #[must_use]
    pub fn is_left_shoulder(&self) -> bool {
        self.is_pressed(XINPUT_GAMEPAD_LEFT_SHOULDER)
    }

    #[inline]
    #[must_use]
    pub fn is_right_shoulder(&self) -> bool {
        self.is_pressed(XINPUT_GAMEPAD_RIGHT_SHOULDER)
    }

    #[inline]
    #[must_use]
    fn is_pressed(&self, button: XINPUT_GAMEPAD_BUTTON_FLAGS) -> bool {
        (self.gamepad.wButtons & button).0 != 0
    }

    #[inline]
    #[must_use]
    pub fn left_joystick_x(&self) -> f32 {
        Self::thumb_stick_ratio(self.gamepad.sThumbLX, XINPUT_GAMEPAD_LEFT_THUMB_DEADZONE.0)
    }

    #[inline]
    #[must_use]
    pub fn left_joystick_y(&self) -> f32 {
        -Self::thumb_stick_ratio(self.gamepad.sThumbLY, XINPUT_GAMEPAD_LEFT_THUMB_DEADZONE.0)
    }

    #[inline]
    #[must_use]
    pub fn right_joystick_x(&self) -> f32 {
        Self::thumb_stick_ratio(self.gamepad.sThumbRX, XINPUT_GAMEPAD_RIGHT_THUMB_DEADZONE.0)
    }

    #[inline]
    #[must_use]
    pub fn right_joystick_y(&self) -> f32 {
        -Self::thumb_stick_ratio(self.gamepad.sThumbRY, XINPUT_GAMEPAD_RIGHT_THUMB_DEADZONE.0)
    }

    #[inline]
    #[must_use]
    fn thumb_stick_ratio(amount: i16, dead_zone: u16) -> f32 {
        if amount.unsigned_abs() <= dead_zone {
            0f32
        } else if amount < 0 {
            let dead_zone = f32::from(dead_zone);
            -((f32::from(amount) + dead_zone) / (f32::from(i16::MIN) + dead_zone))
        } else {
            let dead_zone = f32::from(dead_zone);
            (f32::from(amount) - dead_zone) / (f32::from(i16::MAX) - dead_zone)
        }
    }

    #[inline]
    #[must_use]
    pub fn left_trigger(&self) -> f32 {
        Self::trigger_ratio(self.gamepad.bLeftTrigger)
    }

    #[inline]
    #[must_use]
    pub fn right_trigger(&self) -> f32 {
        Self::trigger_ratio(self.gamepad.bRightTrigger)
    }

    #[inline]
    #[must_use]
    fn trigger_ratio(amount: u8) -> f32 {
        if u16::from(amount) <= XINPUT_GAMEPAD_TRIGGER_THRESHOLD.0 {
            0f32
        } else {
            let threshold = f32::from(XINPUT_GAMEPAD_TRIGGER_THRESHOLD.0);
            (f32::from(amount) - threshold) / (f32::from(u8::MAX) - threshold)
        }
    }
}
