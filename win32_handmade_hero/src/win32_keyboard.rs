use windows::Win32::{
    Foundation::{LPARAM, WPARAM},
    UI::Input::KeyboardAndMouse::{
        GetKeyState, VIRTUAL_KEY, VK_A, VK_CONTROL, VK_D, VK_DOWN, VK_E, VK_ESCAPE, VK_F4, VK_L,
        VK_LEFT, VK_Q, VK_RIGHT, VK_S, VK_UP, VK_W,
    },
};

pub struct Win32Keyboard {
    w_param: WPARAM,
    l_param: LPARAM,
}

impl Win32Keyboard {
    #[inline]
    #[must_use]
    pub fn from_params(w_param: WPARAM, l_param: LPARAM) -> Self {
        Self { w_param, l_param }
    }

    #[inline]
    #[must_use]
    pub fn was_key_down(&self) -> bool {
        let previous_key_state_mask = 1 << 30;
        (self.l_param.0 & previous_key_state_mask) != 0
    }

    #[inline]
    #[must_use]
    pub fn is_key_down(&self) -> bool {
        let transition_state_mask = 1 << 31;
        (self.l_param.0 & transition_state_mask) == 0
    }

    #[inline]
    #[must_use]
    pub fn is_alt(&self) -> bool {
        let context_code_mask = 1 << 29;
        (self.l_param.0 & context_code_mask) != 0
    }

    #[inline]
    #[must_use]
    #[expect(clippy::unused_self)]
    pub fn is_control(&self) -> bool {
        Self::is_special_key_down(VK_CONTROL)
    }

    #[inline]
    #[must_use]
    fn is_special_key_down(key: VIRTUAL_KEY) -> bool {
        let key_state = unsafe { GetKeyState(i32::from(key.0)) };
        let is_down_mask = 1 << 15; // The key is down if the high-order bit is set.
        (key_state & is_down_mask) != 0
    }

    #[inline]
    #[must_use]
    pub fn is_f4(&self) -> bool {
        self.virtual_key() == VK_F4
    }

    #[inline]
    #[must_use]
    pub fn is_a(&self) -> bool {
        self.virtual_key() == VK_A
    }

    #[inline]
    #[must_use]
    pub fn is_d(&self) -> bool {
        self.virtual_key() == VK_D
    }

    #[inline]
    #[must_use]
    pub fn is_e(&self) -> bool {
        self.virtual_key() == VK_E
    }

    #[inline]
    #[must_use]
    pub fn is_l(&self) -> bool {
        self.virtual_key() == VK_L
    }

    #[inline]
    #[must_use]
    pub fn is_q(&self) -> bool {
        self.virtual_key() == VK_Q
    }

    #[inline]
    #[must_use]
    pub fn is_s(&self) -> bool {
        self.virtual_key() == VK_S
    }

    #[inline]
    #[must_use]
    pub fn is_w(&self) -> bool {
        self.virtual_key() == VK_W
    }

    #[inline]
    #[must_use]
    pub fn is_up(&self) -> bool {
        self.virtual_key() == VK_UP
    }

    #[inline]
    #[must_use]
    pub fn is_down(&self) -> bool {
        self.virtual_key() == VK_DOWN
    }

    #[inline]
    #[must_use]
    pub fn is_left(&self) -> bool {
        self.virtual_key() == VK_LEFT
    }

    #[inline]
    #[must_use]
    pub fn is_right(&self) -> bool {
        self.virtual_key() == VK_RIGHT
    }

    #[inline]
    #[must_use]
    pub fn is_escape(&self) -> bool {
        self.virtual_key() == VK_ESCAPE
    }

    #[inline]
    #[must_use]
    fn virtual_key(&self) -> VIRTUAL_KEY {
        #[expect(clippy::cast_possible_truncation)]
        VIRTUAL_KEY(self.w_param.0 as u16)
    }
}
