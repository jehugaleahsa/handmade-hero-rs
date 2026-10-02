use handmade_hero_interface::point_2d::Point2d;
use windows::Win32::Foundation::{POINT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, VIRTUAL_KEY, VK_LBUTTON, VK_MBUTTON, VK_RBUTTON,
};
use windows::Win32::UI::WindowsAndMessaging::{GetCursorPos, WHEEL_DELTA};
use windows::core::Result as Win32Result;

#[derive(Debug)]
pub struct Win32Mouse {
    vertical_wheel_delta: i32,
    horizontal_wheel_delta: i32,
}

impl Win32Mouse {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Win32Mouse {
            vertical_wheel_delta: 0,
            horizontal_wheel_delta: 0,
        }
    }

    #[expect(clippy::unused_self)]
    pub fn coordinates(&self) -> Win32Result<Point2d<i32>> {
        let mut cursor_coordinate = POINT::default();
        unsafe {
            GetCursorPos(&raw mut cursor_coordinate)?;
        }
        let point = Point2d::from_x_y(cursor_coordinate.x, cursor_coordinate.y);
        Ok(point)
    }

    #[inline]
    #[must_use]
    #[expect(clippy::unused_self)]
    pub fn is_left(&self) -> bool {
        Self::is_special_key_down(VK_LBUTTON)
    }

    #[inline]
    #[must_use]
    #[expect(clippy::unused_self)]
    pub fn is_middle(&self) -> bool {
        Self::is_special_key_down(VK_MBUTTON)
    }

    #[inline]
    #[must_use]
    #[expect(clippy::unused_self)]
    pub fn is_right(&self) -> bool {
        Self::is_special_key_down(VK_RBUTTON)
    }

    #[must_use]
    fn is_special_key_down(key: VIRTUAL_KEY) -> bool {
        let key_state = unsafe { GetKeyState(i32::from(key.0)) };
        let is_down_mask = 1 << 15; // The key is down if the high-order bit is set.
        (key_state & is_down_mask) != 0
    }

    #[inline]
    pub fn process_vertical_scroll(&mut self, w_param: WPARAM) {
        self.vertical_wheel_delta += Self::to_scroll_delta(w_param);
    }

    #[inline]
    pub fn process_horizontal_scroll(&mut self, w_param: WPARAM) {
        self.horizontal_wheel_delta += Self::to_scroll_delta(w_param);
    }

    /// Returns the whole notches scrolled since the last call, keeping any partial notch for
    /// the next call.
    #[inline]
    #[must_use]
    pub fn take_vertical_wheel_notches(&mut self) -> i32 {
        Self::take_notches(&mut self.vertical_wheel_delta)
    }

    /// Returns the whole notches scrolled since the last call, keeping any partial notch for
    /// the next call.
    #[inline]
    #[must_use]
    pub fn take_horizontal_wheel_notches(&mut self) -> i32 {
        Self::take_notches(&mut self.horizontal_wheel_delta)
    }

    #[must_use]
    fn take_notches(wheel_delta: &mut i32) -> i32 {
        let wheel_delta_per_notch = WHEEL_DELTA.cast_signed();
        let notches = *wheel_delta / wheel_delta_per_notch;
        *wheel_delta %= wheel_delta_per_notch;
        notches
    }

    #[inline]
    #[must_use]
    fn to_scroll_delta(w_param: WPARAM) -> i32 {
        let high_order = ((w_param.0 >> 16) & 0xFF_FF) as i16;
        high_order as i32
    }
}
